use super::*;
use crate::conflicts::presentation::open;
use crate::status::presentation::{editor, entry, key, plain, text, wait};
use crossterm::event::{KeyCode, KeyModifiers};
use rev::command::Command as Native;
use std::os::unix::fs::PermissionsExt;

fn peer(repository: &Repository, mode: &str) -> std::path::PathBuf {
    // The existing test executable supplies the controlled RPC boundary. The
    // shell preserves the protocol socket as stderr while libtest's ordinary
    // stdout stays separate. Duplicating the descriptor also supports sockets.
    let quote = |s: &std::ffi::OsStr| format!("'{}'", s.to_str().unwrap().replace('\'', r"'\''"));
    let root = quote(repository.fixture.0.as_os_str());
    let executable = quote(std::env::current_exe().unwrap().as_os_str());
    let script = format!(
        "#!/bin/sh\nexport REV_GIT_CONFLICT_PEER_DIR={root}\nif [ \"$1\" = '--version' ]; then : > {root}/version-pid; printf '1.3.0\\n'; exit 0; fi\nprintf '%s\\n' \"$@\" > {root}/args\n: > {root}/rpc-started\nexec {executable} --exact conflicts::explanation::rpc_peer --nocapture --test-threads=1 2>&1 > /dev/null\n"
    );
    let path = repository.fixture.write("pi-peer", &script);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    repository.fixture.write("mode", mode);
    path
}

// Child-only fixture entrypoint, following the repository's current_exe test
// pattern. It emits controlled RPC facts, never provider requests or secrets.
#[test]
fn rpc_peer() {
    use std::io::{BufRead, Read, Write};
    use std::time::{Duration, Instant};
    let Some(root) = std::env::var_os("REV_GIT_CONFLICT_PEER_DIR") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let gate = fs::read_to_string(root.join("mode")).unwrap() == "settle_gate";
    let pipe = fs::File::from(rustix::io::dup(std::io::stderr()).unwrap());
    let mut pipe = std::io::BufWriter::new(pipe);
    let mut send = |value: serde_json::Value| {
        serde_json::to_writer(&mut pipe, &value).unwrap();
        pipe.write_all(b"\n").unwrap();
        pipe.flush().unwrap();
    };
    let input = std::io::stdin();
    let mut messages = 0;
    for line in input.lock().take(4_259_840).lines() {
        let request: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let kind = request["type"].as_str().unwrap();
        let data = match kind {
            "get_state" => {
                serde_json::json!({"isStreaming":false,"isCompacting":false,"messageCount":messages,"pendingMessageCount":0,"model":{"provider":"fixture","id":"controlled"}})
            }
            "prompt" => serde_json::json!({"disposition":"started"}),
            "clear_queue" => serde_json::json!({"steering":[],"followUp":[]}),
            _ => serde_json::Value::Null,
        };
        if kind == "prompt" {
            fs::write(root.join("prompt"), request["message"].as_str().unwrap()).unwrap();
            messages += 1;
        }
        send(
            serde_json::json!({"type":"response","id":request["id"],"command":kind,"success":true,"data":data}),
        );
        if kind == "prompt" {
            let answer = serde_json::to_string(&serde_json::json!({"version":1,"text":"Captured conflict explanation. Unicode: é."})).unwrap();
            send(
                serde_json::json!({"type":"message_end","message":{"role":"assistant","stopReason":"stop","content":[{"type":"text","text":answer}]}}),
            );
            send(serde_json::json!({"type":"agent_end","messages":[]}));
            if gate {
                fs::write(root.join("partial"), "").unwrap();
                let deadline = Instant::now() + Duration::from_secs(5);
                while !root.join("release").exists() {
                    assert!(
                        Instant::now() < deadline,
                        "owned conflict peer settlement deadline"
                    );
                    std::thread::sleep(Duration::from_millis(2));
                }
            }
            send(serde_json::json!({"type":"agent_settled","aborted":false}));
        }
    }
    fs::write(root.join("closed"), "").unwrap();
}
fn configured(path: &Path) -> String {
    format!(
        "[ai.pi]\ncommand={}\npersist_sessions=false\nrequest_timeout_ms=5000\n",
        serde_json::to_string(path).unwrap()
    )
}
fn prompt(repository: &Repository) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(repository.fixture.0.join("prompt")).unwrap()).unwrap()
}
fn enter(editor: &mut rev::session::Session) {
    entry(editor);
    wait(editor, |e| e.echo().contains("Git Status refreshed"));
    open(editor);
}

// Missing executable or complete input admission failure leaves the same
// manual conflict view and exact side-resolution route usable without AI.
#[test]
fn optional_ai() {
    for variant in ["missing", "bound", "stale"] {
        let repository = Repository::new("sha1");
        let program = if variant == "missing" {
            repository.fixture.0.join("missing-pi")
        } else {
            peer(&repository, "normal")
        };
        let mut settings = configured(&program);
        if variant == "bound" {
            settings.push_str("max_input_bytes=64\n");
        }
        let mut owned = editor(&repository, "modal", &settings, true);
        let editor = &mut owned.0;
        topic(
            &repository,
            Path::new("file"),
            b"base\n",
            b"current\n",
            b"applied\n",
        );
        stopped(&repository, &["merge", "topic"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        enter(editor);
        if variant == "stale" {
            fs::write(repository.root.join("file"), "changed saved result\n").unwrap();
        }
        let disk = fs::read(repository.root.join("file")).unwrap();
        plain(editor, 'H');
        wait(editor, |e| {
            let echo = e.echo().to_ascii_lowercase();
            if variant == "missing" {
                echo.contains("pi")
                    && (echo.contains("unavailable")
                        || echo.contains("failed")
                        || echo.contains("cannot")
                        || echo.contains("could not"))
            } else if variant == "bound" {
                echo.contains("explanation") && echo.contains("admission")
            } else {
                echo.contains("capture failed or became stale")
            }
        });
        assert!(editor.git_visible());
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(fs::read(repository.root.join("file")).unwrap(), disk);
        assert!(!repository.fixture.0.join("prompt").exists());
        if variant == "stale" {
            open(editor);
        }
        plain(editor, '2');
        wait(editor, |e| {
            e.echo().contains("Confirm replace result with Current")
        });
        plain(editor, 'n');
        assert!(editor.echo().contains("Conflict resolution canceled"));
    }
}

// The native H route copies full role/object/source facts in both profiles,
// including raw paths, unsaved text and explicit binary metadata. Its existing
// AI result is read-only and cannot stage, edit or continue a Git sequence.
#[test]
fn copied_request() {
    for (profile, format, variant) in [
        ("non-modal", "sha1", "text"),
        ("modal", "sha256", "dirty"),
        ("modal", "sha1", "binary"),
    ] {
        let repository = Repository::new(format);
        let program = peer(&repository, "normal");
        let mut owned = editor(&repository, profile, &configured(&program), true);
        let editor = &mut owned.0;
        let path = if variant == "dirty" {
            std::path::PathBuf::from(OsString::from_vec(b"-file\n\xff".to_vec()))
        } else {
            "file".into()
        };
        if variant == "binary" {
            topic(&repository, &path, b"base\0", b"current\0", b"applied\0");
        } else {
            topic(&repository, &path, b"base\n", b"current\n", b"applied\n");
        }
        stopped(&repository, &["merge", "topic"]);
        enter(editor);
        if variant == "dirty" {
            key(editor, KeyCode::Enter, KeyModifiers::NONE);
            plain(editor, 'i');
            plain(editor, 'x');
            enter(editor);
        }
        let native = text(editor);
        let disk = fs::read(repository.root.join(&path)).unwrap();
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let head = repository.git(["rev-parse", "HEAD"]);
        plain(editor, 'H');
        wait(editor, |e| e.echo().contains("Pi explanation ready"));
        assert!(!editor.git_visible());
        assert!(editor.buffer().unwrap().is_readonly());
        let envelope = prompt(&repository);
        assert_eq!(envelope["context"], "input");
        assert_eq!(envelope["output"], "explanation");
        assert_eq!(envelope["source_write_authority"], false);
        assert_eq!(envelope["dirty_sources"], serde_json::json!([]));
        assert_eq!(envelope["observations"]["stage_authority"], false);
        assert_eq!(envelope["observations"]["continue_authority"], false);
        assert_eq!(envelope["observations"]["kind"], "git-conflict");
        let context: serde_json::Value =
            serde_json::from_str(envelope["observations"]["text"].as_str().unwrap()).unwrap();
        assert_eq!(context["kind"], "both modified");
        assert_eq!(context["operation"], "Some(Merge)");
        assert_eq!(
            context["head"].as_str().unwrap().as_bytes(),
            head.strip_suffix(b"\n").unwrap()
        );
        for (stage, role) in context["stages"]
            .as_array()
            .unwrap()
            .iter()
            .zip(["Base", "Current", "Applied"])
        {
            assert_eq!(stage["role"], role);
            assert!(!stage["meaning"].as_str().unwrap().is_empty());
            let object = stage["object"].as_str().unwrap();
            assert_eq!(object.len(), if format == "sha256" { 64 } else { 40 });
            let actual = repository.git(["cat-file", "blob", object]);
            if variant == "binary" {
                assert_eq!(stage["text"], serde_json::Value::Null);
                assert!(stage["binary"].as_bool().unwrap());
            } else {
                assert_eq!(stage["text"].as_str().unwrap().as_bytes(), actual);
            }
        }
        if variant == "dirty" {
            assert_eq!(context["visiting_result"]["text"], native);
            assert_eq!(context["visiting_result"]["dirty"], true);
            let path = context["path"].as_str().unwrap();
            assert_eq!(path, "-file\\n\\xFF");
            assert!(!path.contains('\u{fffd}'));
        }
        if variant == "binary" {
            assert_eq!(envelope["observations"]["complete"], false);
            assert_eq!(context["saved_result"]["state"], "unavailable");
        } else {
            assert_eq!(
                context["saved_result"]["text"].as_str().unwrap().as_bytes(),
                disk
            );
        }
        let args: Vec<_> = fs::read_to_string(repository.fixture.0.join("args"))
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect();
        assert!(args.iter().any(|a| a == "--no-tools"));
        assert!(!args.iter().any(|a| a == "--extension"));
        editor.execute(Native::AiApplyResult);
        assert!(editor.echo().to_ascii_lowercase().contains("explanation"));
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        assert_eq!(fs::read(repository.root.join(&path)).unwrap(), disk);
        editor.execute(Native::AiDiscardResult);
        assert_eq!(text(editor), native);
        if variant == "dirty" {
            assert!(editor.buffer().unwrap().is_modified());
        }
    }
}

// Provider settlement is gated by an owned fixture marker. Changes to saved
// bytes, actual index state or a visiting native revision invalidate currency
// without altering the copied request or granting any mutation authority.
#[test]
fn historical_source() {
    for changed in ["disk", "index", "buffer"] {
        let repository = Repository::new("sha256");
        let program = peer(&repository, "settle_gate");
        let mut owned = editor(&repository, "modal", &configured(&program), true);
        let editor = &mut owned.0;
        topic(
            &repository,
            Path::new("file"),
            b"base\n",
            b"current\n",
            b"applied\n",
        );
        stopped(&repository, &["merge", "topic"]);
        enter(editor);
        if changed == "buffer" {
            key(editor, KeyCode::Enter, KeyModifiers::NONE);
            enter(editor);
        }
        plain(editor, 'H');
        wait(editor, |_| repository.fixture.0.join("partial").exists());
        assert!(editor.git_visible());
        let captured = fs::read(repository.fixture.0.join("prompt")).unwrap();
        match changed {
            "disk" => fs::write(repository.root.join("file"), "changed after capture\n").unwrap(),
            "index" => {
                repository.git(["add", "--", "file"]);
            }
            _ => {
                plain(editor, 'q');
                plain(editor, 'q');
                plain(editor, 'q');
                plain(editor, 'i');
                plain(editor, 'x');
            }
        }
        let disk = fs::read(repository.root.join("file")).unwrap();
        let index = repository.git(["ls-files", "--stage", "-z"]);
        repository.fixture.write("release", "");
        wait(editor, |e| e.echo().contains("Pi explanation ready"));
        assert_eq!(
            fs::read(repository.fixture.0.join("prompt")).unwrap(),
            captured
        );
        if changed != "buffer" {
            assert!(text(editor).contains("historical snapshot"));
        } else {
            assert!(text(editor).starts_with('x'));
        }
        assert_eq!(fs::read(repository.root.join("file")).unwrap(), disk);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        editor.execute(Native::AiDiscardResult);
    }
}
