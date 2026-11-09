//! Revision metadata and subscriptions. The native observer retains no text;
//! supervisors own transport. Every queue has an independent finite ceiling.
use super::{Connection, Host, Service, editor, reserve};
use crate::{buffer, file::watch::Wake};
use rev_plugin_sdk::{identity::*, methods::*, rpc::*};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

const SOURCES: usize = 256;
const SUBSCRIPTIONS: usize = 16; // per package; eight live packages, 128 total
const TOTAL: usize = 1024;
const PACKAGE: usize = 256;

struct Source {
    sequence: u64,
    kind: EventKind,
    stamp: buffer::Stamp,
    origin: Origin,
}
struct Subscription {
    connection: Weak<Connection>,
    package: PackageId,
    generation: Generation,
    params: EventsSubscribeParams,
    cancelled: Arc<AtomicBool>,
    paused: bool,
    after: u64,
}
impl Drop for Subscription {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}
struct State {
    session: OpaqueId,
    serial: u64,
    origin: Option<Origin>,
    sources: VecDeque<Source>,
    deliveries: VecDeque<(Weak<Connection>, Delivery)>,
    subscriptions: BTreeMap<OpaqueId, Subscription>,
    overflow: bool,
    total: Arc<AtomicUsize>,
}
pub(super) struct Events {
    state: Mutex<State>,
    wake: Arc<Wake>,
}
pub(crate) struct OriginGuard {
    events: Arc<Events>,
    previous: Option<Origin>,
}
impl Drop for OriginGuard {
    fn drop(&mut self) {
        self.events.state.lock().expect("event owner").origin = self.previous.take();
    }
}
struct Permit {
    total: Arc<AtomicUsize>,
    package: Arc<AtomicUsize>,
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.total.fetch_sub(1, Ordering::AcqRel);
        self.package.fetch_sub(1, Ordering::AcqRel);
    }
}
pub(super) struct Delivery {
    pub event: BufferEvent,
    pub cancelled: Arc<AtomicBool>,
    pub deadline: Instant,
    _permit: Option<Permit>,
}
#[derive(Default)]
pub(super) struct Mailbox {
    pub queue: VecDeque<Delivery>,
    pub grants: VecDeque<(BufferEvent, Arc<AtomicBool>)>,
    count: Arc<AtomicUsize>,
}
impl Mailbox {
    pub fn cause(
        &self,
        origin: &Origin,
        invocation: Option<&OpaqueId>,
    ) -> Result<Option<(Origin, Arc<AtomicBool>)>, RpcError> {
        let matching = self.grants.iter().rev().filter(|(event, _)| {
            event.origin.operation == origin.operation
                && event.origin.depth == origin.depth
                && invocation.is_none_or(|id| *id == event.invocation())
        });
        let mut retired = false;
        for (event, cancelled) in matching {
            if cancelled.load(Ordering::Acquire) {
                retired = true;
                continue;
            }
            return Ok(Some((event.origin.clone(), Arc::clone(cancelled))));
        }
        if retired {
            Err(RpcError::application(ErrorKind::Cancelled))
        } else {
            Ok(None)
        }
    }
}
impl Events {
    fn new_state(session: OpaqueId) -> State {
        State {
            session,
            serial: 0,
            origin: None,
            sources: VecDeque::new(),
            deliveries: VecDeque::new(),
            subscriptions: BTreeMap::new(),
            overflow: false,
            total: Arc::default(),
        }
    }
    pub fn new(session: OpaqueId, wake: Arc<Wake>) -> Self {
        Self {
            state: Mutex::new(Self::new_state(session)),
            wake,
        }
    }
    pub(super) fn remaining(&self) -> bool {
        let state = self.state.lock().expect("event owner");
        state.overflow || !state.sources.is_empty() || !state.deliveries.is_empty()
    }
}
impl State {
    fn next(&mut self) -> u64 {
        self.serial = self
            .serial
            .checked_add(1)
            .expect("event sequences exhausted");
        self.serial
    }
    fn native_origin(&mut self) -> Origin {
        let serial = self.next();
        Origin {
            package: None,
            operation: OpaqueId::new(format!("source-{}:{serial}", self.session))
                .expect("finite origin"),
            depth: 0,
        }
    }
    fn resnapshot(&mut self, id: OpaqueId, generation: Generation) -> BufferEvent {
        let origin = self.native_origin();
        BufferEvent {
            subscription: id,
            sequence: Revision::new(self.next().to_string()).expect("finite sequence"),
            generation,
            kind: EventKind::Resnapshot,
            stamp: None,
            origin,
        }
    }
    fn admit(
        &self,
        connection: &Arc<Connection>,
        delivery: &mut Delivery,
        invalidation: bool,
    ) -> bool {
        if delivery._permit.is_some() {
            return true;
        }
        let package = Arc::clone(&connection.events.lock().expect("event mailbox").count);
        // Permits include fan-out staging, queued metadata and encoded frames.
        // Reserve one invalidation slot for every admitted subscription.
        let package_limit = if invalidation {
            PACKAGE
        } else {
            PACKAGE - SUBSCRIPTIONS
        };
        let total_limit = if invalidation {
            TOTAL
        } else {
            TOTAL - 8 * SUBSCRIPTIONS
        };
        if !reserve(&package, 1, package_limit) {
            return false;
        }
        if !reserve(&self.total, 1, total_limit) {
            package.fetch_sub(1, Ordering::AcqRel);
            return false;
        }
        delivery._permit = Some(Permit {
            total: Arc::clone(&self.total),
            package,
        });
        true
    }
    fn enqueue(
        &self,
        connection: &Arc<Connection>,
        mut delivery: Delivery,
        invalidation: bool,
    ) -> bool {
        if !self.admit(connection, &mut delivery, invalidation) {
            return false;
        }
        connection
            .events
            .lock()
            .expect("event mailbox")
            .queue
            .push_back(delivery);
        connection.signal();
        true
    }
    fn invalidate(&mut self, connection: &Arc<Connection>) {
        let ids: Vec<_> = self
            .subscriptions
            .iter()
            .filter(|(_, sub)| !sub.paused && sub.connection.ptr_eq(&Arc::downgrade(connection)))
            .map(|(id, _)| id.clone())
            .collect();
        for id in &ids {
            let sub = self.subscriptions.get_mut(id).expect("subscription");
            sub.cancelled.store(true, Ordering::Release);
            sub.cancelled = Arc::default();
            sub.paused = true;
        }
        connection
            .events
            .lock()
            .expect("event mailbox")
            .queue
            .retain(|d| !d.cancelled.load(Ordering::Acquire));
        for id in ids {
            let sub = &self.subscriptions[&id];
            let generation = sub.generation.clone();
            let cancelled = Arc::clone(&sub.cancelled);
            let event = self.resnapshot(id, generation);
            let mut delivery = Delivery {
                event,
                cancelled,
                deadline: Instant::now() + Duration::from_secs(5),
                _permit: None,
            };
            if !self.admit(connection, &mut delivery, true) {
                connection
                    .notice("Plugin event invalidation unavailable; generation retired".into());
                connection.stop();
                break;
            }
            self.deliveries
                .push_back((Arc::downgrade(connection), delivery));
        }
    }
}
impl Source {
    fn event(
        &self,
        id: OpaqueId,
        generation: Generation,
        sequence: u64,
        session: &OpaqueId,
    ) -> BufferEvent {
        BufferEvent {
            subscription: id,
            sequence: Revision::new(sequence.to_string()).expect("finite sequence"),
            generation,
            kind: self.kind,
            stamp: Some(Stamp {
                buffer: Handle {
                    session: session.clone(),
                    id: OpaqueId::new(format!("buffer-{}", self.stamp.buffer))
                        .expect("native buffer"),
                },
                revision: Revision::new(self.stamp.revision.to_string()).expect("native revision"),
            }),
            origin: self.origin.clone(),
        }
    }
}
impl Subscription {
    fn observes(&self, source: &Source) -> bool {
        !self.paused
            && source.sequence > self.after
            && self.params.kinds.contains(&source.kind)
            && (self.params.buffers.is_empty()
                || self
                    .params
                    .buffers
                    .iter()
                    .any(|h| h.id.as_str() == format!("buffer-{}", source.stamp.buffer)))
    }
}
impl State {
    fn deliver(&mut self, connection: &Weak<Connection>, delivery: Delivery) {
        if delivery.cancelled.load(Ordering::Acquire) {
            return;
        }
        let Some(connection) = connection.upgrade() else {
            return;
        };
        if connection.stop.load(Ordering::Acquire) {
            return;
        }
        let invalidation = delivery.event.kind == EventKind::Resnapshot;
        if !self.enqueue(&connection, delivery, invalidation) {
            if invalidation {
                connection
                    .notice("Plugin event invalidation unavailable; generation retired".into());
                connection.stop();
            } else {
                self.invalidate(&connection);
            }
        }
    }
    fn fan_out(&mut self, source: &Source) {
        let ids: Vec<_> = self
            .subscriptions
            .iter()
            .filter(|(_, sub)| sub.observes(source))
            .map(|(id, _)| id.clone())
            .collect();
        for id in ids {
            let sub = &self.subscriptions[&id];
            if sub.paused {
                continue;
            }
            let Some(connection) = sub.connection.upgrade() else {
                continue;
            };
            if connection.stop.load(Ordering::Acquire) {
                continue;
            }
            let cancelled = Arc::clone(&sub.cancelled);
            let generation = sub.generation.clone();
            let sequence = self.next();
            let event = source.event(id, generation, sequence, &self.session);
            let mut delivery = Delivery {
                event,
                cancelled,
                deadline: Instant::now() + Duration::from_secs(60),
                _permit: None,
            };
            if !self.admit(&connection, &mut delivery, false) {
                self.invalidate(&connection);
                continue;
            }
            self.deliveries
                .push_back((Arc::downgrade(&connection), delivery));
        }
    }
}

impl buffer::events::Observer for Events {
    fn published(&self, kind: buffer::events::Kind, stamp: buffer::Stamp) {
        let mut state = self.state.lock().expect("event owner");
        state.subscriptions.retain(|_, sub| {
            sub.connection
                .upgrade()
                .is_some_and(|c| !c.stop.load(Ordering::Acquire))
        });
        if state.subscriptions.values().all(|sub| sub.paused) {
            return;
        }
        if state.sources.len() == SOURCES {
            state.sources.clear();
            state.deliveries.retain(|(_, delivery)| {
                delivery.event.kind == EventKind::Resnapshot
                    && !delivery.cancelled.load(Ordering::Acquire)
            });
            state.overflow = true;
        }
        if !state.overflow {
            let kind = match kind {
                buffer::events::Kind::Opened => EventKind::Opened,
                buffer::events::Kind::Changed => EventKind::Changed,
                buffer::events::Kind::Closed => EventKind::Closed,
            };
            let origin = state
                .origin
                .clone()
                .unwrap_or_else(|| state.native_origin());
            let sequence = state.next();
            state.sources.push_back(Source {
                sequence,
                kind,
                stamp,
                origin,
            });
        }
        drop(state);
        self.wake.signal();
    }
}
impl Host {
    pub fn event_observer(&self) -> Arc<dyn buffer::events::Observer> {
        Arc::<Events>::clone(&self.events)
    }
    pub fn event_root(&self) -> Origin {
        self.events
            .state
            .lock()
            .expect("event owner")
            .native_origin()
    }
    pub fn event_origin(&self, origin: Origin) -> OriginGuard {
        let previous = self
            .events
            .state
            .lock()
            .expect("event owner")
            .origin
            .replace(origin);
        OriginGuard {
            events: Arc::clone(&self.events),
            previous,
        }
    }
    pub fn subscribe(
        &self,
        request: &editor::Request,
        params: EventsSubscribeParams,
    ) -> Result<EventsSubscribeResult, RpcError> {
        let mut state = self.events.state.lock().expect("event owner");
        state.subscriptions.retain(|_, sub| {
            sub.connection
                .upgrade()
                .is_some_and(|c| !c.stop.load(Ordering::Acquire))
        });
        if state
            .subscriptions
            .values()
            .filter(|s| s.package == request.package)
            .count()
            >= SUBSCRIPTIONS
            || state.subscriptions.len() >= 8 * SUBSCRIPTIONS
        {
            return Err(RpcError::application(ErrorKind::LimitExceeded));
        }
        if params
            .buffers
            .iter()
            .enumerate()
            .any(|(i, h)| params.buffers.iter().take(i).any(|v| v == h))
        {
            return Err(RpcError::application(ErrorKind::InvalidArguments));
        }
        let id =
            OpaqueId::new(format!("subscription-{}", state.next())).expect("finite subscription");
        let sequence = Revision::new(state.serial.to_string()).expect("finite sequence");
        state.subscriptions.insert(
            id.clone(),
            Subscription {
                connection: Arc::downgrade(&request.connection),
                package: request.package.clone(),
                generation: request.generation.clone(),
                params,
                cancelled: Arc::default(),
                paused: false,
                after: sequence.as_str().parse().expect("native sequence"),
            },
        );
        drop(state);
        Ok(EventsSubscribeResult {
            subscription: id,
            sequence,
        })
    }
    pub fn unsubscribe(&self, request: &editor::Request, id: &OpaqueId) -> Result<(), RpcError> {
        let mut state = self.events.state.lock().expect("event owner");
        if !state
            .subscriptions
            .get(id)
            .is_some_and(|s| s.package == request.package && s.generation == request.generation)
        {
            return Err(RpcError::application(ErrorKind::InvalidArguments));
        }
        state.subscriptions.remove(id);
        drop(state);
        request.connection.signal();
        Ok(())
    }
    pub fn service_events(&mut self, service: &mut Service) {
        let mut state = self.events.state.lock().expect("event owner");
        while service.take() {
            if state.overflow {
                state.overflow = false;
                let connections: Vec<_> = self
                    .slots
                    .values()
                    .filter(|s| !s.retired)
                    .map(|s| Arc::clone(&s.connection))
                    .collect();
                for connection in connections {
                    state.invalidate(&connection);
                }
                continue;
            }
            if let Some((connection, delivery)) = state.deliveries.pop_front() {
                state.deliver(&connection, delivery);
                continue;
            }
            let Some(source) = state.sources.pop_front() else {
                break;
            };
            state.fan_out(&source);
        }
        drop(state);
    }
}
