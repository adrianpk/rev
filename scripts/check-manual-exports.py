#!/usr/bin/env python3
"""Explicit optional GNU formatter check of exact renderer golden documents."""
import re
import shutil
import subprocess
import tempfile
from pathlib import Path
from configuration_ui.manual_document import fixture

ROOT = Path(__file__).resolve().parents[1]
GOLDEN = ROOT / 'tests/fixtures/org-manual'


def run(argv, root):
    result = subprocess.run(argv, cwd=root, capture_output=True, timeout=10, check=False)
    assert len(result.stdout) + len(result.stderr) <= 1048576, 'Formatter output limit'
    assert result.returncode == 0, result.stderr.decode(errors='replace')
    assert not result.stderr, result.stderr.decode(errors='replace')
    return result.stdout


def main():
    tools = {name: shutil.which(name) for name in ['makeinfo', 'groff']}
    assert all(tools.values()), 'Explicit check requires makeinfo and groff'
    with tempfile.TemporaryDirectory(prefix='rev-manual-check-') as directory:
        root = Path(directory)
        for name, kind in [('document.texi', 'texinfo'), ('document.1', 'man')]:
            data = (GOLDEN / name).read_bytes()
            assert len(data) <= 65536
            fixture(data, kind)
            (root / name).write_bytes(data)
        run([tools['makeinfo'], '--no-split', '--output', str(root / 'manual.info'), str(root / 'document.texi')], root)
        info = (root / 'manual.info').read_bytes()
        assert len(info) <= 1048576
        # Info uses control separators; visible fields must retain node and link
        # relationships, declared hierarchy and inert formatter-looking data.
        for expected in [b'Node: Top', b'Up: Top', b'Field @manual', b'Child', b'3. three', b'4. four', b'@include secret', b'.sy touch marker', b"\\X'escape'", b'A _note_.', b'inline *body*']:
            assert expected in info, (expected, info)
        output = run([tools['groff'], '-S', '-Kutf8', '-Tutf8', '-man', str(root / 'document.1')], root)
        # GNU terminal output may use SGR or overstrikes for font selection.
        output = re.sub(rb"\x1b\[[0-9;]*m", b"", output)
        while b'\x08' in output:
            output = re.sub(rb'.\x08', b'', output)
        for expected in [b'Field @manual(1)', b'Start', b'Child', b'@include secret', b'.sy touch marker', b"\\X'escape'", b'inline body']:
            assert expected in output, (expected, output)
        for number, word in [(3, b'three'), (4, b'four')]:
            assert re.search(str(number).encode() + rb'\. +' + word, output), output
        assert set(path.name for path in root.iterdir()) == {'document.texi', 'document.1', 'manual.info'}
    print('Manual exports: exact goldens accepted by makeinfo and groff; hierarchy, references, notes and inert source verified; zero skipped checks')


if __name__ == '__main__':
    main()
