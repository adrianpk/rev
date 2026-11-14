#!/usr/bin/env python3
"""Compare stock reference effects for the requested functional follow-through."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

REFERENCE = r'''
;;; parity-reference.el -*- lexical-binding: t; -*-
(require 'json)
(require 'org)
(require 'org-src)
(require 'lisp-mode)
(defun parity-state ()
  `((text . ,(buffer-string)) (point . ,(1- (point)))
    (mark . ,(when (mark t) (1- (mark t))))
    (region_active . ,(if (use-region-p) t :json-false))
    (modified . ,(if (buffer-modified-p) t :json-false))
    (kill_ring . ,(vconcat kill-ring))))
(defun parity-keys (keys)
  ;; A quit ends its keyboard macro. Resume the next human input burst afterward.
  (let ((parts (split-string keys "C-g" nil)))
    (while (cdr parts)
      (condition-case nil
          (execute-kbd-macro (kbd (concat (car parts) "C-g")))
        (quit nil))
      (setq parts (cdr parts)))
    (unless (string-empty-p (string-trim (car parts)))
      (execute-kbd-macro (kbd (car parts))))))
(defun parity-case (name mode source point keys)
  (save-window-excursion
    (with-temp-buffer
      (funcall mode) (insert source) (goto-char (1+ point))
      (buffer-enable-undo) (setq buffer-undo-list nil)
      (set-buffer-modified-p nil)
      (set-window-buffer (selected-window) (current-buffer))
      (let ((kill-ring nil) (kill-ring-yank-pointer nil)
            (last-command nil) (transient-mark-mode t)
            (inhibit-message t) (ring-bell-function #'ignore))
        (parity-keys keys)
        `((case . ,name) (mode . ,(symbol-name mode)) (keys . ,keys)
          (initial_text . ,source) (initial_point . ,point)
          (prompt_depth . ,(minibuffer-depth)) (result . ,(parity-state)))))))
(defun parity-library (name)
  (let* ((compiled (locate-library name))
         (source (concat (file-name-sans-extension compiled) ".el.gz")))
    (unless (file-exists-p source) (error "Reference source unavailable: %s" name))
    `((library . ,name) (source_file . ,(file-name-nondirectory source))
      (sha256 . ,(with-temp-buffer
        (insert-file-contents-literally source) (secure-hash 'sha256 (current-buffer)))))))
(defun parity-source-save ()
  (let* ((path (expand-file-name "source.org" (getenv "REV_PARITY_FIXTURE")))
         (original "#+begin_src text\nkept\n#+end_src\n")
         (make-backup-files nil) (auto-save-default nil) parent draft result)
    (with-temp-file path (insert original))
    (unwind-protect
        (save-window-excursion
          (setq parent (find-file-noselect path))
          (switch-to-buffer parent) (org-mode)
          (goto-char (point-min)) (search-forward "kept")
          (org-edit-special) (setq draft (current-buffer))
          (goto-char (point-min)) (insert "x") (org-edit-src-save)
          (setq result
            `((draft_retained . ,(if (eq (current-buffer) draft) t :json-false))
              (parent_text . ,(with-current-buffer parent (buffer-string)))
              (parent_modified . ,(if (buffer-modified-p parent) t :json-false))
              (disk_text . ,(with-temp-buffer (insert-file-contents path) (buffer-string)))))
          (org-edit-src-exit)
          result)
      (dolist (buffer (list draft parent))
        (when (buffer-live-p buffer)
          (with-current-buffer buffer (set-buffer-modified-p nil)) (kill-buffer buffer))))))
(with-temp-file (getenv "REV_PARITY_OUTPUT") (insert (json-encode
  `((emacs_version . ,emacs-version) (org_version . ,(org-version))
    (libraries . ,(vconcat (mapcar #'parity-library '("simple" "paragraphs" "lisp" "org-src"))))
    (cases . ,(vector
      (parity-case "whole_buffer" 'text-mode "one\ntwo\n" 2 "C-x h")
      (parity-case "paragraph_mark" 'text-mode "one two\nline\n\nnext\n" 4 "M-h")
      (parity-case "open_line" 'text-mode "abcd" 2 "C-o")
      (parity-case "open_line_count" 'text-mode "abcd" 2 "C-u 2 C-o")
      (parity-case "join_previous" 'text-mode "prev\n  current\n  next" 7 "M-^")
      (parity-case "join_following_prefix" 'text-mode "prev\n  current\n  next" 7 "C-u M-^")
      (parity-case "paragraph_forward" 'text-mode "one\n\ntwo\n" 0 "M-}")
      (parity-case "paragraph_backward" 'text-mode "one\n\ntwo\n" 7 "M-{")
      (parity-case "sexp_forward" 'emacs-lisp-mode "(ab) tail" 0 "C-M-f")
      (parity-case "sexp_backward" 'emacs-lisp-mode "(ab) tail" 4 "C-M-b")
      (parity-case "region_indent" 'emacs-lisp-mode "(progn\n(foo)\n(bar))\n" 0 "C-x h C-M-\\")
      (parity-case "command_repeat" 'text-mode "abcd" 0 "C-d C-x z")
      (parity-case "copy_yank_workflow" 'text-mode "alpha beta" 0 "C-SPC M-f M-w C-g C-e C-y")
      (parity-case "kill_yank_workflow" 'text-mode "alpha beta" 0 "C-SPC M-f C-w C-e C-y")
      (parity-case "counted_motion" 'text-mode "abcd" 0 "C-u 2 C-f")
      (parity-case "named_motion" 'text-mode "abcd" 0 "M-x forward-char RET")
      (parity-case "named_cancel" 'text-mode "abcd" 0 "M-x delete-char C-g")))
    (source_edit_save . ,(parity-source-save))))))
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    directory = Path(__file__).resolve().parent
    with tempfile.TemporaryDirectory(prefix='rev-parity-reference-') as fixture:
        script = Path(fixture) / 'reference.el'
        script.write_text(REFERENCE.lstrip())
        process = subprocess.run(['emacs', '-Q', '--batch', '--load', str(script)],
            env=dict(os.environ, REV_PARITY_FIXTURE=fixture,
                     REV_PARITY_OUTPUT=str(Path(fixture) / "result.json")), text=True,
            capture_output=True, timeout=60)
        if process.returncode:
            raise RuntimeError(process.stderr)
        result = json.loads((Path(fixture) / "result.json").read_text())
    cases = {case['case']: case for case in result['cases']}
    assert len(cases) == 17
    assert cases['whole_buffer']['result']['region_active']
    assert cases['open_line']['result']['text'] == 'ab\ncd'
    assert cases['open_line']['result']['point'] == 2
    assert cases['sexp_forward']['result']['point'] == 4
    assert cases['command_repeat']['result']['text'] == 'cd'
    save = result['source_edit_save']
    assert save['draft_retained'] and not save['parent_modified']
    assert save['parent_text'] == save['disk_text'] and 'xkept' in save['disk_text']
    output = directory / 'parity-reference.json'
    if args.write:
        output.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    else:
        assert json.loads(output.read_text()) == result, 'Functional reference evidence drift'
    print('Functional reference passed: 17 stock key workflows; retained source draft with saved parent/disk; source hashes.')


if __name__ == '__main__':
    main()
