"""Independent HTML parsing for native exports, without renderer helpers."""
from html.parser import HTMLParser
from pathlib import Path
import sys

VOID = {'meta', 'img'}
ALLOWED = set('html head meta title body main h1 h2 h3 h4 h5 h6 p ul ol li a '
              'nav strong em u del code pre blockquote div table tr td caption '
              'figure figcaption span sup section img th'.split())
BLOCK = set('h1 h2 h3 h4 h5 h6 p ul ol li nav pre blockquote div table tr '
            'figure section'.split())


class Document(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.stack = []
        self.nodes = []
        self.active = []
        self.ids = set()
        self.links = []
        self.doctype = False

    def handle_decl(self, decl):
        assert decl.lower() == 'doctype html' and not self.doctype
        self.doctype = True

    def handle_starttag(self, tag, attrs):
        assert tag in ALLOWED, ('unexpected active element', tag)
        values = dict(attrs)
        assert len(values) == len(attrs), ('duplicate attributes', tag)
        assert all(not key.lower().startswith('on') and key not in {'style', 'srcdoc'}
                   for key in values), values
        assert not (tag == 'a' and 'a' in self.stack), 'nested anchor'
        assert not (tag in BLOCK and 'p' in self.stack), ('block inside paragraph', tag)
        if tag in {'td', 'th'}:
            assert self.stack[-1] == 'tr'
        if tag == 'tr':
            assert self.stack[-1] == 'table'
        if tag == 'caption':
            assert self.stack[-1] == 'table'
        if tag == 'figcaption':
            assert self.stack[-1] == 'figure'
        if 'id' in values:
            assert values['id'] and values['id'] not in self.ids, values['id']
            self.ids.add(values['id'])
        if tag == 'a':
            href = values.get('href', '')
            assert href and not href.lower().startswith(('javascript:', 'data:', 'vbscript:'))
            self.links.append(href)
        if tag == 'img':
            src = values['src']
            assert src and not src.startswith('/') and ':' not in src
            assert '..' not in src.split('/') and 'alt' in values
        self.nodes.append((tag, values, []))
        if tag not in VOID:
            self.stack.append(tag)
            self.active.append(self.nodes[-1])

    def handle_endtag(self, tag):
        assert self.stack and self.stack.pop() == tag, ('unbalanced element', tag)
        self.active.pop()

    def handle_data(self, data):
        for _, _, text in self.active:
            text.append(data)

    def finish(self):
        self.close()
        assert self.doctype and not self.stack, self.stack
        assert [n[0] for n in self.nodes].count('html') == 1
        assert [n[0] for n in self.nodes].count('head') == 1
        assert [n[0] for n in self.nodes].count('body') == 1
        assert [n[0] for n in self.nodes].count('main') == 1
        for href in self.links:
            if href.startswith('#'):
                assert href[1:] in self.ids, ('missing local target', href)
        return self

    def texts(self, tag):
        return [''.join(text).strip() for name, _, text in self.nodes if name == tag]


def parse(data):
    assert len(data) <= 16777216
    result = Document()
    result.feed(data.decode('utf-8', errors='strict'))
    return result.finish()


def full(data):
    document = parse(data)
    assert document.texts('title') == ['Field & Notes']
    assert document.texts('h1') == ['Field & Notes', '1 TODO Overview n :work:', '2 Details']
    assert document.texts('th') == ['Name', 'Count', 'ITEM', 'EFFORT']
    assert document.texts('td') == ['Blue', '2', '界', '3', 'Recorded', '1:00']
    assert any(tag == 'ol' and attrs.get('start') == '3' for tag, attrs, _ in document.nodes)
    metadata = {a['name']: a['content'] for tag, a, _ in document.nodes
                if tag == 'meta' and 'name' in a}
    assert metadata == {'author': 'Example Writer', 'email': 'writer@example.test',
                        'date': '2026-01-02'}
    assert document.nodes[0][1] == {'lang': 'en'}
    images = [a for tag, a, _ in document.nodes if tag == 'img']
    assert images == [{'alt': 'Icon & mark', 'height': '50%', 'id': 'picture',
                       'loading': 'lazy', 'src': 'images/picture.png', 'width': '320'}]
    assert 'https://example.test/a?x=1&y=2' in document.links
    assert 'https://example.test/note' in document.links
    code = document.texts('code')
    assert 'touch marker\n<script>alert("source")</script>' in code
    assert '<script>alert("export")</script>' in code
    assert 'totals' in document.texts('strong')
    assert 'body' in document.texts('strong') and 'meaning' in document.texts('strong')
    assert 'Quoted' in ' '.join(document.texts('p'))
    return document


if __name__ == '__main__':
    full(Path(sys.argv[1]).read_bytes())
    print('HTML semantic fixture: independently parsed structure, metadata, cells, assets, notes and inert code passed.')
