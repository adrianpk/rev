"""Independent admission of native Texinfo/man structure; never execute data."""
import re

TEX_COMMANDS = set('documentencoding setfilename settitle node top contents chapter section subsection subsubsection unnumbered unnumberedsec unnumberedsubsec unnumberedsubsubsec example quotation itemize bullet enumerate item end strong emph code anchor ref url comma backslashchar bye'.split())


def texinfo(data):
    text = data.decode('utf-8')
    assert text.startswith('\\input texinfo\n'), text
    assert text.endswith('@bye\n'), text
    assert text.count('\\input') == 1
    depth = 0
    at = 0
    while at < len(text):
        char = text[at]
        if char == '@':
            at += 1
            if text[at] in '@{}':
                at += 1
                continue
            match = re.match('[a-z]+', text[at:])
            assert match and match[0] in TEX_COMMANDS, text[at:at+80]
            at += len(match[0])
            continue
        if char == '{':
            depth += 1
        elif char == '}':
            depth -= 1
            assert depth >= 0
        at += 1
    assert depth == 0
    names = re.findall(r'^@node ([A-Za-z0-9-]+)$', text, re.M)
    anchors = re.findall(r'@anchor\{([A-Za-z0-9-]+)\}', text)
    assert names and names[0] == 'Top'
    assert len(names + anchors) == len(set(names + anchors))
    refs = re.findall(r'@ref\{([A-Za-z0-9-]+),,', text)
    assert set(refs) <= set(names + anchors), (refs, names, anchors)
    stack = []
    for line in text.splitlines():
        match = re.match(r'^@(example|quotation|itemize|enumerate)(?: |$)', line)
        if match:
            stack.append(match[1])
        elif line.startswith('@end '):
            assert stack and stack.pop() == line[5:], line
    assert not stack
    return text


def man(data):
    text = data.decode('utf-8')
    assert text.startswith('.TH "'), text
    stack = []
    literal = False
    for line in text.splitlines():
        assert not line.startswith("'"), line
        if not line.startswith('.'):
            continue
        name = line.split(maxsplit=1)[0]
        assert name in {'.TH', '.SH', '.SS', '.PP', '.RS', '.RE', '.IP', '.nf', '.fi'}, line
        if name == '.RS':
            stack.append(name)
        elif name == '.RE':
            assert stack and stack.pop() == '.RS'
        elif name == '.nf':
            assert not literal
            literal = True
        elif name == '.fi':
            assert literal
            literal = False
        if name in {'.TH', '.SH', '.SS', '.IP'}:
            assert line.count('"') == (10 if name == '.TH' else 2), line
    assert not literal and not stack
    plain = re.sub(r'\\(?:\[(?:rs|dq|aq)\]|&\.|f(?:B|I|P|\(CW))', '', text)
    assert '\\' not in plain, plain
    return text


def fixture(data, kind):
    text = {'texinfo': texinfo, 'man': man}[kind](data)
    for expected in ['Start', 'Child', '| x | y |', 'three', 'four', 'secret', 'touch marker', 'escape']:
        assert expected in text, (kind, expected)
    return text


if __name__ == '__main__':
    import sys
    from pathlib import Path
    fixture(Path(sys.argv[1]).read_bytes(), sys.argv[2])
