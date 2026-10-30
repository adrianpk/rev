"""Observe the existing PTY owner's isolated roots, restoration and cleanup."""

from contextlib import contextmanager
import errno
import os
from pathlib import Path

from .support import PTY


@contextmanager
def owned(root):
    original = PTY['Session']
    sessions = []

    class Observed(original):
        def __init__(self, command, colors=False, env=None, cwd=None):
            names = {'HOME': 'home', 'XDG_CONFIG_HOME': 'user',
                     'XDG_DATA_HOME': 'data', 'XDG_STATE_HOME': 'state',
                     'XDG_CACHE_HOME': 'cache'}
            assert env is not None
            for key, name in names.items():
                assert Path(env[key]).is_relative_to(root), key
                assert Path(env[key]).name == name, key
            if cwd is None:
                cwd = root / 'work' if (root / 'work').is_dir() else root
            assert Path(cwd).is_relative_to(root), 'fixture cwd escaped ownership'
            super().__init__(command, colors, env, cwd)
            self.restored = False
            self.closed = False
            sessions.append(self)

        def finish(self, expected):
            super().finish(expected)
            self.restored = True

        def close(self):
            super().close()
            assert self.process.returncode is not None, 'owned child was not reaped'
            for descriptor in [self.master, self.slave]:
                try:
                    os.fstat(descriptor)
                except OSError as error:
                    assert error.errno == errno.EBADF
                else:
                    raise AssertionError('owned PTY descriptor remains open')
            self.closed = True

    PTY['Session'] = Observed
    try:
        yield sessions
        assert sessions, 'scenario created no owned terminal'
        assert all(s.restored and s.closed for s in sessions), 'incomplete terminal lifetime'
    finally:
        try:
            for session in sessions:
                if not session.closed:
                    session.close()
        finally:
            PTY['Session'] = original
