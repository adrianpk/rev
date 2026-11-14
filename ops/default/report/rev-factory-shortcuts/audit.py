#!/usr/bin/env python3
"""Reproduce the documentation audit without loading personal configuration.

Compile the candidate's library into a private temporary directory. Reuse only
prebuilt dependency inputs, never a prebuilt editor library or a shared output
directory. --write refreshes evidence; the default compares without writing.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib


NATIVE = r'''
#![forbid(unsafe_code)]
use rev::buffer::Buffers;
use rev::command::{Command, Navigation};
use rev::command::navigation::Target;
use rev::config::{InteractionProfile, Keymaps, prepare};
use rev::coordinates::{Coordinates, Scalar};
use rev::input::keymap::{self, Context, Sequence};
use rev::session::Session;
use rev::view::{Size, View};
use serde_json::{Value, json};
use std::path::Path;

fn named(name: &str) -> Command { Command::named(name).unwrap() }
fn text(s: &Session) -> String { let b=s.buffer().unwrap(); b.slice(&b.span(0..b.len_chars()).unwrap()).unwrap().to_string() }
fn session(root: &Path, config: &str, source: &str, point: usize) -> Session {
    let path = root.join("config.toml");
    std::fs::write(&path, config).unwrap();
    let settings = prepare(&path);
    assert!(settings.diagnostic.is_none(), "{:?}", settings.diagnostic);
    source_session(&settings.snapshot, source, point)
}
fn source_session(settings: &rev::config::Snapshot, source: &str, point: usize) -> Session {
    let mut buffers = Buffers::with_settings(settings.clone()).unwrap();
    let id = buffers.create(source).unwrap();
    let mut view = View::with_settings(buffers.get(id).unwrap(), settings);
    view.point.position = Scalar(point);
    Session::new(buffers, view, Size { width: 100, height: 32 }).unwrap()
}
fn lookup(maps: &keymap::Effective, context: Context, key: &str) -> Option<String> {
    maps.map(context).lookup(&Sequence::parse(key).unwrap()).map(|b| b.name().into())
}
fn first_text(root: &Path) -> Value {
    let cases = [("spaces", "    x", 4), ("tab", "\tx", 1),
        ("mixed", " \t x", 3), ("blank", " \t ", 3), ("empty", "", 0),
        ("unicode-space", "\u{a0}x", 1), ("wide", "  界", 2),
        ("combining", "  e\u{301}", 2)];
    let states = ["modal-enter-normal", "modal-enter-insert", "modal-enter-replace",
        "modal-enter-visual-character", "modal-enter-visual-line", "modal-enter-visual-block"];
    let snapshots = ["", "[editor]\ninteraction_profile='modal'"].map(|config| {
        let path = root.join("config.toml");
        std::fs::write(&path,config).unwrap();
        let prepared = prepare(&path);
        assert!(prepared.diagnostic.is_none());
        prepared.snapshot
    });
    for (_, source, expected) in cases {
        for state in std::iter::once(None).chain(states.iter().map(|s| Some(*s))) {
            let mut s = source_session(&snapshots[usize::from(state.is_some())],source,source.chars().count());
            s.execute(named("set-mark-command"));
            if let Some(state) = state { s.execute(named(state)); }
            let before = s.editing_state();
            let stamp = s.buffer().unwrap().stamp();
            let mark = s.buffer().unwrap().mark();
            let visual = s.buffer().unwrap().visual();
            s.execute(Command::Navigate(Navigation {
                target: Target::FirstText, count: Some(4), character: None }));
            assert_eq!(s.view().point.position, Scalar(expected));
            assert_eq!(s.editing_state(), before);
            assert_eq!(s.buffer().unwrap().stamp(), stamp);
            assert_eq!(s.buffer().unwrap().mark(), mark);
            assert_eq!(s.buffer().unwrap().visual(), visual);
            assert!(s.buffer().unwrap().history().is_empty());
            assert_eq!(text(&s), source);
        }
    }
    json!({"cases": cases.map(|(name, _, offset)| json!({"case":name,"offset":offset})),
        "sessions":56,"count":4,"unchanged":["source","stamp","history","editing-state","mark","selection-shape"]})
}
fn mappings() -> Value {
    let factory = keymap::compile(&Keymaps::default()).unwrap();
    for context in [Context::Editor, Context::VimNormal, Context::VimInsert,
        Context::VimReplace, Context::VimVisualCharacter, Context::VimVisualLine, Context::VimVisualBlock] {
        for key in ["M-m","C-x +","C-x p p"] {
            assert_eq!(lookup(&factory,context,key),None);
        }
    }
    for context in [Context::Editor,Context::VimInsert,Context::VimReplace] {
        assert_eq!(lookup(&factory,context,"C-M-_"),None);
    }
    let mut keys = Keymaps::default();
    for (key, command) in [("M-m","modal-line-first-text"), ("C-x +","balance-windows"),
        ("C-M-_","undo-redo"), ("C-x p p","project-switch-project")] {
        keys.editor.insert(key.into(), command.into());
    }
    // Editor overrides belong only to the nonmodal map. Each modal owner must
    // receive its own prospective additions to test the proposed state policy.
    for map in [&mut keys.vim_normal, &mut keys.vim_insert, &mut keys.vim_replace,
        &mut keys.vim_visual_character, &mut keys.vim_visual_line, &mut keys.vim_visual_block] {
        map.insert("M-m".into(),"modal-line-first-text".into());
        map.insert("C-x +".into(),"balance-windows".into());
        map.insert("C-x p p".into(),"project-switch-project".into());
    }
    keys.vim_insert.insert("C-M-_".into(),"undo-redo".into());
    keys.vim_replace.insert("C-M-_".into(),"undo-redo".into());
    for profile in [InteractionProfile::NonModal, InteractionProfile::Modal] {
        let maps = keymap::compile_for(&keys, profile).unwrap();
        for context in [Context::Editor, Context::VimNormal, Context::VimInsert,
            Context::VimReplace, Context::VimVisualCharacter, Context::VimVisualLine, Context::VimVisualBlock] {
            for (key, command) in [("M-m","modal-line-first-text"), ("C-x +","balance-windows"),
                ("C-x p p","project-switch-project")] {
                assert_eq!(lookup(&maps,context,key).as_deref(),Some(command));
            }
        }
        assert_eq!(lookup(&maps,Context::Editor,"C-r").as_deref(),Some("isearch-backward"));
        assert_eq!(lookup(&maps,Context::VimNormal,"C-r").as_deref(),Some("undo-redo"));
        assert_eq!(lookup(&maps,Context::VimNormal,"C-o").as_deref(),Some("modal-jump-backward"));
        assert_eq!(lookup(&maps,Context::VimInsert,"C-o").as_deref(),Some("modal-one-normal-command"));
        for context in [Context::Editor,Context::VimInsert,Context::VimReplace] {
            assert_eq!(lookup(&maps,context,"C-M-_").as_deref(),Some("undo-redo"));
        }
    }
    let mut separate = Keymaps::default();
    separate.editor.insert("M-m".into(),"forward-char".into());
    let maps = keymap::compile_for(&separate,InteractionProfile::Modal).unwrap();
    assert_eq!(lookup(&maps,Context::Editor,"M-m").as_deref(),Some("forward-char"));
    assert_eq!(lookup(&maps,Context::VimInsert,"M-m"),None);
    for (context, key) in [(Context::Editor,"C-g"),(Context::VimInsert,"M-x")] {
        let mut invalid = Keymaps::default();
        let map = if context == Context::Editor { &mut invalid.editor } else { &mut invalid.vim_insert };
        map.insert(key.into(),"undo".into());
        assert!(keymap::compile_for(&invalid,InteractionProfile::Modal).is_err());
    }
    assert_eq!(Sequence::parse("C-?").unwrap(),Sequence::parse("Backspace").unwrap());
    assert_eq!(Sequence::parse("C-i").unwrap(),Sequence::parse("TAB").unwrap());
    assert_ne!(Sequence::parse("C-M-_").unwrap(),Sequence::parse("Backspace").unwrap());
    json!({"profiles":2,"editing_maps":7,"protected_keys":["C-g","modal M-x"],
        "aliases":["C-? = Backspace","C-i = TAB"],"editor_override_isolated":true})
}
fn effects(root: &Path) -> Value {
    let mut s = session(root,"","prev\n  current\n  next",7);
    s.execute(named("modal-join-lines"));
    assert_eq!(text(&s),"prev\n  current next");
    let joined = json!({"text":text(&s),"point":s.view().point.position.0});
    s.execute(named("undo"));
    assert_eq!(text(&s),"prev\n  current\n  next");
    s.execute(named("undo-redo"));
    assert_eq!(text(&s),"prev\n  current next");
    let s = session(root,"","one\n \t\ntwo",0);
    let coordinates = Coordinates::new(s.buffer().unwrap()).unwrap();
    let paragraph = coordinates.navigation(Scalar(0),Target::Paragraph(rev::search::Direction::Forward),1).unwrap().0;
    assert_eq!(paragraph,4);
    let s = session(root,"","one\n%% boundary\ntwo",0);
    let paragraph_rule = Coordinates::new(s.buffer().unwrap()).unwrap()
        .navigation(Scalar(0),Target::Paragraph(rev::search::Direction::Forward),1).unwrap().0;
    assert_eq!(paragraph_rule,19);
    let s = session(root,"","(ab)",0);
    let pair = Coordinates::new(s.buffer().unwrap()).unwrap().navigation(Scalar(0),Target::Pair,1).unwrap().0;
    assert_eq!(pair,3);
    let mut s = session(root,"[windows]\nsplit_ratio=0.3","kept",0);
    s.execute(named("split-window-right"));
    s.execute(named("split-window-right"));
    let selected = s.windows().selected_id();
    let size = Size { width:100,height:32 };
    let widths_before: Vec<_> = s.windows().geometry(size).unwrap().panes().iter().map(|(_,a)|a.width()).collect();
    s.execute(named("balance-windows"));
    let after: Vec<_> = s.windows().geometry(size).unwrap().panes().iter().map(|(_,a)|a.width()).collect();
    assert_eq!(s.windows().selected_id(),selected);
    assert_ne!(widths_before,after);
    assert!(after.iter().max().unwrap()-after.iter().min().unwrap()<=1);
    s.execute(named("find-file"));
    let retained = s.windows().geometry(size).unwrap();
    s.execute(named("balance-windows"));
    assert_eq!(format!("{retained:?}"),format!("{:?}",s.windows().geometry(size).unwrap()));
    s.execute(named("keyboard-quit"));
    assert_eq!(text(&s),"kept");
    let mut project = session(root,"","unsaved",0);
    project.insert('x');
    let before = text(&project);
    project.execute(named("project-switch-project"));
    let project_message = project.echo().to_owned();
    project.execute(named("keyboard-quit"));
    assert_eq!(text(&project),before);
    json!({"join":joined,"join_undo_redo":true,"paragraph_offset":paragraph,"paragraph_rule_offset":paragraph_rule,
        "matching_pair_offset":pair,"pane_widths_before":widths_before,
        "pane_widths_after":after,"balance_prompt_rejected":true,
        "project_without_storage":project_message,"project_source_retained":true})
}
fn main() {
    let root = std::env::args().nth(1).unwrap();
    let commands: Vec<_> = Command::ALL.into_iter().map(|c| json!({"name":c.name(),
        "variant":format!("{c:?}"),"canonical":rev::command::Binding::named(c.name()).unwrap().name(),"description":c.description()})).collect();
    let bindings: Vec<_> = Context::ALL.into_iter().flat_map(|c| c.defaults().iter().map(move |(k,n)|
        json!({"context":c.name(),"key":k,"command":n}))).collect();
    let actions: Vec<_> = rev::command::PromptAction::ALL.into_iter().map(|a|json!(a.name())).collect();
    let first = first_text(Path::new(&root));
    eprintln!("First-text cases finished");
    let maps = mappings();
    eprintln!("Map cases finished");
    let effects = effects(Path::new(&root));
    eprintln!("Effect cases finished");
    println!("{}",json!({"commands":commands,"bindings":bindings,"actions":actions,
        "probes":{"first_text":first,"mappings":maps,"effects":effects}}));
}
'''


REFERENCE = r'''
(require 'json)
(require 'org)
(require 'kmacro)
(require 'edmacro)
(require 'bookmark)
(require 'project)
(require 'rect)
(require 'windmove)
(defun audit-command (name)
  (let* ((symbol (intern name)) (present (commandp symbol)))
    `((name . ,name) (interactive . ,(if present t :json-false))
      (bindings . ,(vconcat (when present (mapcar #'key-description
        (where-is-internal symbol (list (current-global-map)))))))
      (summary . ,(if present (car (split-string (or (documentation symbol t) "") "\n")) "")))))
(defun audit-motion (source offset command)
  (with-temp-buffer
    (text-mode) (insert source) (goto-char (1+ offset))
    (call-interactively command)
    `((text . ,(buffer-string)) (point . ,(1- (point))))))
(let* ((input (json-read-file (getenv "REV_AUDIT_CATALOG")))
       (names (append (mapcar (lambda (c) (cdr (assq 'name c))) (cdr (assq 'commands input)))
         '("back-to-indentation" "delete-indentation" "forward-paragraph" "backward-paragraph"
           "forward-sexp" "backward-sexp" "indent-region" "repeat" "open-line"
           "bookmark-bmenu-list" "complete-symbol" "mark-whole-buffer" "mark-paragraph")))
       (first (mapcar (lambda (case)
         `((case . ,(car case)) (offset . ,(cdr (assq 'point (audit-motion (cadr case) 0 'back-to-indentation))))))
         '(("spaces" "    x") ("tab" "\tx") ("mixed" " \t x") ("blank" " \t ")
           ("empty" "") ("unicode-space" "\u00a0x") ("wide" "  界") ("combining" "  e\u0301"))))
       (menu (when (boundp 'kmacro-menu-mode-map) (symbol-value 'kmacro-menu-mode-map))))
  (princ (json-encode `((emacs_version . ,emacs-version) (org_version . ,(org-version))
    (commands . ,(vconcat (mapcar #'audit-command names)))
    (first_text . ,(vconcat first))
    (join . ,(audit-motion "prev\n  current\n  next" 7 'delete-indentation))
    (paragraph . ,(audit-motion "one\n \t\ntwo" 0 'forward-paragraph))
    (paragraph_rule . ,(with-temp-buffer (text-mode) (insert "one\n%% boundary\ntwo")
      (goto-char 1) (let ((paragraph-start "%%") (paragraph-separate "%%"))
        (forward-paragraph) (1- (point)))))
    (sexp . ,(audit-motion "(ab)" 0 'forward-sexp))
    (macro_menu . ,(vconcat (mapcar (lambda (key) `((key . ,key)
      (command . ,(format "%s" (when menu (lookup-key menu (kbd key)))))))
      '("e" "d" "x" "m" "u" "q" "g" "?"))))))))
'''


def run(arguments, **kwargs):
    result = subprocess.run(arguments, text=True, capture_output=True, **kwargs)
    if result.returncode:
        raise RuntimeError(result.stderr or result.stdout)
    return result.stdout


def compile_native(root, dependencies, temporary, probe_source=NATIVE):
    manifest = tomllib.loads((root / "Cargo.toml").read_text())
    external = []
    for dependency in manifest["dependencies"]:
        name = dependency.replace("-", "_")
        candidates = list(dependencies.glob(f"lib{name}-*.rlib"))
        if not candidates:
            raise RuntimeError(f"Missing prebuilt dependency: {dependency}")
        artifact = max(candidates, key=lambda p: p.stat().st_mtime_ns)
        external.extend(["--extern", f"{name}={artifact}"])
    common = ["rustc", "--edition=2024", "-C", "debuginfo=0", "-L", f"dependency={dependencies}"]
    build = root / "target" / "factory-shortcut-audit"
    build.mkdir(parents=True, exist_ok=True)
    library = build / "librev_audit.rlib"
    identity = hashlib.sha256()
    for path in sorted((root / "src").rglob("*")):
        if path.is_file():
            identity.update(path.relative_to(root).as_posix().encode())
            identity.update(path.read_bytes())
    identity.update(str(external).encode())
    for flag in external[1::2]:
        artifact = Path(flag.split("=", 1)[1])
        identity.update(hashlib.sha256(artifact.read_bytes()).digest())
    identity.update(run(["rustc", "--version"]).encode())
    marker = build / "source.sha256"
    environment = dict(os.environ, CARGO_MANIFEST_DIR=str(root))
    if not library.is_file() or not marker.is_file() or marker.read_text() != identity.hexdigest():
        run(common + ["--crate-name", "rev", "--crate-type=rlib", str(root / "src/lib.rs"),
                      "-o", str(library)] + external, env=environment, timeout=180)
        marker.write_text(identity.hexdigest())
    source = temporary / "probe.rs"
    source.write_text(probe_source)
    executable = temporary / "probe"
    run(common + [str(source), "--extern", f"rev={library}", "-o", str(executable)] + external,
        timeout=180)
    return json.loads(run([str(executable), str(temporary)], timeout=120))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dependency-dir", type=Path, required=True)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    root = Path(run(["git", "rev-parse", "--show-toplevel"]).strip())
    run(["git", "diff", "--quiet", "HEAD", "--", "src", "Cargo.toml", "Cargo.lock", "crates", "vendor"])
    directory = Path(__file__).resolve().parent
    with tempfile.TemporaryDirectory(prefix="rev-shortcut-audit-") as name:
        temporary = Path(name)
        native = compile_native(root, args.dependency_dir.resolve(), temporary)
        catalog = {key: native[key] for key in ["commands", "bindings", "actions"]}
        catalog["source_tree"] = run(["git", "rev-parse", "HEAD:src"]).strip()
        catalog["schema_version"] = 1
        catalog["compiler"] = run(["rustc", "--version"]).strip()
        catalog["dependency_lock_sha256"] = hashlib.sha256((root / "Cargo.lock").read_bytes()).hexdigest()
        input_path = temporary / "catalog.json"
        input_path.write_text(json.dumps(catalog))
        reference_path = temporary / "reference.el"
        reference_path.write_text(REFERENCE)
        reference = json.loads(run(["emacs", "-Q", "--batch", "--load", str(reference_path)],
                                  env=dict(os.environ, REV_AUDIT_CATALOG=str(input_path)), timeout=30))
        assert native["probes"]["first_text"]["cases"] == reference["first_text"]
        assert reference["join"]["text"] != native["probes"]["effects"]["join"]["text"]
        assert reference["sexp"]["point"] != native["probes"]["effects"]["matching_pair_offset"]
        assert reference["paragraph_rule"] == 4
        assert reference["paragraph_rule"] != native["probes"]["effects"]["paragraph_rule_offset"]
        results = {"catalog.json": catalog, "reference-core.json": reference,
                   "probe-results.json": native["probes"]}
        for filename, value in results.items():
            path = directory / filename
            if args.write:
                path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")
            else:
                assert json.loads(path.read_text()) == value, f"Evidence drift: {filename}"
        print(f"Live audit passed: {len(catalog['commands'])} commands; "
              f"{len(catalog['bindings'])} entries; 28 contexts; 56 first-text sessions; "
              "profile, alias, join, history, pane and reference probes.")


if __name__ == "__main__":
    main()
