"""Independent ZIP/CRC, XML, relationship and native document assertions."""
from io import BytesIO
from pathlib import Path
import struct
import sys
import zipfile
import xml.etree.ElementTree as E

NS = {key: 'urn:oasis:names:tc:opendocument:xmlns:' + suffix for key, suffix in
      [('office', 'office:1.0'), ('text', 'text:1.0'), ('style', 'style:1.0'),
       ('table', 'table:1.0'), ('manifest', 'manifest:1.0')]}
NS.update(dc='http://purl.org/dc/elements/1.1/', xlink='http://www.w3.org/1999/xlink')
MIME = b'application/vnd.oasis.opendocument.text'
MEMBERS = ['mimetype', 'content.xml', 'styles.xml', 'meta.xml', 'META-INF/manifest.xml']


def key(prefix, name):
    return '{' + NS[prefix] + '}' + name


def text(node):
    result = node.text or ''
    for child in node:
        if child.tag == key('text', 's'):
            result += ' ' * int(child.get(key('text', 'c'), '1'))
        elif child.tag == key('text', 'tab'):
            result += '\t'
        elif child.tag == key('text', 'line-break'):
            result += '\n'
        else:
            result += text(child)
        result += child.tail or ''
    return result


class Document:
    def __init__(self, data):
        assert len(data) <= 134217728
        with zipfile.ZipFile(BytesIO(data)) as archive:
            assert archive.namelist() == MEMBERS
            infos = archive.infolist()
            assert all(i.compress_type == zipfile.ZIP_STORED and not i.flag_bits
                       and not i.extra and not i.comment for i in infos)
            assert all(i.file_size == i.compress_size for i in infos)
            assert sum(i.file_size for i in infos) <= len(data)
            assert infos[0].header_offset == 0 and archive.comment == b''
            # Inspect actual local header bytes independently of ZIP metadata.
            assert data[:4] == b'PK\x03\x04'
            assert struct.unpack_from('<HH', data, 26) == (8, 0)
            assert data[30:38] == b'mimetype' and data[38:38 + len(MIME)] == MIME
            assert archive.testzip() is None and archive.read('mimetype') == MIME
            parts = {name: E.fromstring(archive.read(name)) for name in MEMBERS[1:]}
        self.content, self.styles, self.meta = (parts[n] for n in MEMBERS[1:4])
        for name, root in zip(['document-content', 'document-styles', 'document-meta'],
                              [self.content, self.styles, self.meta]):
            assert root.tag == key('office', name)
            assert root.get(key('office', 'version')) == '1.3'
        manifest = parts[MEMBERS[-1]]
        assert manifest.tag == key('manifest', 'manifest')
        entries = [(n.get(key('manifest', 'full-path')),
                    n.get(key('manifest', 'media-type'))) for n in manifest]
        assert len(entries) == 4
        assert dict(entries) == {'/': MIME.decode(), 'content.xml': 'text/xml',
                                 'styles.xml': 'text/xml', 'meta.xml': 'text/xml'}
        self.relationships()

    def texts(self, tag):
        return [text(n) for n in self.content.findall('.//' + tag, NS)]

    def relationships(self):
        bookmarks = [n.get(key('text', 'name')) for n in
                     self.content.findall('.//text:bookmark', NS)]
        assert len(bookmarks) == len(set(bookmarks))
        notes = [n.get(key('text', 'id')) for n in
                 self.content.findall('.//text:note', NS)]
        assert len(notes) == len(set(notes))
        for n in self.content.findall('.//text:note-ref', NS):
            assert n.get(key('text', 'ref-name')) in notes
        for n in self.content.findall('.//text:a', NS):
            href = n.get(key('xlink', 'href'))
            assert n.get(key('xlink', 'type')) == 'simple' and href
            if href.startswith('#'):
                assert href[1:] in bookmarks
            assert not href.lower().startswith(('javascript:', 'data:', 'shell:'))
        styles = {n.get(key('style', 'name')): n.get(key('style', 'family'))
                  for n in self.styles.findall('.//style:style', NS)}
        lists = {n.get(key('style', 'name')) for n in
                 self.styles.findall('.//text:list-style', NS)}
        for n in self.content.iter():
            name = n.get(key('text', 'style-name'))
            if name:
                if n.tag == key('text', 'list'):
                    assert name in lists
                else:
                    assert styles.get(name) == ('text' if n.tag == key('text', 'span')
                                                else 'paragraph')
        allowed = {'office': ['document-content', 'body', 'text'],
                   'text': ['p', 'h', 'span', 's', 'tab', 'line-break', 'list', 'list-item',
                            'a', 'bookmark', 'note', 'note-body', 'note-citation', 'note-ref'],
                   'table': ['table', 'table-column', 'table-row', 'table-cell']}
        tags = {key(p, name) for p, names in allowed.items() for name in names}
        assert all(n.tag in tags for n in self.content.iter())


def verify(document):
    assert document.meta.find('.//dc:title', NS).text == 'Notes & Plans 界'
    assert document.meta.find('.//dc:creator', NS).text == 'Writer'
    headings = document.content.findall('.//text:h', NS)
    assert [text(n) for n in headings] == ['Forms 界', 'Detail']
    assert [n.get(key('text', 'outline-level')) for n in headings] == ['1', '2']
    assert all(n.get(key('text', 'is-list-header')) == 'true' for n in headings)
    rows = [[text(c) for c in n.findall('table:table-cell', NS)] for n in
            document.content.findall('.//table:table-row', NS)]
    assert rows == [['Name', 'Count'], ['A&B', '2_3']]
    lists = document.content.findall('.//text:list', NS)
    assert [n.get(key('text', 'style-name')) for n in lists] == ['BulletList', 'NumberList']
    assert [n.get(key('text', 'start-value')) for n in lists[1]] == ['3', '4']
    links = {n.get(key('xlink', 'href')): text(n) for n in
             document.content.findall('.//text:a', NS)}
    assert links['https://example.test/a?x=1&y=2'] == 'site'
    assert any(v == 'target' and k.startswith('#') for k, v in links.items())
    assert document.texts('text:note-citation') == ['1', '2']
    assert any('A note.' in value for value in document.texts('text:note-body'))
    assert document.texts('text:note-ref') == ['1']
    body = text(document.content)
    for value in ['Body bold and italic.', 'α é 界', 'two  spaces\tend',
                  '\\write18{touch marker}', '<script>unsafe()</script>',
                  '[[file:images/missing.png]]', 'Caption & data']:
        assert value in body, (value, body)


if __name__ == '__main__':
    document = Document(Path(sys.argv[1]).read_bytes())
    if len(sys.argv) > 2:
        assert sys.argv[2] == '--fixture'
        verify(document)
