#!/usr/bin/python3
"""Exercise the shipped resource in actual offline Pi; no provider prompts."""
import json
import os
from pathlib import Path
import selectors
import subprocess
import sys
import tempfile
import time

exe = str(Path(sys.argv[1]).resolve(strict=True))
probe = Path(__file__).with_name('discovery-probe.js').resolve()
flags = ['--mode', 'rpc', '--no-session', '--tools', 'rev_read,rev_list,rev_search',
         '--extension', str(probe), '--no-extensions', '--no-mcp', '--no-skills',
         '--no-prompt-templates', '--no-context-files', '--no-themes', '--no-approve', '--offline']
assert subprocess.run([exe, '--offline', '--version'], capture_output=True, timeout=5, check=True).stdout.strip() == b'1.3.0'

class Pi:
    def __init__(self, root, snapshots=(), provider=False):
        local_flags = [*flags, '--extension', str(probe.with_name('discovery-provider.js')), '--model', 'rev-fixture/local'] if provider else flags
        self.child = subprocess.Popen([exe, *local_flags], cwd=root, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.streams = selectors.DefaultSelector()
        self.streams.register(self.child.stdout, selectors.EVENT_READ, 'out')
        self.streams.register(self.child.stderr, selectors.EVENT_READ, 'err')
        self.buffer, self.stderr, self.counter = bytearray(), bytearray(), 0
        try:
            st = root.stat()
            grant = dict(version=1, nonce=1, root=str(root), dev=str(st.st_dev), ino=str(st.st_ino), timeout_ms=15000, max_input_bytes=4194304, private_roots=[str(root/'private-store')])
            records = self.command('/rev-discovery ' + json.dumps(grant), 'rev-discovery-v1')
            admission = [r for r in records if r.get('type') == 'extension_ui_request'][0]
            data = json.loads(admission['message'])
            assert data['admitted'] and sorted(data['tools']) == ['rev_list', 'rev_read', 'rev_search'], data
            assert [r for r in records if r['type'] == 'response'][0]['data']['disposition'] == 'handled'
            state = self.command('', 'state', kind='get_state')[0]['data']
            assert state['messageCount'] == state['pendingMessageCount'] == 0 and not state.get('sessionFile')
            if not provider: assert self.probe(envelope=dict(dirty_sources=list(snapshots)))['installed']
        except BaseException:
            self.abort()
            raise
    def command(self, message, notification, kind='prompt'):
        self.counter += 1
        self.child.stdin.write(json.dumps(dict(type=kind,id=str(self.counter),**(dict(message=message) if kind=='prompt' else {}))).encode() + b'\n')
        self.child.stdin.flush()
        records = []
        total = 0
        end = time.monotonic() + 5
        while not (any(r.get('type') == 'response' and r.get('id') == str(self.counter) for r in records) and (True if notification == 'state' else any(r.get('type') == 'agent_settled' for r in records) if notification is None else any(r.get('type') == 'extension_ui_request' and json.loads(r['message']).get('type') == notification for r in records))):
            assert time.monotonic() < end, ('offline command deadline', len(records), self.stderr[-1024:].decode(errors='replace'))
            for key, _ in self.streams.select(.05):
                data = os.read(key.fd, 65536)
                assert data, ('Pi stream closed', len(records), self.stderr[-1024:].decode(errors='replace'))
                if key.data == 'err':
                    self.stderr.extend(data); del self.stderr[:-65536]
                else:
                    self.buffer.extend(data)
                    assert len(self.buffer) <= 2 * 1024 * 1024
            while b'\n' in self.buffer:
                raw, _, rest = self.buffer.partition(b'\n'); self.buffer = bytearray(rest)
                total += len(raw)
                assert total <= 4194304, 'offline record budget'
                records.append(json.loads(raw))
        return records
    def probe(self, **value):
        records = self.command('/rev-probe ' + json.dumps(value), 'rev-probe-result')
        return json.loads(next(r['message'] for r in records if r.get('type') == 'extension_ui_request'))['result']
    def tool(self, name, **args):
        result = self.probe(tool=name, args=args)
        if result.get('isError'): return result
        return json.loads(result['content'][0]['text'])
    def close(self):
        self.child.stdin.close()
        self.child.wait(timeout=2)
        assert self.child.returncode == 0
        self.streams.close()
        self.child.stdout.close(); self.child.stderr.close()
    def abort(self):
        if self.child.poll() is None: self.child.kill(); self.child.wait(timeout=2)

checks = 0
with tempfile.TemporaryDirectory(prefix='rev-pi-discovery-') as temporary:
    base = Path(temporary); root = base/'project'; root.mkdir()
    outside = base/'outside'; outside.mkdir(); (outside/'secret').write_text('OUTSIDE-SECRET')
    (root/'main.zig').write_text('stale disk\n')
    (root/'module.zig').write_text('const answer = 42;\n')
    (root/'unicode').write_bytes('é\r\n👩\u200d💻\u2028end\n'.encode())
    (root/'binary').write_bytes(b'hello\0secret')
    (root/'invalid').write_bytes(b'\xff')
    (root/'large').write_bytes(b'x'*(1048576+1))
    (root/'escape').symlink_to(outside, target_is_directory=True)
    (root/'private-store').mkdir(); (root/'private-store'/'opaque').write_text('PRIVATE-STORE-SECRET')
    (root/'.env').write_text('CREDENTIAL-SECRET')
    (root/'.pi').mkdir(); (root/'.pi'/'settings.json').write_text('{"defaultTools":["bash","write"]}')
    (root/'AGENTS.md').write_text('RESOURCE-INSTRUCTION-MUST-NOT-LOAD')
    (root/'.pi'/'extensions').mkdir(); (root/'.pi'/'extensions'/'bad.js').write_text('throw new Error("EXTERNAL-RESOURCE-LOADED")')
    (root/'pipe').touch(); (root/'pipe').unlink(); os.mkfifo(root/'pipe')
    (root/'hard').hardlink_to(outside/'secret')
    snapshots = [dict(path='main.zig', text='fresh unsaved needle\n', buffer_id='native', revision=7, complete=True, bom=False)]
    pi = Pi(root, snapshots)
    try:
        read = pi.tool('rev_read', path='main.zig')
        assert not read.get('isError'), read
        assert read['text'] == 'fresh unsaved needle\n' and read['revision'] == 7 and read['provenance'] == 'native editor snapshot'; checks += 1
        listed = pi.tool('rev_list', recursive=True, match='.zig')
        assert {v['path'] for v in listed['items']} == {'main.zig','module.zig'}; checks += 1
        search = pi.tool('rev_search', query='needle')
        assert len(search['matches']) == 1 and search['matches'][0]['provenance'] == 'native editor snapshot'; checks += 1
        assert pi.tool('rev_search', query='stale disk')['matches'] == []; checks += 1
        (root/'main.zig').unlink()
        assert pi.tool('rev_read', path='main.zig')['text'] == 'fresh unsaved needle\n'; checks += 1
        (root/'main.zig').write_text('stale disk\n')
        assert not pi.tool('rev_list', limit=1)['complete']; checks += 1
        assert pi.tool('rev_read', path='unicode')['text'] == 'é\r\n👩\u200d💻\u2028end\n'; checks += 1
        for path in ['../outside/secret', str(outside/'secret'), 'escape/secret', '.env', '.pi/settings.json', 'binary', 'invalid', 'large', 'missing', 'pipe', 'hard', 'private-store/opaque']:
            result = pi.tool('rev_read', path=path)
            assert result.get('isError'), (path, result)
            assert 'OUTSIDE-SECRET' not in json.dumps(result) and 'CREDENTIAL-SECRET' not in json.dumps(result); checks += 1
        # A pinned open file is swapped after admission, before its first read.
        race = pi.probe(tool='rev_read', args=dict(path='module.zig'), race=dict(path=str(root/'module.zig'), escape=str(outside/'secret')))
        assert race['isError'] and 'OUTSIDE-SECRET' not in json.dumps(race); checks += 1
        (root/'module.zig').unlink(); (root/'module.zig-old').rename(root/'module.zig')
        # A grant's root identity cannot be replaced, even by a contained path.
        root.rename(base/'retired'); root.mkdir(); (root/'main.zig').write_text('replacement')
        assert pi.tool('rev_read', path='main.zig')['isError']; checks += 1
        pi.close()
    finally: pi.abort()
    root.rmdir() if not list(root.iterdir()) else None
    (root/'main.zig').unlink(); root.rmdir(); (base/'retired').rename(root)
    pi = Pi(root)
    try:
        for _ in range(32): assert not pi.tool('rev_read', path='module.zig').get('isError')
        assert pi.tool('rev_read', path='module.zig')['isError']; checks += 1
        pi.close()
    finally: pi.abort()
    pi = Pi(root)
    try:
        assert pi.probe(turns=9)['isError']
        assert pi.tool('rev_read', path='module.zig')['isError']; checks += 1
        pi.close()
    finally: pi.abort()
    (root/'wide').write_text('😀'*262144)
    pi = Pi(root)
    try:
        results = [pi.tool('rev_read', path='wide') for _ in range(32)]
        assert any(r.get('isError') for r in results)
        assert sum(len(json.dumps(r, ensure_ascii=False).encode()) for r in results if not r.get('isError')) <= 4194304
        checks += 1
        pi.close()
    finally: pi.abort()
    pi = Pi(root)
    try:
        result = pi.probe(tool='rev_read', args=dict(path='module.zig'), race=dict(path=str(root), escape=str(outside)))
        assert result['isError'] and 'OUTSIDE-SECRET' not in json.dumps(result)
        assert result['details']['fatal']; checks += 1
        pi.close()
    finally:
        pi.abort()
        if root.is_symlink(): root.unlink(); root.with_name(root.name+'-old').rename(root)
    # The actual Pi agent/tool runner consumes controlled model events. Unlike
    # the command probe, this proves tool messages and settled RPC ordering.
    pi = Pi(root, provider=True)
    try:
        records = pi.command(json.dumps(dict(instruction='Fixture only', dirty_sources=snapshots)), None)
        ends = [r for r in records if r.get('type') == 'tool_execution_end']
        assert len(ends) == 3 and all(not r['isError'] for r in ends), [(r.get('type'),r.get('isError')) for r in ends]
        read = next(r for r in ends if r['toolName'] == 'rev_read')
        assert json.loads(read['result']['content'][0]['text'])['text'] == 'fresh unsaved needle\n'
        assert (root/'fixture-turns').read_text().splitlines() == ['1','2']
        final = [r for r in records if r.get('type') == 'message_end' and r.get('message',{}).get('role') == 'assistant'][-1]
        assert final['message']['stopReason'] == 'stop'
        assert records[-1]['type'] == 'agent_settled' and not records[-1]['aborted']
        checks += 1
        pi.close()
    finally: pi.abort()
    (root/'fixture-turns').unlink(); (root/'fixture-infinite').touch()
    pi = Pi(root, provider=True)
    try:
        records = pi.command(json.dumps(dict(instruction='Fixture only', dirty_sources=snapshots)), None)
        assert len((root/'fixture-turns').read_text().splitlines()) <= 8, 'model turn budget failed'
        checks += 1
        pi.close()
    finally: pi.abort()
print(json.dumps(dict(version='1.3.0', checks=checks, external_provider_requests=0, controlled_model_loop=True, profile='only rev_read,rev_list,rev_search')))
