"""Owned physical export prompts, exact output bytes and source preservation."""
from .org_document import encoded, exact
from .org_tasks import goto, named
from .org_time import quiet
from .html_document import parse
from .odt_document import Document as OdtDocument
from .manual_document import texinfo, man
from .support import (BINARY, PTY, close, completed, environment, finish,
                      send, snapshot, step, write)

LINES = ['* Heading 界', 'Body', 'EOF']
MARKDOWN = '# Heading 界\nBody unsaved\nEOF'.encode()


def choose(session, command, path, marker):
    named(session, command, b'Export scope')
    step(session, b'buffer\r', b'Export OPTIONS')
    step(session, b'toc:nil num:nil\r', b'Export to local file: ')
    return step(session, path.encode() + b'\r', marker)


def journey(root, colored):
    write(root, 'user/rev/config.toml', b'org.time.zone="UTC"\n')
    source = write(root, 'work/notes.org', encoded(LINES))
    write(root, 'work/keep.txt', b'keep')
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)],
                             env=environment(root, colored), cwd=source.parent)
    try:
        session.ready(b'> notes.org -- RW -- L1 C1')
        session.wait(lambda: completed(session, lambda f: exact(f, LINES)))
        quiet(session)
        goto(session, 2)
        step(session, b'\x05', b'L2 C5')
        current = LINES.copy()
        current[1] += ' unsaved'
        send(session, PTY['paste_wire'](b' unsaved'), lambda f: exact(f, current))
        choose(session, 'org-export-to-markdown', 'output.md', b'Exported Org document')
        assert (source.parent / 'output.md').read_bytes() == MARKDOWN
        assert source.read_bytes() == encoded(LINES)
        choose(session, 'org-export-to-html', 'output.html', b'Exported Org document')
        html_bytes = (source.parent / 'output.html').read_bytes()
        document = parse(html_bytes)
        assert document.texts('h1') == ['Heading 界']
        assert document.texts('p') == ['Body unsaved\nEOF']
        assert source.read_bytes() == encoded(LINES)
        choose(session, 'org-export-to-latex', 'article.tex', b'Exported Org document')
        article = (source.parent / 'article.tex').read_bytes()
        assert article.startswith(b'\\documentclass{article}\n')
        assert b'\\section*{Heading ' in article and b'Body unsaved\nEOF' in article
        assert article.endswith(b'\\end{document}\n')
        choose(session, 'org-export-to-beamer', 'slides.tex', b'Exported Org document')
        slides = (source.parent / 'slides.tex').read_bytes()
        assert slides.startswith(b'\\documentclass{beamer}\n')
        assert b'\\begin{frame}[fragile]{Heading ' in slides
        assert slides.count(b'\\begin{frame}') == slides.count(b'\\end{frame}') == 1
        choose(session, 'org-export-to-latex-pdf', 'missing.pdf', b'No PDF compiler configured')
        assert not (source.parent / 'missing.pdf').exists()
        assert source.read_bytes() == encoded(LINES)
        choose(session, 'org-export-to-odt', 'document.odt', b'Exported Org document')
        odt_bytes = (source.parent / 'document.odt').read_bytes()
        odt = OdtDocument(odt_bytes)
        assert odt.texts('text:h') == ['Heading 界']
        assert odt.texts('text:p') == ['Body unsaved\nEOF']
        assert source.read_bytes() == encoded(LINES)
        choose(session, 'org-export-to-texinfo', 'manual.texi', b'Exported Org document')
        texi_bytes = (source.parent / 'manual.texi').read_bytes()
        texi = texinfo(texi_bytes)
        assert '@unnumbered Heading 界' in texi and 'Body unsaved\nEOF' in texi
        choose(session, 'org-export-to-man', 'manual.1', b'Exported Org document')
        man_bytes = (source.parent / 'manual.1').read_bytes()
        manual = man(man_bytes)
        assert '.SH "Heading 界"' in manual and 'Body unsaved\nEOF' in manual
        assert source.read_bytes() == encoded(LINES)
        choose(session, 'org-export-to-text', 'ascii.txt', b'U+754C')
        assert not (source.parent / 'ascii.txt').exists()
        choose(session, 'org-export-to-unicode', 'keep.txt', b'yes/no:')
        frame = step(session, b'no\r', b'Org export declined')
        assert exact(frame, current)
        assert (source.parent / 'keep.txt').read_bytes() == b'keep'
        named(session, 'org-export-to-org', b'Export scope')
        frame = step(session, b'\x07', b'Org export cancelled')
        assert exact(frame, current)
        send(session, b'\x18u', lambda f: exact(f, LINES))
        assert source.read_bytes() == encoded(LINES)
        quiet(session)
        if not colored:
            assert b'\x1b[38;2;' not in session.output and b'\x1b[48;2;' not in session.output
        finish(session, colored)
        before['work/output.md'] = MARKDOWN
        before['work/output.html'] = html_bytes
        before['work/article.tex'] = article
        before['work/slides.tex'] = slides
        before['work/document.odt'] = odt_bytes
        before['work/manual.texi'] = texi_bytes
        before['work/manual.1'] = man_bytes
        assert snapshot(root) == before
    finally:
        close(session)
