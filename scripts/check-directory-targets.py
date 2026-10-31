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


def main():
    subprocess.run(['cargo', 'build', '--locked', '--bin', 'rev'], cwd=ROOT, check=True, timeout=180)
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
