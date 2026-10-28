"""Measure ordered vertical repeat through the existing controlling PTY owner."""

import os
import re
import select
import time

from configuration_ui.support import (BINARY, PTY, close, completed, environment,
                                      finish, step)


BUDGET_MS = 150


def repeat(session, key, count, rate, expected):
    session.output.clear()
    started = time.monotonic()
    sent = 0
    first = None
    released = None
    received = 0
    deadline = started + count / rate + 4
    while time.monotonic() < deadline:
        now = time.monotonic()
        if sent < count and now >= started + sent / rate:
            os.write(session.master, key)
            sent += 1
            if sent == count:
                released = time.monotonic()
        if not select.select([session.master], [], [], 0.001)[0]:
            continue
        data = os.read(session.master, 65536)
        assert data, 'native input stream closed during repeat'
        received += len(data)
        assert received <= 1048576, 'repeat output exceeded 1 MiB'
        session.output.extend(data)
        frames = bytes(session.output).split(PTY['SHOW'])
        session.output[:] = frames[-1]
        for frame in frames[:-1]:
            positions = re.findall(rb'-- RW -- L(\d+) C1 +Text', frame)
            if not positions:
                continue
            if first is None:
                first = (time.monotonic() - started) * 1000
            if sent == count and int(positions[-1]) == expected:
                tail = (time.monotonic() - released) * 1000
                assert first <= BUDGET_MS, f'first update {first:.1f} ms exceeds {BUDGET_MS} ms'
                assert tail <= BUDGET_MS, f'release tail {tail:.1f} ms exceeds {BUDGET_MS} ms'
                # No input follows release: any subsequent frame would expose drift.
                assert not select.select([session.master], [], [], 0.15)[0], 'navigation drifted after release'
                return round(first, 2), round(tail, 2)
    raise AssertionError(f'ordered repeat did not reach line {expected}')


def run(root):
    for name, lines, width, height, rate, count in [
        ('ordinary', 400, 80, 24, 30, 30),
        ('ordinary-wide', 400, 240, 70, 120, 120),
        ('large', 100000, 80, 24, 30, 30),
        ('large-wide', 100000, 240, 70, 120, 120),
    ]:
        fixture = root / name
        fixture.mkdir()
        path = fixture / 'navigation.txt'
        source = b'navigation row with ordinary text\n' * lines
        path.write_bytes(source)
        session = PTY['Session']([str(BINARY), str(path)], env=environment(fixture))
        try:
            session.wait(lambda: completed(session, lambda frame: PTY['status_contains'](frame, b'-- RW -- L1 C1 Text')))
            if (width, height) != (80, 24):
                PTY['resize_surface'](session, width, height, True)
            for direction, key, target in [('down', b'\x1b[B', count + 1), ('up', b'\x1b[A', 1)]:
                first, tail = repeat(session, key, count, rate, target)
                print(f'PASS navigation {name} {direction}: {rate}/s, '
                      f'first={first} ms, release={tail} ms, budget={BUDGET_MS} ms', flush=True)
            # Ordered asymmetric reversal and text commands expose dropped input.
            step(session, b'\x1b[B' * 120 + b'\x1b[A' * 60 + b'X', b'** RW -- L61 C2 Text', b'Xnavigation')
            step(session, b'\x18u', b'-- RW -- L61 C1 Text')
            step(session, b'\x1b<\x1b[A' * 3, b'-- RW -- L1 C1 Text')
            step(session, b'\x1b>\x1b[B' * 3, f'-- RW -- L{lines + 1} C1 Text'.encode())
            step(session, b'\x1b', b'ESC')
            session.output.clear()
            session.wait(lambda: completed(session, lambda frame: b'Escape' in frame))
            step(session, b'f', f'** RW -- L{lines + 1} C2 Text'.encode())
            step(session, b'\x18u', f'-- RW -- L{lines + 1} C1 Text'.encode())
            step(session, b'\x18r', b'C-x r')
            step(session, b'\x07', b'Quit')
            finish(session)
            assert path.read_bytes() == source, 'navigation modified document bytes'
        finally:
            close(session)
    print('Navigation PTY suite: 4 owned sessions passed, 0 skipped.')


def goto_line(root):
    from configuration_ui.support import cursor, rows

    for name, source, last_line, last_column in [
        ('unicode', '\r\n'.join(f'row {line:03} 界e\u0301' for line in range(1, 81)).encode(), 80, 12),
        ('empty', b'', 1, 1),
    ]:
        fixture = root / name
        fixture.mkdir()
        path = fixture / 'goto.txt'
        path.write_bytes(source)
        session = PTY['Session']([str(BINARY), str(path)], env=environment(fixture))
        try:
            session.wait(lambda: completed(session, lambda frame: PTY['status_contains'](frame, b'-- RW -- L1 C1 Text')))
            step(session, b'\x1bg', b'M-g')
            step(session, b'g', b'Goto line: ')
            frame = step(session, b'37', b'Goto line: 37')
            assert PTY['status_contains'](frame, b'-- RW -- L1 C1 Text'), 'draft moved document point'
            target = 37 if source else 1
            target_status = f'-- RW -- L{target} C1 Text'.encode()
            frame = step(session, b'\r', target_status)
            assert cursor(frame)[0] < 23, 'goto did not reveal document cursor'
            before = {row: text for row, text in rows(frame).items() if row < 24}
            step(session, b'\x1bg\x1bg', b'Goto line: ')
            step(session, b'78', b'Goto line: 78')
            frame = step(session, b'\x07', b'Quit')
            assert {row: text for row, text in rows(frame).items() if row < 24} == before
            assert PTY['status_contains'](frame, target_status), 'cancel moved document point'
            step(session, b'\x1bxgoto-line\r', b'Goto line: ')
            draft = b'9223372036854775808'
            frame = step(session, draft + b'\r', b'Enter a signed 64-bit decimal line number')
            assert b'Goto line: ' + draft in frame and PTY['status_contains'](frame, target_status)
            corrected = b'\x01' + b'\x1b[3~' * len(draft) + b'+2\r'
            step(session, corrected, f'-- RW -- L{2 if source else 1} C1 Text'.encode())
            step(session, b'\x1bgg-1\r', b'-- RW -- L1 C1 Text')
            step(session, b'\x1bgg999\r', f'-- RW -- L{last_line} C{last_column} Text'.encode())
            step(session, b'\x1bg', b'M-g')
            step(session, b'\x07', b'Quit')
            finish(session)
            assert path.read_bytes() == source, 'goto-line changed source bytes'
            print(f'PASS goto-line {name}: aliases, M-x, correction, bounds, reveal, cancellation and source bytes')
        finally:
            close(session)
    print('Goto-line PTY suite: 2 owned sessions passed, 0 skipped.')
