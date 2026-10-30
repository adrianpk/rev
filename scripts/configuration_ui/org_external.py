"""Owned PNG protocol, source coordinates, explicit viewers and restoration."""
import base64
import fcntl
import os
import re
import shutil
import struct
import termios
from pathlib import Path

from .org_document import encoded, exact as source_exact
from .org_time import quiet
from .support import (BINARY, ROOT, PTY, close, completed, environment, finish,
                      restoration, snapshot, step, write)

LINES = ['[[file:pixel.png]]', 'body 界 keep', 'tail EOF']
PNG = (ROOT / 'tests/fixtures/org-preview.png').read_bytes()
APC = re.compile(rb'\x1b_G([^;]*);([^\x1b]*)\x1b\\')
PLACE = re.compile(rb'\x1b\[(\d+);(\d+)H\x1b_Ga=p,([^;]*);\x1b\\')
POLICY = b"mouse.enabled=true\norg.limits.job_seconds=2\norg.tools.viewer=['/bin/sh','viewer.sh','{target}']\n"


def exact(frame):
    # APCs are graphics controls, not source cells. Their complete payload,
    # geometry and ownership are independently asserted below.
    return source_exact(APC.sub(b'', frame), LINES)


def launch(root, colored, program, arguments):
    env = environment(root, colored)
    env['TERM'] = 'xterm-kitty' if colored else 'xterm-256color'
    # The owned PTY is direct, even when its parent harness runs under tmux.
    command = [shutil.which('env'), '-u', 'TMUX', '-u', 'STY', str(program), *arguments]
    assert command[0] is not None
    return PTY['Session'](command, env=env, cwd=root / 'work')


def placements(wire, width=80, height=24):
    ids = set()
    for row, column, values in PLACE.findall(wire):
        data = dict(part.split(b'=') for part in values.split(b','))
        r, c = int(row), int(column)
        rows, columns = int(data[b'r']), int(data[b'c'])
        assert data[b'C'] == b'1' and data[b'z'] == b'-1' and data[b'q'] == b'2'
        assert 1 <= rows <= 12 and 1 <= columns <= 40
        assert 1 <= r <= r + rows - 1 <= height - 2
        assert 7 <= c <= c + columns - 1 <= width
        ids.add(data[b'i'])
    return ids


def deleted(wire):
    return set(re.findall(rb'a=d,d=I,i=(\d+),q=2;', wire))


def resize(session, width, height):
    session.output.clear()
    fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack('HHHH', height, width, 0, 0))
    session.height = height
    session.wait(lambda: completed(session, lambda f: b'-- RW --' in f))
    frame = PTY['last_frame'](session)
    PTY['terminal_bounds'](frame, width, height)
    return frame



def named(session, command, marker):
    return step(session, b'\x1bx' + command.encode() + b'\r', marker)


def stopped(pid):
    path = Path(f'/proc/{pid}/stat')
    return not path.exists() or path.read_text().split(') ', 1)[1].startswith('Z')


def journey(root, colored):
    write(root, 'user/rev/config.toml', POLICY)
    source = write(root, 'work/source.org', encoded(LINES))
    image = write(root, 'work/pixel.png', PNG)
    write(root, 'work/viewer.sh', b'printf "%s" "$1" > called; echo $$ > leader; sleep 30 & echo $! > descendant; wait\n')
    before = snapshot(root)
    session = launch(root, colored, BINARY, [str(source)])
    try:
        session.ready(b'> source.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, exact))
        quiet(session)
        assert b'\x1b_G' not in session.output and not (source.parent / 'called').exists()
        frame = step(session, b'\x03\x18\x16', b'Org previews: 1 PNG')
        assert exact(frame) and b'L1 C1' in frame
        ids = placements(frame)
        if colored:
            assert len(ids) == 1
            uploads = [payload for header, payload in APC.findall(frame) if header.startswith(b'a=t,')]
            assert len(uploads) == 1 and base64.b64decode(uploads[0], validate=True) == PNG
        else:
            assert not ids and b'\x1b_G' not in frame and b'terminal PNG unavailable' in frame
        assert source.read_bytes() == encoded(LINES)
        frame = step(session, b'\x1b[<0;9;2M\x1b[<0;9;2m', b'L2 C3')
        assert exact(frame), 'preview changed mouse/source coordinates'
        step(session, b'\x1bxgoto-line\r', b'Goto line: ')
        step(session, b'1\r', b'L1 C1')
        frame = step(session, b'\x18' + b'3', b'-- RW --')
        ids |= placements(frame)
        frame = named(session, 'org-link-preview-clear', b'Org previews cleared')
        retired = deleted(frame)
        frame = step(session, b'\x18' + b'1', b'-- RW --')
        retired |= deleted(frame)
        assert retired == ids, 'split clear/window removal did not retire owned graphics'
        quiet(session)
        step(session, b'\x03\x18\x16', b'Org previews: 1 PNG')
        active = placements(PTY['last_frame'](session))
        frame = resize(session, 31, 8)
        assert placements(frame, 31, 8) == active
        assert b'a=t,' not in frame, 'geometry change retransmitted data'
        frame = resize(session, 80, 24)
        assert exact(frame) and placements(frame) == active
        quiet(session)
        step(session, b'\x03\x0f', b'Run configured viewer? yes/no: ')
        assert not (source.parent / 'called').exists()
        step(session, b'no\r', b'nothing launched')
        assert not (source.parent / 'called').exists()
        step(session, b'\x03\x0f', b'Run configured viewer? yes/no: ')
        step(session, b'yes\r', b'Configured viewer running')
        leader_path = source.parent / 'leader'
        child_path = source.parent / 'descendant'
        session.wait(lambda: child_path.exists() and child_path.read_text().strip().isdigit())
        leader, child = int(leader_path.read_text()), int(child_path.read_text())
        step(session, b'\x07', b'Org link cancelled; source unchanged')
        session.wait(lambda: not Path(f'/proc/{leader}').exists() and stopped(child))
        assert (source.parent / 'called').read_bytes() == str(image).encode()
        quiet(session)
        assert source.read_bytes() == encoded(LINES) and image.read_bytes() == PNG
        if not colored:
            assert b'\x1b_G' not in session.output
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        assert deleted(session.output) == active, 'exit did not free owned graphics'
        assert b'\x1b[?1002l' in session.output and b'\x1b[?1006l' in session.output
        before['work/called'] = str(image).encode()
        before['work/leader'] = f'{leader}\n'.encode()
        before['work/descendant'] = f'{child}\n'.encode()
        assert snapshot(root) == before
    finally:
        close(session)


def fault(root, mode):
    write(root, 'user/rev/config.toml', b'')
    source = write(root, 'work/source.org', encoded(LINES))
    write(root, 'work/pixel.png', PNG)
    before = snapshot(root)
    session = launch(root, True, ROOT / 'target/debug/examples/terminal-probe', [mode, str(source)])
    try:
        session.ready(b'probe-ready')
        os.write(session.master, b'!')
        session.finish(1 if mode == 'org-preview-io' else 101)
        wire = bytes(session.output)
        acquired = set(re.findall(rb'a=t,t=d,f=100,i=(\d+),', wire))
        assert len(acquired) == 1 and placements(wire) == acquired
        assert deleted(wire) == acquired, 'failure/unwind lost graphics cleanup'
        restoration(session)
        assert snapshot(root) == before
    finally:
        close(session)
