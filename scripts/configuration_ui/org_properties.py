"""Property prompts, provenance and checked source edits in the owned Org PTY."""

from .support import canvas, send, step, visit

LINES = ['#+PROPERTY: CUSTOM default', '* Parent', ':PROPERTIES:',
         ':PROJECT: root', ':PROJECT+: base', ':UNKNOWN: retained', ':END:',
         '** Child', 'body', ':LOGBOOK:', 'note', ':END:']
SOURCE = ('\ufeff' + '\r\n'.join(LINES)).encode()


def property_journey(session, source, goto, document):
    def exact(frame, lines):
        text = document(frame)
        return text[:len(lines)] == lines and not any(text[len(lines):])

    def named(name, label):
        return step(session, b'\x1bx' + name.encode() + b'\r', label)

    def argument(value, label):
        return step(session, value.encode() + b'\r', label)

    def unchanged():
        return send(session, b'\x18u', lambda frame: exact(frame, LINES) and b'-- RW --' in frame)

    frame = visit(session, str(source), b'> properties.org -- RW -- L1 C1')
    assert exact(frame, LINES)
    goto(session, 8)
    named('org-inspect-property', b'Org property: ')
    frame = argument('PROJECT', b'inherited PROJECT at line 4')
    assert 'root base' in canvas(frame)[24]
    step(session, b'\x03\x18p', b'Org property: ')
    argument('PROJECT', b'Org property value: ')
    expected = LINES[:8] + [':PROPERTIES:', ':PROJECT: child', ':END:'] + LINES[8:]
    send(session, b'child\r', lambda frame: exact(frame, expected) and b'Org property changed' in frame)
    unchanged()
    goto(session, 8)
    named('org-edit-property-owner', b'Org property: ')
    argument('PROJECT', b'Property owner number: ')
    argument('1', b'Owner property value: ')
    expected = LINES.copy()
    expected[3] = ':PROJECT: changed'
    send(session, b'changed\r', lambda frame: exact(frame, expected) and b'Org property changed' in frame)
    unchanged()
    goto(session, 8)
    step(session, b'\x03\x18p', b'Org property: ')
    argument('CANCELLED', b'Org property value: ')
    send(session, b'\x07', lambda frame: exact(frame, LINES) and b'Quit' in frame)
    named('org-delete-property', b'Org property: ')
    send(session, b'PROJECT\r', lambda frame: exact(frame, LINES) and b'inherited owners retained' in frame)
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == SOURCE, 'property operations changed BOM/CRLF/EOF bytes after undo'
