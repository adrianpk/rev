#!/usr/bin/env python3
"""Reproduce isolated mode references and native proposal-admission probes."""

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile


REFERENCE = r'''
;;; reference.el -*- lexical-binding: t; -*-
(require 'json)
(require 'org)
(require 'org-agenda)
(require 'org-capture)
(require 'org-src)
(require 'org-table)
(require 'org-habit)
(require 'org-attach)
(require 'org-clock)
(require 'org-timer)
(require 'org-feed)
(require 'ob-tangle)
(dolist (backend '(ox-ascii ox-md ox-org ox-html ox-latex ox-beamer ox-odt ox-texinfo ox-man ox-icalendar))
  (require backend))
(require 'calendar)
(require 'markdown-mode)
(require 'magit)
(require 'magit-worktree)
(require 'git-commit)
(defvar audit-maps
  '((org . org-mode-map) (org_agenda . org-agenda-mode-map)
    (org_capture . org-capture-mode-map) (org_calendar . calendar-mode-map)
    (org_source_edit . org-src-mode-map) (markdown . markdown-mode-map)
    (git . magit-status-mode-map) (git_history . magit-log-mode-map)
    (git_inspect . magit-revision-mode-map) (git_refs . magit-refs-mode-map)
    (git_sequence . magit-process-mode-map) (git_worktree . magit-worktree-list-mode-map)
    (git_stash . magit-stash-mode-map) (git_conflict . magit-status-mode-map)
    (git_commit . git-commit-mode-map)))
(defun audit-name (name)
  (or (cdr (assoc name
    '(("org-insert-item-checkbox" . "org-insert-todo-heading")
      ("org-content" . "org-content")
      ("markdown-continue-item" . "markdown-insert-list-item")
      ("markdown-return" . "markdown-enter-key")
      ("markdown-meta-return" . "markdown-insert-list-item")
      ("org-export-to-text" . "org-ascii-export-to-ascii")
      ("org-export-to-unicode" . "org-ascii-export-to-ascii")
      ("org-export-to-markdown" . "org-md-export-to-markdown")
      ("org-export-to-org" . "org-org-export-to-org")
      ("org-export-to-html" . "org-html-export-to-html")
      ("org-export-to-latex" . "org-latex-export-to-latex")
      ("org-export-to-beamer" . "org-beamer-export-to-latex")
      ("org-export-to-latex-pdf" . "org-latex-export-to-pdf")
      ("org-export-to-beamer-pdf" . "org-beamer-export-to-pdf")
      ("org-export-to-odt" . "org-odt-export-to-odt")
      ("org-export-to-texinfo" . "org-texinfo-export-to-texinfo")
      ("org-export-to-man" . "org-man-export-to-man")
      ("org-export-to-icalendar" . "org-icalendar-export-to-ics")
      ("org-habit-toggle-habits" . "org-habit-toggle-display-in-agenda"))))
    (if (string-prefix-p "git-" name) (concat "magit-" (substring name 4))
      (if (string-prefix-p "org-calendar-" name)
          (concat "calendar-" (substring name 13)) name))))
(defun audit-keys (command)
  (vconcat (delq nil (mapcar
    (lambda (pair)
      (let* ((map (and (boundp (cdr pair)) (symbol-value (cdr pair))))
             (keys (and (keymapp map) (where-is-internal command (list map)))))
        (when keys `((context . ,(symbol-name (car pair)))
                     (keys . ,(vconcat (mapcar #'key-description keys))))))) audit-maps))))
(defun audit-spelling (key)
  (mapconcat (lambda (part)
    (or (cdr (assoc part '(("Up" . "<up>") ("Down" . "<down>")
      ("Left" . "<left>") ("Right" . "<right>") ("Backspace" . "<backspace>")
      ("BackTab" . "<backtab>") ("S-TAB" . "<backtab>")
      ("PageUp" . "<prior>") ("PageDown" . "<next>")
      ("Home" . "<home>") ("End" . "<end>") ("C-Enter" . "C-<return>")))) part))
    (split-string key " ") " "))
(defun audit-route (row)
  (let* ((context (cdr (assq 'context row)))
         (symbol (cdr (assq (intern context) audit-maps)))
         (map (and symbol (boundp symbol) (symbol-value symbol)))
         (key (cdr (assq 'key row)))
         (target (condition-case nil
                   (when (keymapp map) (lookup-key map (kbd (audit-spelling key))))
                   (error 'unparsed))))
    `((context . ,context) (key . ,key) (native . ,(cdr (assq 'command row)))
      (reference . ,(cond ((keymapp target) "PREFIX") ((symbolp target) (symbol-name target))
                         ((integerp target) "INCOMPLETE") (t "NON-SYMBOL"))))))
(defun audit-edit (mode source offset keys)
  (save-window-excursion (with-temp-buffer
    (funcall mode) (insert source) (goto-char (1+ offset))
    (set-window-buffer (selected-window) (current-buffer))
    (condition-case condition
      (progn (execute-kbd-macro (kbd (if (equal keys "M-S-RET") "M-S-<return>" keys)))
        `((text . ,(buffer-string)) (point . ,(1- (point))) (error . "")))
      (error `((text . ,(buffer-string)) (point . ,(1- (point)))
               (error . ,(error-message-string condition))))))))
(let* ((catalog (json-read-file (getenv "REV_MODE_CATALOG")))
       (commands (seq-filter (lambda (c)
         (string-match-p "\\`\\(?:org\\|git\\|markdown\\)-" (cdr (assq 'name c))))
         (append (cdr (assq 'commands catalog)) nil)))
       (bindings (seq-filter (lambda (b) (assq (intern (cdr (assq 'context b))) audit-maps))
         (append (cdr (assq 'bindings catalog)) nil))))
  (princ (json-encode
    `((emacs_version . ,emacs-version) (org_version . ,(org-version))
      (commands . ,(vconcat (mapcar (lambda (c)
        (let* ((name (cdr (assq 'name c))) (reference (audit-name name)) (symbol (intern reference))
               (available (commandp symbol)))
          `((name . ,name) (reference . ,reference) (interactive . ,(if available t :json-false))
            (summary . ,(if available (car (split-string (or (documentation symbol t) "") "\n")) ""))
            (bindings . ,(if available (audit-keys symbol) []))))) commands)))
      (bindings . ,(vconcat (mapcar #'audit-route bindings)))
      (effects . ((org_checkbox_list . ,(audit-edit 'org-mode "- item\n" 6 "M-S-RET"))
        (org_checkbox_heading . ,(audit-edit 'org-mode "* Heading\n" 9 "M-S-RET"))
        (org_meta_list . ,(audit-edit 'org-mode "- item\n" 6 "M-RET"))
        (markdown_return . ,(audit-edit 'markdown-mode "- item\n" 6 "RET"))
        (markdown_meta_return . ,(audit-edit 'markdown-mode "- item\n" 6 "M-RET"))))
      (agenda_keys . ,(vconcat (mapcar (lambda (key)
        `((key . ,key) (command . ,(format "%s" (lookup-key org-agenda-mode-map (kbd key))))))
        '("d" "M" "s" "A" "G" "S" "K" "V" "v"))))))))
'''


NATIVE = r'''
#![forbid(unsafe_code)]
use rev::buffer::{Buffers, Mode};
use rev::command::{Command, Scroll};
use rev::config::{InteractionProfile, Keymaps, Snapshot, prepare};
use rev::coordinates::Scalar;
use rev::input::keymap::{self, Context, Sequence};
use rev::session::Session;
use rev::view::{Size, View};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use serde_json::{Value,json};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration,Instant};

fn named(name: &str) -> Command { Command::named(name).unwrap() }
fn text(s: &Session) -> String { let b=s.buffer().unwrap(); b.slice(&b.span(0..b.len_chars()).unwrap()).unwrap().to_string() }
fn snapshot(root: &Path, config: &str) -> Snapshot {
    let path=root.join("config.toml"); std::fs::write(&path,config).unwrap();
    let prepared=prepare(&path); assert!(prepared.diagnostic.is_none(),"{:?}",prepared.diagnostic);
    prepared.snapshot
}
fn session(settings: &Snapshot, source: &str, mode: Mode, point: usize) -> Session {
    let mut buffers=Buffers::with_settings(settings.clone()).unwrap();
    let id=buffers.create(source).unwrap(); buffers.get_mut(id).unwrap().select_mode(mode).unwrap();
    let mut view=View::with_settings(buffers.get(id).unwrap(),settings); view.point.position=Scalar(point);
    let mut s=Session::new(buffers,view,Size{width:100,height:24}).unwrap(); settle(&mut s); s
}
fn settle(s: &mut Session) {
    let deadline=Instant::now()+Duration::from_secs(5);
    loop {
        s.frame().unwrap();
        if s.remaining_input(Instant::now()).is_none() { return; }
        assert!(Instant::now()<deadline,"Owned mode probe failed to settle: {}",s.echo());
        s.expire_input(Instant::now()+Duration::from_millis(10)); std::thread::yield_now();
    }
}
fn action(s: &mut Session,name: &str) { s.execute(named(name)); settle(s); }
fn argument(s: &mut Session,value: &str) {
    s.handle(Event::Paste(value.into())); s.handle(Event::Key(KeyEvent::new(KeyCode::Enter,KeyModifiers::NONE))); settle(s);
}
fn keys(s: &mut Session,value: &str) {
    for key in Sequence::parse(value).unwrap().chords() {
        s.handle(Event::Key(KeyEvent::new(key.code,key.modifiers)));
    }
    settle(s);
}
fn proposals(root: &Path) -> Value {
    let proposal:Value=serde_json::from_str(&std::fs::read_to_string(root.join("mapping-proposal.json")).unwrap()).unwrap();
    let changes=proposal["changes"].as_array().unwrap(); let mut overrides=serde_json::Map::new();
    let factory=keymap::compile(&Keymaps::default()).unwrap();
    for change in changes {
        for context in change["contexts"].as_array().unwrap() {
            let name=context.as_str().unwrap(); let c=Context::ALL.into_iter().find(|c|c.name()==name).unwrap();
            let key=change["key"].as_str().unwrap();
            assert_eq!(Sequence::parse(key).unwrap().to_string(),change["normalized"].as_str().unwrap());
            let before=factory.map(c).lookup(&Sequence::parse(key).unwrap()).map(|b|b.name().to_owned());
            assert_eq!(before,change["before"][name].as_str().map(str::to_owned));
            if let Some(target)=change["after"].as_str() {
                let map=overrides.entry(name).or_insert_with(||json!({})).as_object_mut().unwrap();
                assert!(map.insert(key.into(),json!(target)).is_none());
            }
        }
    }
    let values=Value::Object(overrides);
    let keymaps:Keymaps=serde_json::from_value(values.clone()).unwrap(); let mut traces=Vec::new();
    for profile in [InteractionProfile::NonModal,InteractionProfile::Modal] {
        let compiled=keymap::compile_for(&keymaps,profile).unwrap();
        for context in Context::ALL {
            let mut effective:BTreeMap<Sequence,String>=compiled.map(context).bindings()
                .map(|(k,b)|(k.clone(),b.name().to_owned())).collect();
            for change in changes {
                if !change["contexts"].as_array().unwrap().iter().any(|c|c.as_str()==Some(context.name())) { continue; }
                let sequence=Sequence::parse(change["key"].as_str().unwrap()).unwrap();
                if change["after"].is_null() { assert!(effective.remove(&sequence).is_some()); }
                let actual=effective.get(&sequence).map(String::as_str);
                assert_eq!(actual,change["after"].as_str());
                traces.push(json!({"profile":format!("{profile:?}"),"context":context.name(),
                    "proposal":change["id"],"normalized":sequence.to_string(),"target":actual}));
            }
            let entries:Vec<_>=effective.iter().collect();
            for pair in entries.windows(2) {
                assert!(!pair[1].0.chords().starts_with(pair[0].0.chords()) || pair[0].1=="prefix-argument",
                    "Proposal creates a prefix conflict in {}",context.name());
            }
            assert_eq!(effective.get(&Sequence::parse("C-g").unwrap()).map(String::as_str),Some("keyboard-quit"));
        }
    }
    let mut active_languages=0;
    for profile in ["non-modal","modal"] {
        let mut config=format!("editor.interaction_profile='{profile}'\n");
        for (context,map) in values.as_object().unwrap() {
            config.push_str(&format!("[keys.{context}]\n"));
            for (key,value) in map.as_object().unwrap() {
                config.push_str(&format!("{}={}\n",serde_json::to_string(key).unwrap(),value));
            }
        }
        let settings=snapshot(root,&config);
        let states:Vec<_>=if profile=="modal" {vec![Some("modal-enter-normal"),Some("modal-enter-insert"),
            Some("modal-enter-replace"),Some("modal-enter-visual-character"),Some("modal-enter-visual-line"),Some("modal-enter-visual-block")]} else {vec![None]};
        for mode in [Mode::Text,Mode::Org,Mode::Markdown] {
            for state in &states {
                let mut s=session(&settings,"  item\n",mode,0);
                if let Some(state)=state { action(&mut s,state); }
                let prior=s.editing_state(); keys(&mut s,"M-m");
                assert_eq!(s.view().point.position,Scalar(2));assert_eq!(s.editing_state(),prior);
                assert_eq!(text(&s),"  item\n");active_languages+=1;
            }
        }
    }
    json!({"profiles":2,"base_context_edits":46,"active_language_sessions":active_languages,"normalized_traces":traces,
        "removal_method":"Exact normalized leaf removal from compiled-map snapshots; no configuration unbinding sentinel.",
        "scope":"Admission and normalized prefix checks, not shipped factories or physical terminal effects."})
}
fn org_effects(settings:&Snapshot) -> Value {
    let source="* Task\n:PROPERTIES:\n:EFFORT_ALL: 15min 30min\n:EFFORT: 15min\n:END:\n";
    let mut s=session(settings,source,Mode::Org,0); action(&mut s,"org-inc-effort");
    assert!(text(&s).contains(":EFFORT: 30min")); assert_eq!(s.buffer().unwrap().history().len(),1);
    let mut list=session(settings,"- item\n",Mode::Org,6); action(&mut list,"org-insert-item-checkbox");
    assert!(text(&list).contains("- [ ]")); let checkbox_list=text(&list);
    let mut heading=session(settings,"* Heading\n",Mode::Org,9); action(&mut heading,"org-insert-item-checkbox");
    assert_eq!(text(&heading),"* Heading\n"); let heading_error=heading.echo().to_owned();
    let column_source="#+COLUMNS: %ITEM %EFFORT\n* Task\n:PROPERTIES:\n:EFFORT: 30min\n:END:\n";
    let mut columns=session(settings,column_source,Mode::Org,0); let stamp=columns.buffer().unwrap().stamp();
    action(&mut columns,"org-columns"); assert!(columns.query_agenda().is_some(),"{}",columns.echo());
    assert_eq!(columns.buffer().unwrap().stamp(),stamp); assert!(columns.buffer().unwrap().history().is_empty());
    assert_eq!(text(&columns),column_source);
    json!({"effort_cycled":true,"effort_one_group":true,"checkbox_list":checkbox_list,
        "checkbox_heading_unchanged":true,"checkbox_heading_message":heading_error,"columns_source_unchanged":true})
}
fn source_effects(settings:&Snapshot) -> Value {
    let source="#+BEGIN_SRC text\nkept\n#+END_SRC\n";
    let mut s=session(settings,source,Mode::Org,17); action(&mut s,"org-babel-insert-header-arg");
    assert!(s.echo().contains("header argument"),"{}",s.echo());
    action(&mut s,"keyboard-quit"); assert_eq!(text(&s),source);
    action(&mut s,"org-babel-insert-header-arg"); argument(&mut s,":var x=1");
    assert!(text(&s).contains("#+BEGIN_SRC text :var x=1"),"{}",text(&s));
    let mut draft=session(settings,source,Mode::Org,17); action(&mut draft,"org-edit-special");
    assert!(draft.source_edit_draft().is_some(),"{}",draft.echo()); draft.insert('x');
    let retained=text(&draft); action(&mut draft,"save-buffer");
    assert!(draft.source_edit_draft().is_some()); assert_eq!(text(&draft),retained);
    let save_message=draft.echo().to_owned(); assert!(save_message.contains("Source draft retained"));
    action(&mut draft,"org-edit-src-abort"); assert_eq!(text(&draft),source);
    json!({"header_cancel_unchanged":true,"header_inserted":true,"source_save_message":save_message,
        "source_save_retains_draft":true,"source_abort_unchanged":true})
}
fn markdown_effects(settings:&Snapshot) -> Value {
    let mut results=Vec::new();
    for (kind,source,point,key) in [("plain","plain",5,"RET"),("list","- item\n",6,"RET"),
        ("list-meta","- item\n",6,"M-RET"),("table","| A | B |\n| --- | --- |\n| c | d |\n",2,"RET")] {
        let mut s=session(settings,source,Mode::Markdown,point); keys(&mut s,key);
        if kind=="list" || kind=="list-meta" { assert!(text(&s).contains("\n- ")); }
        if kind=="plain" { assert_eq!(text(&s),"plain\n"); }
        if kind=="table" {
            let mut helper=session(settings,source,Mode::Markdown,point); action(&mut helper,"markdown-table-next-row");
            assert_eq!(text(&s),text(&helper)); assert_eq!(s.view().point.position,helper.view().point.position);
            assert!(s.view().point.position.0>point);
        }
        results.push(json!({"context":kind,"key":key,"text":text(&s),"point":s.view().point.position.0}));
    }
    json!(results)
}
fn clock(_: &rev::config::Time) -> Result<rev::org::time::ZonedOperation,String> {
    use rev::org::time::{ZonedOperation,ZoneSelection,ZoneEnvironment};
    ZonedOperation::capture(&ZoneSelection::Utc,&ZoneEnvironment{localtime:"unused".into(),zoneinfo:"unused".into(),tz:None},
        ||Ok(std::time::UNIX_EPOCH+Duration::from_secs(1709303400)))
}
fn agenda_effects(root:&Path) -> Value {
    let settings=snapshot(root,"org.time.zone='UTC'\n[keys.org_agenda]\nd='org-agenda-day-view'");
    let source="* TODO First\n<2024-03-01 Fri>\n* TODO Second\n<2024-03-01 Fri>\n";
    let mut s=session(&settings,source,Mode::Org,0).with_org_clock(clock);
    action(&mut s,"org-agenda"); argument(&mut s,"current"); keys(&mut s,"d");
    action(&mut s,"org-agenda-goto-date"); argument(&mut s,"2024-03-01");
    assert_eq!(s.calendar_agenda().unwrap_or_else(||panic!("Agenda probe: {}",s.echo())).query.span,rev::config::AgendaSpan::Day);
    assert_eq!(text(&s),source);
    let calendar=s.calendar_agenda().unwrap(); let count=calendar.lines.len();
    let selected=calendar.lines.iter().position(|line|matches!(line,rev::org::agenda::CalendarLine::Entry(i)
        if calendar.rows[*i].title.contains("Second"))).unwrap_or_else(||panic!("Missing fixture row: {:?}",calendar.rows));
    for _ in 0..count { s.execute(Command::Scroll(Scroll::PreviousLine)); }
    for _ in 0..selected { s.execute(Command::Scroll(Scroll::NextLine)); }
    action(&mut s,"org-set-effort"); argument(&mut s,"30min");
    let changed=text(&s); let second=changed.find("* TODO Second").unwrap();
    assert!(!changed[..second].contains("EFFORT")); assert!(changed[second..].contains(":EFFORT: 30min"));
    json!({"day_key_view_only":true,"selected_action_target":"Second","first_entry_unchanged":true,
        "selected_effort_changed":true,"source_actions":s.agenda_action_results().len()})
}
fn calendar_capture_effects(root:&Path,settings:&Snapshot) -> Value {
    let source="* Task\n";
    let mut s=session(settings,source,Mode::Org,0).with_org_clock(clock);
    action(&mut s,"org-timestamp"); keys(&mut s,"S-Right"); keys(&mut s,"RET");
    assert!(text(&s).contains("<2024-03-02 Sat"),"{}",text(&s));
    let mut cancelled=session(settings,source,Mode::Org,0).with_org_clock(clock);
    action(&mut cancelled,"org-timestamp"); keys(&mut cancelled,"S-Right"); keys(&mut cancelled,"C-g");
    assert_eq!(text(&cancelled),source);assert!(cancelled.buffer().unwrap().history().is_empty());
    let destination=root.join("unwritten-capture.org");
    let config=format!("org.capture.default_file={}",serde_json::to_string(&destination).unwrap());
    let capture_settings=snapshot(root,&config);
    let mut capture=session(&capture_settings,"origin",Mode::Text,0).with_org_clock(clock);
    action(&mut capture,"org-capture"); argument(&mut capture,"");
    assert!(capture.capture_draft().is_some(),"{}",capture.echo());capture.insert('x');
    let draft=text(&capture); action(&mut capture,"save-buffer");
    assert!(capture.capture_draft().is_some());assert_eq!(text(&capture),draft);
    keys(&mut capture,"C-c C-k");assert!(capture.capture_draft().is_none());assert_eq!(text(&capture),"origin");
    assert!(!destination.exists());
    json!({"calendar_shifted_selection_committed":true,"calendar_cancel_source_unchanged":true,
        "capture_save_retains_draft":true,"capture_abort_restores_source":true,"capture_destination_created":false})
}
fn git_effects(root:&Path,settings:&Snapshot) -> Value {
    let mut buffers=Buffers::with_settings(settings.clone()).unwrap();
    let id=buffers.visit(&root.join("repository/note.txt")).unwrap(); let view=View::new(buffers.get(id).unwrap());
    let mut s=Session::new(buffers,view,Size{width:100,height:24}).unwrap(); settle(&mut s);
    action(&mut s,"git-status"); assert!(s.git_visible()); assert!(s.git_observation().is_some(),"{}",s.echo());
    action(&mut s,"git-log"); action(&mut s,"git-fetch"); let fetch=s.echo().to_owned();
    assert!(fetch.contains("Refs")); action(&mut s,"git-push"); let push=s.echo().to_owned(); assert!(push.contains("Refs"));
    action(&mut s,"git-refs"); action(&mut s,"git-fetch"); let remote=s.echo().to_owned(); assert!(remote.contains("remote"));
    action(&mut s,"keyboard-quit");
    assert_eq!(text(&s),"kept\n");
    json!({"history_fetch_refused":fetch,"history_push_refused":push,"refs_without_remote_refused":remote,
        "source_unchanged":text(&s)=="kept\n","transport_started":false})
}
fn main() {
    let root=std::env::args().nth(1).unwrap();let root=Path::new(&root);
    let settings=snapshot(root,"org.time.zone='UTC'");
    println!("{}",json!({"proposal":proposals(root),"org":org_effects(&settings),
        "source_edit":source_effects(&settings),"markdown":markdown_effects(&settings),
        "agenda":agenda_effects(root),"calendar_capture":calendar_capture_effects(root,&settings),"git":git_effects(root,&settings)}));
}
'''


def package_evidence(packages, name, filename):
    source = (packages / name / filename).resolve()
    text = source.read_text()
    version = re.search(r"(?m)^;; (?:Package-)?Version:\s*(\S+)", text).group(1)
    result = {"version": version, "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest()}
    result["revision"] = subprocess.run(
        ["git", "-C", str(source.parent), "rev-parse", "HEAD"],
        check=True, capture_output=True, text=True).stdout.strip()
    return result


def check_proposal_documents(root, directory):
    """Check complete proposal structure without claiming mapping approval."""
    paths = [root / "ops/default/spec/editor-factory-shortcut-alignment.md",
             root / "ops/default/plan/rev-factory-shortcuts.md",
             root / "ops/default/tracker/rev-factory-shortcuts.md",
             root / "ops/default/report/slices/rev-factory-shortcuts/slice-2-mode-contract.md",
             directory / "semantic-matrix.md", directory / "mode-review.md",
             directory / "mapping-contract.md"]
    for path in paths:
        for target in re.findall(r"\[[^\]]+\]\(([^)]+)\)", path.read_text()):
            if "://" not in target:
                assert (path.parent / target.split("#")[0]).is_file(), (path.name, target)
    catalog = json.loads((directory / "catalog.json").read_text())
    names = {entry["name"] for entry in catalog["commands"]}
    matrix = (directory / "semantic-matrix.md").read_text()
    rows = re.findall(r"(?m)^\|\s*\x60([^\x60]+)\x60\s*\|", matrix)
    assert len(names) == 707 and names <= set(rows)
    assert all(rows.count(name) == 1 for name in names), "Duplicate command row"
    assert not re.search(r"\|\s*unresolved(?:\s|/)", matrix, re.I)
    contexts = {entry["context"] for entry in catalog["bindings"]}
    assert len(contexts) == 28 and all(context in matrix for context in contexts)
    proposal_path = directory / "mapping-proposal.json"
    proposal = json.loads(proposal_path.read_text())
    assert proposal["source_tree"] == catalog["source_tree"]
    assert len(proposal["changes"]) == 26
    assert sum(len(change["contexts"]) for change in proposal["changes"]) == 46
    assert all(change["after"] is None or change["after"] in names
               for change in proposal["changes"])
    digest = hashlib.sha256(proposal_path.read_bytes()).hexdigest()
    assert digest in (directory / "mapping-contract.md").read_text()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package-dir", type=Path, required=True)
    parser.add_argument("--dependency-dir", type=Path)
    parser.add_argument("--reference-only", action="store_true")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    directory = Path(__file__).resolve().parent
    root = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"],
                              check=True, capture_output=True, text=True).stdout.strip())
    specification = importlib.util.spec_from_file_location("core_audit", directory / "audit.py")
    core = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(core)
    with tempfile.TemporaryDirectory(prefix="rev-mode-audit-") as name:
        temporary = Path(name)
        reference_source = temporary / "reference.el"
        reference_source.write_text(REFERENCE)
        command = ["emacs", "-Q", "--batch"]
        for package in sorted(args.package_dir.resolve().iterdir()):
            if package.is_dir():
                command.extend(["-L", str(package)])
        command.extend(["--load", str(reference_source)])
        reference = json.loads(core.run(command, env=dict(os.environ,
            REV_MODE_CATALOG=str(directory / "catalog.json")), timeout=60))
        reference["packages"] = {
            "markdown-mode": package_evidence(args.package_dir, "markdown-mode", "markdown-mode.el"),
            "magit": package_evidence(args.package_dir, "magit", "magit.el"),
        }
        results = {"reference-modes.json": reference}
        if not args.reference_only:
            if args.dependency_dir is None:
                parser.error("--dependency-dir is required for native probes")
            (temporary / "mapping-proposal.json").write_bytes((directory / "mapping-proposal.json").read_bytes())
            repository = temporary / "repository"
            repository.mkdir()
            os.environ.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
            for command in [["init", "--initial-branch=main"], ["config", "user.name", "Example Author"],
                            ["config", "user.email", "author@example.invalid"],
                            ["config", "commit.gpgsign", "false"], ["config", "core.hooksPath", os.devnull]]:
                core.run(["git", "-C", str(repository)] + command)
            (repository / "note.txt").write_text("kept\n")
            core.run(["git", "-C", str(repository), "add", "note.txt"])
            core.run(["git", "-C", str(repository), "commit", "-m", "Fixture"])
            before = [core.run(["git", "-C", str(repository)] + command)
                      for command in [["rev-parse", "HEAD"], ["status", "--porcelain"], ["ls-files", "--stage"]]]
            results["mode-probe-results.json"] = core.compile_native(
                root, args.dependency_dir.resolve(), temporary, NATIVE)
            after = [core.run(["git", "-C", str(repository)] + command)
                     for command in [["rev-parse", "HEAD"], ["status", "--porcelain"], ["ls-files", "--stage"]]]
            assert before == after, "Owned Git probe changed HEAD, saved state or index"
            results["mode-probe-results.json"]["git"]["head_saved_state_index_unchanged"] = True
        check_proposal_documents(root, directory)
        for filename, value in results.items():
            path = directory / filename
            if args.write:
                path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")
            else:
                assert json.loads(path.read_text()) == value, f"Evidence drift: {filename}"
        print(f"Mode reference passed: {len(reference['commands'])} operations; "
              f"{len(reference['bindings'])} contextual routes; isolated package versions/revisions.")


if __name__ == "__main__":
    main()
