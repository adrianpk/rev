"""Verify startup and runtime mouse controls in bounded owned native PTYs."""

import re

from .gutter_geometry import idle
from .pointer_focus import report as press
from .pointer_scroll import report as wheel
from .pointer_selection import report as drag
from .pointer_lifetime import SGR_ON, SGR_OFF, BUTTONS_ON, BUTTONS_OFF, raw
from .support import (BINARY, PTY, close, cursor, environment, finish, passive,
                      rows, send, snapshot, step, write)

SOURCE = b'\xef\xbb\xbf' + ''.join(f'row-{index:03}\r\n' for index in range(150)).encode()


def modes(wire, transition=None):
    assert b'\x1b]52;' not in wire, 'mouse control wrote a terminal clipboard'
    assert set(re.findall(rb'\x1b\[\?(10\d\d)[hl]', wire)) <= {b'1049', b'1002', b'1006'}
    expected = {SGR_ON: 0, BUTTONS_ON: 0, BUTTONS_OFF: 0, SGR_OFF: 0}
    if transition is not None:
        first, second = (SGR_ON, BUTTONS_ON) if transition else (BUTTONS_OFF, SGR_OFF)
        expected[first] = expected[second] = 1
        assert wire.index(first) < wire.index(second), 'reporting pair order changed'
    assert all(wire.count(code) == count for code, count in expected.items()), expected


def stage(session, wire, predicate, transition=None):
    frame = send(session, wire, predicate)
    modes(bytes(session.output), transition)
    raw(session)
    return frame


def toggle(session, enabled, wire=b'\x1bxmouse-mode\r'):
    marker = b'Mouse mode enabled' if enabled else b'Mouse mode disabled'
    return stage(session, wire, lambda f: marker in f, enabled)


def factory(session, root):
    # Factory reporting is active. Wheel targets its pane without moving native
    # focus; ordinary point placement remains usable alongside keyboard commands.
    stage(session, b'\x183', lambda f: rows(f)[23].count('L1 C1') == 2)
    frame = stage(session, press(47, 1) + drag(47, 1, True) + wheel(False, 7, 0),
                  lambda f: cursor(f) == (2, 48) and 'row-003' in rows(f)[1])
    assert 'row-000' in rows(frame)[1]
    toggle(session, False)
    frame = stage(session, wheel(False, 47, 0) + b'\x18o', lambda f: cursor(f) == (1, 7))
    assert 'row-003' in rows(frame)[1] and 'row-000' in rows(frame)[1]
    idle(session)
    return False


def off_then_on(session, root):
    frame = stage(session, press(8, 1) + drag(8, 3) + drag(8, 3, True)
                  + wheel(False, 7, 0) + b'\x18', lambda f: b'C-x' in f)
    assert cursor(frame) == (1, 7) and 'row-000' in rows(frame)[1]
    step(session, b'\x07', b'Quit')
    toggle(session, True)
    frame = stage(session, wheel(False, 7, 0), lambda f: b'L8 C1' in f)
    assert 'row-007' in rows(frame)[1] and cursor(frame) == (1, 7)
    idle(session)
    return True


def restart_off(session, root):
    # Reusing the unchanged root in a new process restores its false preference.
    frame = stage(session, wheel(False, 7, 0) + b'\x18', lambda f: b'C-x' in f)
    assert cursor(frame) == (1, 7) and 'row-000' in rows(frame)[1]
    idle(session)
    # Retire the deliberately tested C-x prefix before the separate exit chord.
    step(session, b'\x07', b'Quit')
    return False


def capture(session, root):
    stage(session, press(7, 0) + drag(7, 22), lambda f: b'L24 C2' in f)
    # A mapped native command disables during a live outside drag. Late reports
    # in its same batch cannot extend the old region or leave a pending wake.
    frame = toggle(session, False, b'\x1bj' + drag(7, 23) + drag(7, 23, True))
    assert b'L24 C2' in frame and b'\x1b[7m' in frame
    idle(session)
    frame = stage(session, b'\x19', lambda f: b'Kill ring is empty' in f)
    assert b'L24 C2' in frame and b'\x1b[7m' in frame
    toggle(session, True, b'\x1bj' + drag(7, 23))
    idle(session)  # Reenable does not restore the retired capture/deadline.
    stage(session, wheel(False, 7, 0), lambda f: 'row-004' in rows(f)[1])
    return True


def invalid(session, root):
    # The source/key diagnostic must be visible, rather than clipped by the
    # original 80-column surface. Keep the same deadline and output ceiling.
    PTY['resize_surface'](session, 180, 24, True)
    frame = PTY['last_frame'](session)
    assert b'Configuration:' in frame and b'mouse.scroll_lines' in frame, rows(frame)[24]
    frame = stage(session, wheel(False, 7, 0), lambda f: 'row-003' in rows(f)[1])
    assert b'L4 C1' in frame  # Complete factory enabled/three-row fallback.
    idle(session)
    return True


def owned(root, journey, initially_enabled):
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(root / 'source.txt')], env=environment(root, False))
    try:
        session.ready(b'-- RW -- L1 C1')
        modes(bytes(session.output), True if initially_enabled else None)
        enabled = journey(session, root)
        session.output.clear()
        finish(session, False)
        modes(bytes(session.output), False if enabled else None)
    finally:
        close(session)
    assert snapshot(root) == before, 'runtime control changed settings or source bytes'
    passive(root)
    print(f'PASS mouse controls {journey.__name__}: native admission/reporting, immutable files and owned cleanup')


def run(root):
    for name, config, journeys, enabled in [
        ('factory', None, [factory], True),
        ('off', 'mouse.enabled = false\nmouse.scroll_lines = 7', [off_then_on, restart_off], False),
        ('capture', 'mouse.scroll_lines = 2\n[keys.editor]\n"M-j" = "mouse-mode"', [capture], True),
        ('invalid', 'mouse.enabled = false\nmouse.scroll_lines = 0', [invalid], True),
    ]:
        fixture = root / name
        fixture.mkdir()
        write(fixture, 'source.txt', SOURCE)
        if config is not None:
            write(fixture, 'user/rev/config.toml', config.encode())
        for journey in journeys:
            owned(fixture, journey, enabled)
    print('Pointer controls PTY suite: 5 owned sessions passed, 0 skipped.')
