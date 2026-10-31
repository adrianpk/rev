#!/usr/bin/env python3
"""Exercise actual CLI directory/file targets and find-file through owned PTYs."""

import os
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parent.parent
sys.dont_write_bytecode = True
PTY = runpy.run_path(str(ROOT / 'scripts/check-pty.py'))


def snapshot(root):
    return {str(path.relative_to(root)): path.read_bytes() if path.is_file() else None
            for path in root.rglob('*')}


def environment(root):
    return {key: str(root / location) for key, location in [
        ('HOME', 'home'), ('XDG_CONFIG_HOME', 'user'), ('XDG_DATA_HOME', 'data'),
        ('XDG_STATE_HOME', 'state'), ('XDG_CACHE_HOME', 'cache')]}


def wait(session, marker, content=None):
    session.wait(lambda: PTY['frame_contains'](session.output, marker, content))
    return PTY['last_frame'](session)


def case(name, target, visible=False, policy='auto'):
    with tempfile.TemporaryDirectory(prefix='rev-directory-targets-') as directory:
        root = Path(directory)
        work = root / 'workspace'
        chosen = work / 'chosen 界 space'
        (chosen / 'child').mkdir(parents=True)
        (chosen / 'child/deep.txt').write_text('unobserved descendant')
        (chosen / 'main.txt').write_text('retained\n')
        (work / 'empty').mkdir()
        config = root / 'user/rev/config.toml'
        config.parent.mkdir(parents=True)
        config.write_text(f'tree.visible = {str(visible).lower()}\ntree.root = "{policy}"\n')
        env = environment(root)
        path = None if target is None else (work / target)
        argument = None if path is None else str(path if name.startswith('absolute') else target)
        before = snapshot(root)
        command = [str(ROOT / 'target/debug/rev')] + ([] if argument is None else [argument])
        session = PTY['Session'](command, env=env, cwd=work)
        try:
            directory_target = path is not None and path.is_dir()
            initial_root = (path if directory_target else
                            path.parent if path is not None and policy != 'cwd' else work)
            label = f'v {initial_root.name}/'.encode()
            if directory_target or visible:
                frame = wait(session, b'Tree *' if directory_target else b'L1 C1 Text', label)
                assert b'deep.txt' not in frame, 'startup enumerated a descendant'
            else:
                frame = wait(session, b'L1 C1 Text')
                assert label not in frame, 'file startup forced the tree visible'
                session.output.clear()
                os.write(session.master, b'\x18tt')
                wait(session, b'Tree *', label)
            session.output.clear()
            os.write(session.master, b'\x18\x06')
            wait(session, b'Find file: ')
            session.output.clear()
            os.write(session.master, b'\r')
            base = path.parent if path is not None and path.is_file() else initial_root
            frame = wait(session, b'Tree *', f'v {base.name}/'.encode())
            assert b'Find file:' not in frame, 'directory acceptance retained the prompt'
            assert b'deep.txt' not in frame
            assert Path(f'/proc/{session.process.pid}/cwd').resolve() == work, 'opening changed launch cwd'
            PTY['accepted_exit'](session)
            assert snapshot(root) == before, 'browsing changed fixture/configuration bytes'
        finally:
            session.close()
    print(f'PASS {name}: CLI target, expanded root, empty Return, exact files and terminal restoration')


def denied():
    with tempfile.TemporaryDirectory(prefix='rev-directory-denied-') as directory:
        root = Path(directory)
        target = root / 'denied'
        target.mkdir()
        config = root / 'user/rev/config.toml'
        config.parent.mkdir(parents=True)
        config.write_text('tree.visible = false\n')
        target.chmod(0)
        session = PTY['Session']([str(ROOT / 'target/debug/rev'), str(target)], env=environment(root), cwd=root)
        try:
            session.finish(1)
            assert b'Permission denied' in session.output
            assert PTY['LEAVE'] in session.output
        finally:
            target.chmod(0o700)
            session.close()
    print('PASS denied directory: explicit failure and restored terminal')


def tree_click(activate):
    listing = runpy.run_path(str(ROOT / 'scripts/check-command-list.py'))
    with tempfile.TemporaryDirectory(prefix='rev-tree-click-') as directory:
        root = Path(directory)
        work = root / 'workspace'
        (work / 'child').mkdir(parents=True)
        (work / 'child/nested.txt').write_bytes(b'nested child\n')
        source = work / 'source.txt'
        source.write_bytes(b'retained source\n')
        target = work / 'chosen 界.txt'
        target.write_bytes(b'clicked target\n')
        config = root / 'user/rev/config.toml'
        config.parent.mkdir(parents=True)
        config.write_text(f'tree.visible = true\nmouse.tree_activate_on_click = {str(activate).lower()}\n')
        before = snapshot(root)
        session = PTY['Session']([str(ROOT / 'target/debug/rev'), str(source)],
                                 env=dict(environment(root), NO_COLOR='1'), cwd=work)
        try:
            session.ready(b'> source.txt -- RW -- L1 C1 Text')
            wait(session, b'> source.txt -- RW -- L1 C1 Text')
            def press(label, predicate):
                frame = PTY['last_frame'](session)
                row = next(row for row, text in listing['rows'](frame).items()
                           if label in text.split('│'.encode())[0])
                session.output.clear()
                os.write(session.master, f'\x1b[<0;2;{row}M'.encode())
                session.wait(lambda: PTY['frame_contains'](session.output, b'', None)
                             and predicate(PTY['last_frame'](session)))
                return PTY['last_frame'](session)
            # Wire rows are one-based; the parser and native decoder own both
            # conversions. The fixture uses the actual composed tree row label.
            frame = press(b'child/', lambda f: b'Tree *' in f)
            assert (b'nested.txt' in frame) == activate
            if activate:
                press(b'child/', lambda f: b'nested.txt' not in f and b'Tree *' in f)
            marker = '> chosen 界.txt -- RW -- L1 C1 Text'.encode()
            frame = press('chosen 界.txt'.encode(), lambda f:
                          PTY['status_contains'](f, marker) if activate else
                          b'Tree *' in f and PTY['status_contains'](f, b'source.txt -- RW -- L1 C1 Text'))
            if not activate:
                # Tree focus makes the retained editing pane inactive: its
                # source/point stay unchanged and its active '>' marker is absent.
                assert b'> source.txt' not in frame
                assert b'clicked target' not in frame
                session.output.clear()
                os.write(session.master, b'\r')
                wait(session, marker, b'clicked target')
            assert PTY['frame_contains'](session.output, marker, b'clicked target')
            session.output.clear()
            os.write(session.master, b'\x18\x03')
            session.finish(0)
            assert PTY['LEAVE'] in session.output and PTY['PASTE_OFF'] in session.output
            assert snapshot(root) == before
        finally:
            session.close()
    print(f'PASS native SGR tree clicks: activate={activate}, directory/leaf identity, explicit keyboard fallback, exact files and terminal restoration')


def main():
    subprocess.run(['cargo', 'build', '--locked', '--bin', 'rev'], cwd=ROOT, check=True, timeout=180)
    if sys.argv[1:] == ['--tree-click']:
        for activate in [True, False]:
            tree_click(activate)
        return
    for name, target, visible, policy in [
        ('no-argument', None, False, 'auto'),
        ('dot-directory', '.', False, 'auto'),
        ('relative-directory', 'chosen 界 space', False, 'auto'),
        ('absolute-directory', 'chosen 界 space', False, 'cwd'),
        ('empty-directory', 'empty', False, 'auto'),
        ('relative-file', 'chosen 界 space/main.txt', False, 'auto'),
        ('absolute-file', 'chosen 界 space/main.txt', True, 'file-parent'),
        ('configured-cwd-file', 'chosen 界 space/main.txt', True, 'cwd'),
    ]:
        case(name, target, visible, policy)
    denied()


if __name__ == '__main__':
    main()
