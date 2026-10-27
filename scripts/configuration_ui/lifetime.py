"""Exercise the existing terminal owner's output-error and unwind paths."""

import os

from .support import ROOT, PTY, close, environment, passive, restoration, snapshot


def fault(root, mode):
    expected = {'io': 1, 'unwind': 101}[mode]
    command = [str(ROOT / 'target/debug/examples/terminal-probe'), mode]
    session = PTY['Session'](command, env=environment(root))
    try:
        session.ready(b'probe-ready')
        assert b'\x1b]12;#012345\x07' in session.output
        assert b'\x1b[38;2;1;35;69m' in session.output
        assert b'\x1b[48;2;67;89;101m' in session.output
        os.write(session.master, b'!')
        session.finish(expected)
        restoration(session)
        assert bytes(session.output).index(b'\x1b]12;#012345\x07') < bytes(session.output).index(b'\x1b]112\x07')
    finally:
        close(session)
    assert snapshot(root) == {}
    passive(root)
    print(f'PASS {mode} terminal failure: expected exit, cursor/text color reset, termios/paste/screen restoration and reaped child/closed PTYs')
