use super::*;

/// Owns the complete admitted worker until it is transferred into its thread.
/// The changing child and its reservation remain with Runner/Finished afterward.
pub(in crate::git) struct Worker {
    pub policy: config::Git,
    pub directory: PathBuf,
    pub request: Request,
    pub progress: Arc<Mutex<Progress>>,
    pub cancel: Cancellation,
    pub admission: Lease,
    pub wake: crate::term::Waker,
}
impl Worker {
    pub fn start(self) -> Result<JoinHandle<Finished>, String> {
        std::thread::Builder::new()
            .name("git-observation".into())
            .spawn(move || self.run())
            .map_err(|e| format!("Start native Git worker: {e}"))
    }
    fn run(self) -> Finished {
        let Self {
            policy,
            directory,
            request,
            progress,
            cancel,
            mut admission,
            wake,
        } = self;
        let private_output = request.private_output();
        let seconds = request.seconds(&policy);
        let mut runner = Runner {
            deadline: Instant::now() + Duration::from_secs(seconds as u64),
            seconds,
            policy,
            directory,
            progress,
            cancel,
            wake: std::sync::Arc::clone(&wake),
            captured: 0,
            private_output,
            transport_diagnostic: None,
            effects: Effects::NotStarted,
            in_progress: false,
            completion: None,
            accept_message: false,
            todo_editor: None,
            driver_admission: None,
            private_index: None,
            workspace: None,
            operation_failure: None,
            failure_verified: false,
            argv: Vec::new(),
            stdout: Vec::new(),
            stderr: Vec::new(),
            truncated: false,
            exit: None,
            end: capture::End::Exited,
            retained: None,
        };
        let observed = runner.run_request(request, &mut admission);
        let mut finished = runner.finish(observed, admission);
        finished.record.admission.retain(finished.record.bytes());
        if let Ok(Publication::Status(observation)) = &mut finished.publication {
            observation.admission.retain(observation.bytes());
        }
        wake();
        finished
    }
}
impl Runner {
    fn run_request(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            Request::Family(request) => self.family(request, admission),
            Request::Repository(request) => self.repository(request, admission),
            Request::ConflictExplain(conflict) => self.explain_conflict(conflict, admission),
            Request::Resolution(request) => self.resolution(request, admission),
            Request::Conflict(request) => self.conflict(request, admission),
            Request::Bisect(request) => self.bisect(request, admission),
            Request::Application(request) => self.application(request, admission),
            Request::Integration(request) => self.integration(request, admission),
            Request::Refs(request) => self.refs(request, admission),
            request => self.run_observation_request(request, admission),
        }
    }
    fn run_observation_request(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            Request::History(request) => self.history(request, admission).map(Candidate::History),
            Request::Index(request) => self.change_index(&request, admission),
            Request::Discard(diff) => self.discard(&diff, admission),
            Request::Commit(request) => self.commit(&request, admission),
            Request::CommitPrepare(request) => self
                .prepare_commit(request, admission)
                .map(Candidate::Composition),
            Request::Diff(target) => self.compare(target, admission).map(Candidate::Diff),
            Request::Status => self
                .observe(admission)
                .map(|(identity, status)| Candidate::Status(identity, status)),
            _ => unreachable!("worker request dispatched by its operation family"),
        }
    }
}
