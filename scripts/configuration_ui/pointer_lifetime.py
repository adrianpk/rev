"""Exercise the native pointer reporting owner with bounded controlling PTYs."""

import os
import re
import termios

from .support import ROOT, PTY, close, environment, passive, restoration, snapshot

SGR_ON = b'\x1b[?1006h'
BUTTONS_ON = b'\x1b[?1002h'
BUTTONS_OFF = b'\x1b[?1002l'
SGR_OFF = b'\x1b[?1006l'
EARLY = ['mouse-format', 'mouse-buttons', 'mouse-rollback']


def raw(session):
    assert not termios.tcgetattr(session.slave)[3] & (termios.ICANON | termios.ECHO)


def toggle(session):
    os.write(session.master, b'!')
    session.wait(lambda: b'probe-mouse-off' in session.output)
    raw(session)
    assert session.output.count(SGR_ON) == session.output.count(BUTTONS_ON) == 1
    assert session.output.count(SGR_OFF) == session.output.count(BUTTONS_OFF) == 1
    assert PTY['PASTE_OFF'] not in session.output, 'toggle released other terminal owners'
    os.write(session.master, b'!')
    session.wait(lambda: b'probe-mouse-on' in session.output)
    raw(session)
    assert session.output.count(SGR_ON) == session.output.count(BUTTONS_ON) == 2
    assert session.output.count(SGR_OFF) == session.output.count(BUTTONS_OFF) == 1


def reporting(session, mode):
    if mode in EARLY:
        session.finish(1)
        return
    session.ready(b'probe-ready')
    if mode == 'mouse-toggle':
        toggle(session)
    os.write(session.master, b'!')
    if mode == 'mouse-restore':
        session.wait(lambda: b'probe-cleanup-pending' in session.output)
        assert termios.tcgetattr(session.slave) == session.before
        for operation in [b'disable mouse button tracking', b'disable SGR mouse coordinates']:
            # Diagnostics arrive when the final error is returned, after this
            # acknowledgment. Both output obligations are still pending here.
            assert operation not in session.output
        # BufWriter can flush a failed command while continuing other cleanup.
        # Retained obligations are proved by fresh resets after this checkpoint.
        os.write(session.master, b'\n')
    expected = {'mouse-normal': 0, 'mouse-toggle': 0, 'mouse-unwind': 101}.get(mode, 1)
    session.finish(expected)


def observations(session, mode):
    wire = bytes(session.output)
    modes = re.findall(rb'\x1b\[\?(10\d\d)[hl]', wire)
    assert set(modes) <= {b'1049', b'1002', b'1006'}, modes
    if mode == 'mouse-before-entry':
        assert PTY['ENTER'] not in wire and not modes
        assert b'requires the alternate screen' in wire
        return
    restoration(session, False)
    assert SGR_OFF in wire
    if mode == 'mouse-format':
        assert BUTTONS_ON not in wire and BUTTONS_OFF not in wire
    else:
        assert BUTTONS_OFF in wire
    if mode in ['mouse-normal', 'mouse-toggle']:
        cycles = 2 if mode == 'mouse-toggle' else 1
        assert all(wire.count(code) == cycles for code in [SGR_ON, BUTTONS_ON, BUTTONS_OFF, SGR_OFF])
        assert wire.rindex(BUTTONS_OFF) < wire.rindex(SGR_OFF) < wire.index(PTY['PASTE_OFF'])
    if mode in ['mouse-restore', 'mouse-rollback']:
        assert b'disable mouse button tracking' in wire
        assert b'disable SGR mouse coordinates' in wire
    if mode == 'mouse-restore':
        retry = wire.split(b'probe-cleanup-pending', 1)[1]
        assert BUTTONS_OFF in retry and SGR_OFF in retry, 'Drop lost failed mouse obligations'


def case(root, mode):
    before = snapshot(root)
    command = [str(ROOT / 'target/debug/examples/terminal-probe'), mode]
    session = PTY['Session'](command, env=environment(root, False))
    try:
        reporting(session, mode)
        observations(session, mode)
    finally:
        close(session)
    assert snapshot(root) == before
    passive(root)
    print(f'PASS pointer lifetime {mode}: limited modes, termios, owned cleanup and passive filesystem')


def run(root):
    modes = ['mouse-before-entry', 'mouse-normal', 'mouse-toggle',
             'mouse-format', 'mouse-buttons', 'mouse-rollback',
             'mouse-buttons-stop', 'mouse-format-stop', 'mouse-restore',
             'mouse-io', 'mouse-unwind']
    for mode in modes:
        fixture = root / mode
        fixture.mkdir()
        case(fixture, mode)
    print(f'Pointer lifetime PTY suite: {len(modes)} owned sessions passed, 0 skipped.')
