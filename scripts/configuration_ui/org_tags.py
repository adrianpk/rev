"""Owned native tag entry, completion, provenance and match-query steps."""
from .support import canvas, send, step, visit

LINES = ['#+FILETAGS: :File:', '#+TAGS: Work weekend', '* TODO Parent :Work:',
         ':PROPERTIES:', ':PROJECT: root', ':END:', '** TODO Child :local:',
         'body', '* Peer :Else:']
SOURCE = ('\ufeff' + '\r\n'.join(LINES)).encode()


def tag_journey(session, source, goto, document):
    def exact(frame, lines):
        text = document(frame)
        return text[:len(lines)] == lines and not any(text[len(lines):])

    def named(name, label):
        return step(session, b'\x1bx' + name.encode() + b'\r', label)

    def unchanged():
        return send(session, b'\x18u', lambda frame: exact(frame, LINES) and b'-- RW --' in frame)

    frame = visit(session, str(source), b'> tags.org -- RW -- L1 C1')
    assert exact(frame, LINES)
    goto(session, 7)
    frame = named('org-inspect-tags', b'inherited [L3] Work')
    assert 'local [L7] local' in canvas(frame)[24] and 'file [L1] File' in canvas(frame)[24]
    step(session, b'\x03\x11', b'Local Org tags: ')
    expected = LINES.copy()
    expected[6] = '** TODO Child :local:new:'
    send(session, b'local:new\r', lambda frame: exact(frame, expected) and b'inherited owners retained' in frame)
    unchanged()
    goto(session, 7)
    step(session, b'\x03\x03', b'Local Org tags: ')
    step(session, b'week\t', b'1 tag candidates: weekend')
    expected[6] = '** TODO Child :weekend:'
    send(session, b'\r', lambda frame: exact(frame, expected) and b'inherited owners retained' in frame)
    unchanged()
    goto(session, 7)
    for query, expected in [('(Work|home)+local/!TODO', b'Org match: yes'),
                            ('PROJECT="root"+File+LEVEL=2', b'Org match: yes'),
                            ('work', b'Org match: no')]:
        named('org-match-tags', b'Org tag/property match: ')
        send(session, query.encode() + b'\r', lambda frame:
             expected in frame and exact(frame, LINES) and b'-- RW --' in frame)
    named('org-match-tags', b'Org tag/property match: ')
    send(session, b'Work|\r', lambda frame: b'Org match at byte' in frame and exact(frame, LINES))
    named('org-remove-tag', b'Remove local Org tag: ')
    send(session, b'Work\r', lambda frame: exact(frame, LINES) and b'inherited owners retained' in frame)
    step(session, b'\x03\x11', b'Local Org tags: ')
    step(session, b'cancelled', b'Local Org tags: cancelled')
    send(session, b'\x07', lambda frame: exact(frame, LINES) and b'Quit' in frame)
    step(session, b'\x18\x13', b'Saved')
    assert source.read_bytes() == SOURCE, 'tag operations changed BOM/CRLF/EOF bytes after undo'
