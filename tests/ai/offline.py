#!/usr/bin/python3
"""Bounded installed-CLI compatibility probe; never submits a provider prompt."""
import json
import os
from pathlib import Path
import selectors
import subprocess
import sys
import tempfile
import time

executable = str(Path(sys.argv[1]).resolve(strict=True))
version = subprocess.run([executable, '--offline', '--version'], capture_output=True, timeout=5, check=True)
assert version.stdout.strip() == b'1.3.0'
flags = ['--mode', 'rpc', '--no-session', '--no-tools', '--no-extensions', '--no-mcp',
         '--no-skills', '--no-prompt-templates', '--no-context-files', '--no-themes',
         '--no-approve', '--offline']
for attempt in range(2):
    with tempfile.TemporaryDirectory(prefix='rev-pi-offline-') as cwd:
        started = time.monotonic()
        child = subprocess.Popen([executable, *flags], cwd=cwd, stdin=subprocess.PIPE,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            child.stdin.write(b'{"id":"offline","type":"get_state"}\n')
            child.stdin.flush()
            streams = selectors.DefaultSelector()
            streams.register(child.stdout, selectors.EVENT_READ, 'stdout')
            streams.register(child.stderr, selectors.EVENT_READ, 'stderr')
            stdout, stderr = bytearray(), bytearray()
            deadline = started + 5
            while b'\n' not in stdout:
                assert time.monotonic() < deadline, 'Pi startup deadline'
                for key, _ in streams.select(.05):
                    data = os.read(key.fd, 4096)
                    assert data, 'Pi closed a stream before get_state'
                    if key.data == 'stdout':
                        stdout.extend(data)
                        assert len(stdout) <= 2 * 1024 * 1024, 'Pi framing limit'
                    else:
                        stderr.extend(data)
                        del stderr[:-65536]
            record = json.loads(stdout.split(b'\n', 1)[0])
            assert record['id'] == 'offline' and record['success']
            state = record['data']
            assert not state['isStreaming'] and not state['isCompacting']
            assert state['messageCount'] == state['pendingMessageCount'] == 0
            assert not state.get('sessionFile')
            startup_ms = round((time.monotonic() - started) * 1000)
            def ticks():
                # Fields after the closing comm parenthesis start at process state.
                fields = Path(f'/proc/{child.pid}/stat').read_text().rsplit(')', 1)[1].split()
                return int(fields[11]) + int(fields[12])
            before = ticks()
            idle_output = bool(streams.select(.25))
            idle_ticks = ticks() - before
            rss = next(line.split()[1] for line in Path(f'/proc/{child.pid}/status').read_text().splitlines() if line.startswith('VmHWM:'))
            child.stdin.close()
            child.wait(timeout=2)
            assert child.returncode == 0
            assert not list(Path(cwd).iterdir()), 'input-only probe created history/resources'
            print(json.dumps(dict(attempt=attempt+1, version='1.3.0', fresh=True,
                                  startup_ms=startup_ms, idle_ms=250, idle_ticks=idle_ticks,
                                  idle_output=idle_output, peak_rss_kib=int(rss), owned_exit=0)))
        finally:
            if child.poll() is None:
                child.kill()
                child.wait(timeout=2)
