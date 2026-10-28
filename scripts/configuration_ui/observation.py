"""Linux open/access evidence on the owned fixture, without an external tracer."""

import ctypes
import os
import struct


OPEN = 0x20
ACCESS = 0x1
OVERFLOW = 0x4000
IGNORED = 0x8000
DIRECTORIES = ['user/rev', 'entries', 'entries/nested', 'names', 'names/nested',
               'small', 'small/nested', 'small/nested/grandchild', 'work']


class Observation:
    def __init__(self, root, directories=DIRECTORIES, mask=OPEN | ACCESS):
        self.libc = ctypes.CDLL(None, use_errno=True)
        self.libc.inotify_init1.argtypes = [ctypes.c_int]
        self.libc.inotify_add_watch.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_uint32]
        self.fd = self.libc.inotify_init1(os.O_NONBLOCK | os.O_CLOEXEC)
        if self.fd < 0:
            raise OSError(ctypes.get_errno(), 'inotify initialization failed')
        self.paths = {}
        self.mask = mask
        try:
            for relative in directories:
                self.add(root, relative)
        except BaseException:
            os.close(self.fd)
            raise

    def add(self, root, relative):
        watch = self.libc.inotify_add_watch(self.fd, os.fsencode(root / relative), self.mask)
        if watch < 0:
            raise OSError(ctypes.get_errno(), 'fixture watch failed')
        self.paths[watch] = relative

    def __enter__(self):
        return self

    def __exit__(self, *_):
        os.close(self.fd)

    def drain(self):
        events = []
        while True:
            try:
                data = os.read(self.fd, 65536)
            except BlockingIOError:
                return events
            events.extend(self.decode(data))
            assert len(events) <= 8192, 'fixture event bound exceeded'

    def decode(self, data):
        offset = 0
        events = []
        while offset < len(data):
            watch, mask, _, length = struct.unpack_from('iIII', data, offset)
            assert not mask & (OVERFLOW | IGNORED), 'filesystem evidence lost'
            name = data[offset + 16:offset + 16 + length].split(b'\0')[0]
            events.append((self.paths[watch], mask, name))
            offset += 16 + length
        assert offset == len(data)
        return events


def no_recursion(events):
    assert not any('/nested' in path or name == b'nested' for path, _, name in events), 'recursive child observation'


def evidence(kind, before, after):
    no_recursion(before + after)
    if kind == 'graph':
        assert sum(bool(mask & OPEN) for _, mask, _ in after) >= 64, 'graph read positive control missing'
    else:
        assert any(mask & OPEN for _, mask, _ in before), 'setup read positive control missing'
        assert not after, 'cached work opened or read a fixture entry'
    return {'setup_open_access_events': len(before), 'work_open_access_events': len(after),
            'recursive_events': 0}
