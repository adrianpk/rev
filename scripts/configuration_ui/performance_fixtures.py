"""Deterministic bounded configuration, directory and Unicode workloads."""

import hashlib
from pathlib import Path


SETTINGS = """editor.tab_width = 8
history.group_limit = 2
history.payload_target_bytes = 4096
kill_ring.capacity = 2
tree.visible = true
tree.width = 18
tree.root = 'file-parent'
minibuffer.candidate_limit = 7
syntax.max_bytes = 65536
"""


def graph(root):
    directory = root / 'user/rev'
    directory.mkdir(parents=True)
    sources = {'config.toml': SETTINGS + "include = ['node-1.toml']\n"}
    for index in range(1, 15):
        sources[f'node-{index}.toml'] = f"include = ['node-{index + 1}.toml']\n"
    leaves = ','.join(f"'leaf-{index}.toml'" for index in range(48))
    sources['node-15.toml'] = f'include = [{leaves}]\n'
    overhead = sum(len(text.encode()) for text in sources.values())
    for index in range(48):
        length = 262144 if index < 7 else (262144 - overhead if index == 7 else 0)
        sources[f'leaf-{index}.toml'] = ' ' * length
    assert len(sources) == 64
    assert sum(len(text.encode()) for text in sources.values()) == 2097152
    for name, text in sources.items():
        (directory / name).write_text(text)
    return {'sources': 64, 'edges': 16, 'total_bytes': 2097152, 'max_source_bytes': 262144}


def candidates(root, kind):
    directory = root / kind
    directory.mkdir()
    (directory / 'nested').mkdir()
    (directory / 'nested/hidden').write_text('must not be observed')
    (directory / 'alias').symlink_to('nested', target_is_directory=True)
    count = 4097 if kind == 'entries' else 2100
    for index in range(count):
        name = f'{index:04}' + ('界' * 83 if kind == 'names' else '')
        (directory / name).touch()
    return {'disk_entries': count + 2, 'entry_limit': 4096, 'name_byte_limit': 524288}


def documents(root):
    directory = root / 'work'
    directory.mkdir()
    text = ''.join(f'\tfn item_{index}() {{ let label = "界e\u0301🙂👩\u200d💻"; }}\n' for index in range(512))
    (directory / 'main.rs').write_text(text)
    code = b'fn item() { let value = 42; }\n'
    content = code * (65536 // len(code))
    (directory / 'cutoff.rs').write_bytes(content + b' ' * (65536 - len(content)))
    return {'panel_lines': 512, 'panel_bytes': len(text.encode()), 'syntax_cutoff_bytes': 65536}


def fixture(root):
    return {'graph': graph(root), 'entries': candidates(root, 'entries'),
            'names': candidates(root, 'names'), 'documents': documents(root)}


def fingerprint(root):
    digest = hashlib.sha256()
    count = 0
    for path in sorted(root.rglob('*')):
        count += 1
        assert count <= 6400, 'fixture entry bound exceeded'
        digest.update(str(path.relative_to(root)).encode())
        if path.is_symlink():
            digest.update(path.readlink().as_posix().encode())
        elif path.is_file():
            assert path.stat().st_size <= 262144
            digest.update(path.read_bytes())
    return {'entries': count, 'sha256': digest.hexdigest()}
