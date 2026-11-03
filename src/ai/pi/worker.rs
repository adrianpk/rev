use super::{
    Answer, Control, Profile, Purpose, Request, directory, history, io, protocol, resource,
};
use crate::config::Pi;
use serde_json::json;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, atomic::Ordering};
use std::time::{Duration, Instant};

pub(super) fn run(
    policy: Pi,
    root: PathBuf,
    root_handle: Option<File>,
    mut request: Request,
    profile: Profile,
    control: &Arc<Control>,
) -> Result<Option<Answer>, String> {
    let canonical = root
        .canonicalize()
        .map_err(|_| "Pi working directory unavailable")?;
    if profile.directory && canonical.as_os_str() != root.as_os_str() {
        return Err("Pi canonical root changed before launch".into());
    }
    let root = canonical;
    let metadata = std::fs::metadata(&root).map_err(|_| "Pi root observation failed")?;
    if !metadata.is_dir() {
        return Err("Pi working directory is not a directory".into());
    }
    if let Some(handle) = &root_handle {
        let held = handle.metadata().map_err(|_| "Pi root handle failed")?;
        if (held.dev(), held.ino()) != (metadata.dev(), metadata.ino()) {
            return Err("Pi directory root changed before launch".into());
        }
    }
    probe(&policy, &root, control, request.deadline)?;
    let mut store = if profile.retained && policy.persist_sessions {
        let state = profile.state.clone().map(Ok).unwrap_or_else(|| {
            crate::config::Roots::from_env()
                .map(|r| r.state)
                .map_err(|_| "Pi state root unavailable".to_string())
        })?;
        Some(history::Store::open(&state)?)
    } else {
        None
    };
    let candidate = store.as_ref().map(history::Store::candidate).transpose()?;
    let resource = if profile.directory {
        Some(resource::Resource::new()?)
    } else {
        None
    };
    if control.canceled() {
        return Err("Pi invocation canceled".into());
    }
    let (pipes, [stdin, stdout, stderr]) = io::Pipes::new()?;
    let mut command = Command::new(&policy.command);
    command.args([
        "--mode",
        "rpc",
        "--no-extensions",
        "--no-mcp",
        "--no-skills",
        "--no-prompt-templates",
        "--no-context-files",
        "--no-themes",
        "--no-approve",
        "--offline",
    ]);
    if let Some(path) = &candidate {
        command.arg("--session").arg(path).arg("--session-dir").arg(
            path.parent()
                .ok_or("Pi private session directory missing")?,
        );
    } else {
        command.arg("--no-session");
    }
    if let Some(resource) = &resource {
        resource
            .check()
            .map_err(|_| "Pi internal resource changed")?;
        command
            .args(["--tools", "rev_read,rev_list,rev_search", "--extension"])
            .arg(&resource.path);
    } else {
        command.arg("--no-tools");
    }
    if !matches!(request.purpose, Purpose::Models)
        && let Some(model) = &policy.model
    {
        command.arg("--model").arg(model);
    }
    let mut child = io::Owned::new(
        command
            .current_dir(&root)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .map_err(|_| "Pi RPC launch failed; check ai.pi.command")?,
        control.clone(),
    );
    if candidate.is_some() {
        child.limit_history()?;
    }
    let output=pipes.drive(&mut child,|inbox,input,_child| {
        let mut private_roots=if profile.directory {directory::private_roots()?}else{Vec::new()};
        if let Some(path)=&candidate && let Some(parent)=path.parent().and_then(|p|p.parent()) {private_roots.push(parent.to_owned());}
        let mut lease=None; let mut history_ready=false; let mut model=String::new(); let mut reused=false;
        loop {
            let result=(|| {
                let answer=match &request.purpose {
                    Purpose::Models=> {
                        let data=protocol::request(inbox,input,control,request.epoch,request.deadline,"get_available_models",json!({}))?;
                        Answer::Models(protocol::models(&data)?)
                    }
                    Purpose::Generate(message)=> {
                        let admit_before=profile.directory && (history_ready || candidate.is_none());
                        if admit_before {
                            let grant=json!({"version":1,"nonce":request.epoch,"root":root,"dev":metadata.dev().to_string(),"ino":metadata.ino().to_string(),"timeout_ms":request.deadline.saturating_duration_since(Instant::now()).as_millis().min(1_800_000) as u64,"max_input_bytes":policy.max_input_bytes,"private_roots":private_roots});
                            protocol::admit(inbox,input,control,request.epoch,request.deadline,&grant)?;
                        }
                        let state=protocol::request(inbox,input,control,request.epoch,request.deadline,"get_state",json!({}))?;
                        let selected=lease.as_ref().map(|l: &history::Lease|l.path.as_path()).or(candidate.as_deref());
                        let observed=protocol::state(&state,selected,!history_ready)?;
                        if history_ready && observed!=model {return Err("Pi model changed across an owned session boundary".into());}
                        model=observed;
                        if !history_ready {
                            if let Some(store)=store.take() {
                                let fingerprint=fingerprint(&policy,&metadata,&private_roots)?;
                                let chosen=store.bind(&root,&fingerprint,&model,candidate.as_deref().ok_or("Pi candidate missing")?,profile.reset)?;
                                reused=Some(&chosen.path)!=candidate.as_ref();
                                if profile.reset {control.reset_applied.store(true,Ordering::Release);}
                                if reused {
                                    let switched=protocol::request(inbox,input,control,request.epoch,request.deadline,"switch_session",json!({"sessionPath":chosen.path}))?;
                                    if switched["cancelled"]!=false {return Err("Pi history switch was canceled".into());}
                                    let state=protocol::request(inbox,input,control,request.epoch,request.deadline,"get_state",json!({}))?;
                                    if protocol::state(&state,Some(&chosen.path),false)?!=model {return Err("Pi restored a different model/provider; preserve history and reset it".into());}
                                }
                                lease=Some(chosen);
                            }
                            if profile.reset {control.reset_applied.store(true,Ordering::Release);}
                            history_ready=true;
                        }
                        if profile.directory && !admit_before {
                            let grant=json!({"version":1,"nonce":request.epoch,"root":root,"dev":metadata.dev().to_string(),"ino":metadata.ino().to_string(),"timeout_ms":request.deadline.saturating_duration_since(Instant::now()).as_millis().min(1_800_000) as u64,"max_input_bytes":policy.max_input_bytes,"private_roots":private_roots});
                            protocol::admit(inbox,input,control,request.epoch,request.deadline,&grant)?;
                        }
                        let provenance=if lease.is_some() {if reused {"private directory history; resumed"}else{"private directory history; new"}}
                            else if profile.directory {if reused {"in-memory directory history; reused"}else{"fresh ephemeral directory history"}}
                            else {"fresh input-only history"};
                        let output=protocol::generate(inbox,input,protocol::Generation {control,policy:&policy,message,epoch:request.epoch,deadline:request.deadline,model:model.clone(),directory:profile.directory,history:provenance.into(),lease:lease.as_mut()})?;
                        if let Some(lease)=&mut lease {lease.touch()?;}
                        reused=true;
                        Answer::Generated(output)
                    }
                };
                Ok(answer)
            })();
            let answer=match result {Ok(answer)=>answer,Err(error)=>{protocol::cancel(inbox,input,request.epoch);return Err(error);}};
            if !profile.retained || policy.idle_timeout_ms==0 {return Ok(Some(answer));}
            let until=Instant::now()+Duration::from_millis(policy.idle_timeout_ms as u64);
            {
                let mut mailbox=control.mailbox.lock().expect("Pi mailbox");
                mailbox.result=Some(answer);mailbox.idle_until=Some(until);
            }
            loop {
                {
                    let mut mailbox=control.mailbox.lock().expect("Pi mailbox");
                    if let Some(next)=mailbox.pending.take() {request=next;break;}
                    if control.canceled() || Instant::now()>=until {mailbox.closed=true;mailbox.idle_until=None;return Ok(None);}
                }
                if inbox.next(until.saturating_duration_since(Instant::now()))?.is_some() {
                    return Err("Pi emitted work after settlement; the owned connection was retired".into());
                }
            }
        }
    })?;
    Ok(output)
}
fn fingerprint(
    policy: &Pi,
    root: &std::fs::Metadata,
    private: &[PathBuf],
) -> Result<String, String> {
    // Literal canonical policy data, not a hash treated as proof of equivalence.
    serde_json::to_string(&json!({"grant":"rev-pi-read-list-search-v2","pi":"1.3.0","root_dev":root.dev().to_string(),"root_ino":root.ino().to_string(),"command":policy.command,"model_selection":policy.model,"persist":policy.persist_sessions,"idle_ms":policy.idle_timeout_ms,"request_ms":policy.request_timeout_ms,"input":policy.max_input_bytes,"result":policy.max_result_bytes,"tools":["rev_read","rev_list","rev_search"],"private":private,"links":"reject","turns":8,"calls":32,"returned":4194304,"file":1048576})).map_err(|_| "Pi grant fingerprint encoding failed".into())
}

fn probe(
    policy: &Pi,
    root: &std::path::Path,
    control: &Arc<Control>,
    deadline: Instant,
) -> Result<(), String> {
    let (pipes, stdio) = io::Pipes::new()?;
    let [stdin, stdout, stderr] = stdio;
    let mut child = io::Owned::new(
        Command::new(&policy.command)
            .arg("--version")
            .env("PI_OFFLINE", "1")
            .current_dir(root)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .map_err(|_| "Pi is unavailable; install Pi 1.3.0 or set ai.pi.command")?,
        control.clone(),
    );
    pipes.drive(&mut child, |inbox, _input, child| {
        let deadline = deadline.min(Instant::now() + Duration::from_secs(5));
        loop {
            if control.canceled() {
                return Err("Pi invocation canceled".into());
            }
            if Instant::now() >= deadline {
                return Err("Pi version probe exceeded its deadline".into());
            }
            if let Some(bytes) = inbox.next(Duration::from_millis(20))? {
                if bytes != b"1.3.0" {
                    return Err(
                        "Unsupported Pi version; this adapter requires checked Pi 1.3.0".into(),
                    );
                }
                // Version output is an admission observation, not model work.
                // The same executable is used for the owned RPC child.
                let exit = Instant::now() + Duration::from_secs(1);
                while Instant::now() < exit {
                    if let Some(status) = child.try_wait().map_err(|_| "Pi version wait failed")? {
                        return if status.success() {
                            Ok(())
                        } else {
                            Err("Pi version probe failed".into())
                        };
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                return Err("Pi version probe did not exit".into());
            }
        }
    })
}
