"""Owned fixture and completed-frame helpers for configuration/UI journeys."""

import errno
import os
from pathlib import Path
import re
import runpy
import unicodedata


ROOT = Path(__file__).resolve().parents[2]
LIST = runpy.run_path(str(ROOT / 'scripts/check-command-list.py'))
PTY = LIST['PTY']
BINARY = ROOT / 'target/debug/rev'
CONFIG = """editor.indent_width = 2
editor.tab_width = 8
files.bom = true
files.newline = 'crlf'
minibuffer.candidate_limit = 2
palette.background = '#101112'
palette.foreground = '#010203'
palette.cursor = '#112233'
palette.selection = '#112233'
palette.selection_text = '#445566'
[keys.editor]
'M-j' = 'newline'
[keys.minibuffer]
'M-a' = 'minibuffer-accept'
'M-c' = 'minibuffer-complete'
'M-j' = 'minibuffer-next'
'M-k' = 'minibuffer-previous'
[keys.tree]
'M-l' = 'tree-expand'
"""


def environment(root, colored=True):
    return dict(LIST['environment'](root, colored), TERM='xterm-256color')


def write(root, name, data):
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return path


def fixtures(root, configured=True):
    if configured:
        write(root, 'user/rev/config.toml', CONFIG.encode())
    path = write(root, 'work/main.rs', '\tlet label = "界e\u0301";\n'.encode())
    write(root, 'work/alpha.txt', b'alpha\n')
    write(root, 'work/alpine.txt', b'alpine\n')
    write(root, 'work/space dir/界 name.txt', 'chosen 界e\u0301\n'.encode())
    return path


def snapshot(root):
    result = {}
    for path in root.rglob('*'):
        assert len(result) < 32, 'fixture exceeds 32 entries'
        result[str(path.relative_to(root))] = read(path) if path.is_file() else None
    return result


def read(path):
    with path.open('rb') as stream:
        data = stream.read(65537)
    assert len(data) <= 65536, 'fixture file exceeds 64 KiB'
    return data


def rows(frame):
    return {row: text.decode('utf-8') for row, text in LIST['rows'](frame).items()}


def cursor(frame):
    moves = re.findall(rb'\x1b\[(\d+);(\d+)H', frame)
    assert frame.endswith(PTY['SHOW']) and moves
    return tuple(int(value) for value in moves[-1])


def completed(session, predicate):
    if not PTY['frame_contains'](session.output, b'', None):
        return False
    return predicate(PTY['last_frame'](session))


def send(session, keys, predicate):
    session.output.clear()
    os.write(session.master, keys)
    session.wait(lambda: completed(session, predicate))
    assert session.process.poll() is None and PTY['LEAVE'] not in session.output
    return PTY['last_frame'](session)


def step(session, keys, marker, content=None):
    return send(session, keys, lambda frame: marker in frame and (content is None or content in frame))


def canvas(frame, width=80, height=24):
    content = rows(frame)
    assert set(content) == set(range(1, height + 1)), content
    assert all(cells(text) == width for text in content.values()), content
    PTY['terminal_bounds'](frame, width, height)
    return content


def cells(text):
    return sum(0 if unicodedata.combining(char) else 1 + (unicodedata.east_asian_width(char) in 'WF') for char in text)


def query(frame, text):
    content = rows(frame)
    line = next(row for row, value in content.items() if value.startswith(text))
    assert cursor(frame)[0] == line
    return line


def visit(session, draft, marker):
    step(session, b'\x18\x06', b'Find file: ')
    step(session, PTY['paste_wire'](draft.encode()), draft.encode())
    return step(session, b'\r', marker)


def finish(session, colored=True):
    os.write(session.master, b'\x18\x03')
    session.finish(0)
    restoration(session, colored)


def restoration(session, colored=True):
    wire = bytes(session.output)
    cleanup = wire[wire.rindex(PTY['PASTE_OFF']):]
    codes = [PTY['PASTE_OFF'], PTY['SHOW'], b'\x1b[0m']
    if colored:
        codes.append(b'\x1b]112\x07')
    else:
        assert b'\x1b]12;' not in wire and b'\x1b]112' not in wire
    codes.append(PTY['LEAVE'])
    offsets = [cleanup.index(code) for code in codes]
    assert offsets == sorted(offsets), 'terminal restoration order changed'
    assert session.process.returncode is not None, 'owned child was not reaped'


def close(session):
    session.close()
    assert session.process.returncode is not None, 'owned child survived cleanup'
    for descriptor in [session.master, session.slave]:
        try:
            os.fstat(descriptor)
        except OSError as error:
            assert error.errno == errno.EBADF
            continue
        raise AssertionError('owned PTY descriptor survived cleanup')


def passive(root):
    for name in ['data', 'state', 'cache']:
        assert not (root / name).exists(), name
