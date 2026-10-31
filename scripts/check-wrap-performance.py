#!/usr/bin/env python3
"""Focused long-line input, resize, reversal and idle measurements on real PTYs."""

import fcntl
import json
import os
from pathlib import Path
import runpy
import struct
import subprocess
import sys
import tempfile
import termios
import time


ROOT = Path(__file__).resolve().parent.parent
sys.dont_write_bytecode = True
PTY = runpy.run_path(str(ROOT / 'scripts/check-pty.py'))
IDLE = runpy.run_path(str(ROOT / 'scripts/check-performance.py'))['idle']


def observe(session, operation, column, dirty=False, echo=None):
    session.output.clear()
    started = time.perf_counter()
    operation()
    marker = f'{"**" if dirty else "--"} RW -- L1 C{column} Text'.encode()
    session.wait(lambda: PTY['frame_contains'](session.output, marker, echo))
    return round((time.perf_counter() - started) * 1000, 3)


def case(directory, name, source):
    path = directory / f'{name}.txt'
    path.write_bytes(source)
    session = PTY['Session']([str(ROOT / 'target/release/rev'), str(path)])
    try:
        session.wait(lambda: PTY['frame_contains'](session.output, b'-- RW -- L1 C1 Text', None))
        end = len(source) + 1
        far = observe(session, lambda: os.write(session.master, b'\x1b>'), end)
        resize = []
        for width in [48, 81, 52, 80]:
            resize.append(observe(session, lambda: fcntl.ioctl(
                session.slave, termios.TIOCSWINSZ, struct.pack('HHHH', 24, width, 0, 0)), end))
        typed = observe(session, lambda: os.write(session.master, b'x' * 128), end + 128, True)
        reversal = observe(session, lambda: os.write(session.master, b'\x7f' * 128), end, True)
        # Interleave a cold resize, visual C-e and a finite ordered input burst.
        def cold_burst():
            fcntl.ioctl(session.slave, termios.TIOCSWINSZ, struct.pack('HHHH', 24, 47, 0, 0))
            os.write(session.master, b'\x05' + b'y' * 128 + b'\x7f' * 128 + b'\x07')
        queued = observe(session, cold_burst, end, True, echo=b'Quit')
        assert b'Quit' in PTY['last_frame'](session), 'interrupt did not finish after retained input'
        # Save confirms exact source restoration, not just an equal cursor.
        observe(session, lambda: os.write(session.master, b'\x18\x13'), end, echo=b'Saved')
        assert path.read_bytes() == source, 'input/reversal changed source bytes'
        quiet = IDLE(session)
        PTY['accepted_exit'](session)
        return {'case': name, 'bytes': len(source), 'far_ms': far,
                'resize_max_ms': max(resize), 'input_128_ms': typed,
                'reversal_128_ms': reversal, 'cold_queued_reversal_ms': queued,
                'idle': quiet, 'exact_saved_source': True, 'terminal_restored': True}
    finally:
        session.close()


def main():
    subprocess.run(['cargo', 'build', '--locked', '--release', '--bin', 'rev'],
                   cwd=ROOT, check=True, timeout=180)
    with tempfile.TemporaryDirectory(prefix='rev-wrap-performance-') as directory:
        results = [case(Path(directory), name, source) for name, source in [
            ('token', b'a' * 65536), ('words', b'one two three four ' * 3276)]]
    print(json.dumps(results, indent=2))
    print('Wrap PTY measurements passed: finite input/reversal, resize, exact save, idle and restoration.')


if __name__ == '__main__':
    main()
