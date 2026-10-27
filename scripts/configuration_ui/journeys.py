"""Connected startup, command and file-prompt journeys on real terminals."""

from .support import BINARY, CONFIG, LIST, PTY, canvas, cursor, environment, finish, fixtures, passive, query, rows, snapshot, step, visit, write


def commands(session, colored):
    frame = step(session, b'\x1bxforward-', b'M-x forward-')
    LIST['ordered'](frame, b'M-x forward-', [b'forward-char', b'forward-word'])
    LIST['selected_style'](frame, b'forward-char', colored)
    query(frame, 'M-x forward-')
    frame = step(session, b'\x1bj\x1bc', b'M-x forward-word')
    LIST['selected_style'](frame, b'forward-word', colored)
    frame = step(session, b'\x1ba', b'> main.rs -- RW L1 C12 Rust')
    assert cursor(frame) == (1, 12)
    step(session, b'\x1bxdraft', b'M-x draft')
    step(session, b'\x1bp', b'M-x forward-word')
    frame = step(session, b'\x1bn', b'M-x draft')
    query(frame, 'M-x draft')
    step(session, b'\x07', b'Quit')
    frame = step(session, b'\x1b<', b'> main.rs -- RW L1 C1 Rust')
    assert cursor(frame) == (1, 1)


def paths(session, source):
    step(session, b'\x1bxfind-file\x1ba', b'Find file: ')
    frame = step(session, PTY['paste_wire'](b'space d') + b'\x1bc', b'Find file: space dir/')
    LIST['ordered'](frame, b'Find file: space dir/', ['界 name.txt'.encode()])
    query(frame, 'Find file: space dir/')
    frame = step(session, b'\x1bk\x1ba', b'> ' + '界 name.txt -- RW L1 C1 Text'.encode())
    assert canvas(frame)[1].startswith('chosen 界e\u0301')
    visit(session, '../main.rs', b'> main.rs -- RW L1 C1 Rust')
    step(session, b'\x18\x06', b'Find file: ')
    frame = step(session, b'al', b'Find file: al')
    LIST['ordered'](frame, b'Find file: al', [b'alpha.txt', b'alpine.txt'])
    assert source.read_bytes().startswith(b'\tlet label')


def configured(root, colored):
    source = fixtures(root)
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)], env=environment(root, colored))
    try:
        session.ready(b'> main.rs -- RW L1 C1 Rust')
        frame = PTY['last_frame'](session)
        assert canvas(frame)[1].startswith('        let label = "界e\u0301";')
        assert (b'\x1b]12;#112233\x07' in session.output) == colored
        assert (b'\x1b[38;2;1;2;3m' in frame) == colored
        frame = step(session, b'\x06', b'> main.rs -- RW L1 C9 Rust')
        assert cursor(frame) == (1, 9)
        step(session, b'\x1b<', b'> main.rs -- RW L1 C1 Rust')
        commands(session, colored)
        paths(session, source)
        assert snapshot(root) == before, 'completion/observation changed disk'
        step(session, b'\r', b'> al -- RW L1 C1 Text')
        target = root / 'work/al'
        assert not target.exists(), 'visit created a file'
        frame = step(session, b'\t' + 'é'.encode() + b'\x1bj', b'> al ** RW L2 C1 Text')
        assert canvas(frame)[1].startswith('  é') and cursor(frame) == (2, 1)
        assert not target.exists(), 'editing published a save'
        step(session, b'\x18\x13', b'Saved ')
        assert target.read_bytes() == b'\xef\xbb\xbf  \xc3\xa9\r\n'
        finish(session, colored)
    finally:
        session.close()
    expected = dict(before, **{'work/al': b'\xef\xbb\xbf  \xc3\xa9\r\n'})
    assert snapshot(root) == expected
    passive(root)
    print(f"PASS connected {'RGB' if colored else 'NO_COLOR'} startup, tab/key policies, M-x selection/history, Unicode path completion, new draft and explicit format save")


def fallback(root, rejected):
    source = fixtures(root, False)
    if rejected:
        write(root, 'user/rev/config.toml', CONFIG.replace('editor.tab_width = 8', 'editor.tab_width = 0').encode())
    before = snapshot(root)
    session = PTY['Session']([str(BINARY), str(source)], env=environment(root))
    try:
        session.ready(b'Configuration:' if rejected else b'> main.rs -- RW L1 C1 Rust')
        frame = PTY['last_frame'](session)
        assert canvas(frame)[1].startswith('    let label')
        assert b'\x1b[38;2;1;2;3m' not in frame
        assert (b'Configuration:' in frame) == rejected
        frame = step(session, b'\x06', b'> main.rs -- RW L1 C5 Rust')
        assert cursor(frame) == (1, 5)
        step(session, b'\x1bj', b'Unbound key')
        visit(session, 'fallback-new', b'> fallback-new -- RW L1 C1 Text')
        assert snapshot(root) == before
        step(session, b'\tQ\r', b'> fallback-new ** RW L2 C1 Text')
        assert not (root / 'work/fallback-new').exists()
        step(session, b'\x18\x13', b'Saved ')
        finish(session)
    finally:
        session.close()
    expected = dict(before, **{'work/fallback-new': b'    Q\n'})
    assert snapshot(root) == expected
    passive(root)
    assert (root / 'user').exists() == rejected
    print(f"PASS {'invalid configuration rollback' if rejected else 'silent missing configuration'} with visible default tab/key/palette behavior, new-file formats and passive XDG roots")
