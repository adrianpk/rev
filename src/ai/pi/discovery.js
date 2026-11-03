// Trusted internal Pi resource. Filesystem access stays here; Pi owns relevance
// and model iteration. No shell, subprocess, package loader or source writes.
import fs from 'node:fs';
import path from 'node:path';

const TOOLS = ['rev_read', 'rev_list', 'rev_search'];
const FILE = 1048576, RETURNED = 4194304;
const FLAGS = fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW | fs.constants.O_NONBLOCK;
const DIRECTORY = FLAGS | fs.constants.O_DIRECTORY;
const decoder = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
const denied = /^(?:\.git|\.pi|\.ssh|\.aws|\.gnupg|\.config|\.local|\.env(?:\..*)?|\.envrc|\.npmrc|\.pypirc|\.netrc|auth\.json|credentials(?:\..*)?|secrets(?:\..*)?|id_rsa.*|id_ed25519.*|.*\.(?:pem|key|p12|pfx))$/i;
const fdPath = fd => `/proc/self/fd/${fd}`;
const identity = st => `${st.dev}:${st.ino}`;
const observed = st => `${identity(st)}:${st.size}:${st.mtimeNs}:${st.ctimeNs}`;
const stat = fd => fs.fstatSync(fd, { bigint: true });
const error = message => { throw new Error(`rev discovery: ${message}`); };

function parts(value, root = false) {
  if (typeof value !== 'string' || value.length > 4096 || value.includes('\0') || (!root && path.isAbsolute(value))) error('invalid relative path');
  const result = value === '.' ? [] : value.split('/');
  if (result.some(p => !p || p === '.' || p === '..' || denied.test(p))) error('path is outside source policy');
  return result;
}
function integer(value, fallback, min, max) {
  value ??= fallback;
  if (!Number.isSafeInteger(value) || value < min || value > max) error('invalid range');
  return value;
}

export default function (pi) {
  let grant, rootFd, rootId, calls = 0, returned = 0, turns = 0, poisoned = false;
  let snapshots = new Map();
  function check(signal) {
    if (!grant || poisoned) error('grant unavailable');
    if (signal?.aborted || Date.now() >= grant.deadline) error('operation canceled or timed out');
    let current;
    try { current = fs.lstatSync(grant.root, { bigint: true }); }
    catch { poisoned = true; error('root observation changed'); }
    if (!current.isDirectory() || current.isSymbolicLink() || identity(current) !== rootId || identity(stat(rootFd)) !== rootId || fs.realpathSync(fdPath(rootFd)) !== grant.root) { poisoned = true; error('root observation changed'); }
  }
  function admitted(fd) {
    const actual = fs.realpathSync(fdPath(fd));
    if ((actual !== grant.root && !actual.startsWith(grant.root + '/')) || grant.private_roots.some(p => actual === p || actual.startsWith(p + '/'))) error('opened handle escaped source grant');
  }
  // Each component is opened relative to the retained directory descriptor.
  // NOFOLLOW and post-IO descriptor/path checks cover swaps after resolution.
  function open(relative, directory, missing = false) {
    const names = parts(relative);
    let fd = fs.openSync(`${fdPath(rootFd)}/.`, DIRECTORY);
    try {
      for (let i = 0; i < names.length; i++) {
        const last = i === names.length - 1;
        let next;
        if (last && !directory) {
          let meta;
          try { meta = fs.lstatSync(`${fdPath(fd)}/${names[i]}`, { bigint: true }); }
          catch (e) { if (missing && e.code === 'ENOENT') { admitted(fd); return undefined; } throw e; }
          if (!meta.isFile() || meta.nlink > 1n) error('unsupported regular file or link');
        }
        try { next = fs.openSync(`${fdPath(fd)}/${names[i]}`, last && !directory ? FLAGS : DIRECTORY); }
        catch (e) { if (missing && last && e.code === 'ENOENT') return undefined; throw e; }
        fs.closeSync(fd); fd = next;
        admitted(fd);
      }
      if (directory ? !stat(fd).isDirectory() : !stat(fd).isFile()) error('unsupported source kind');
      const result = fd; fd = undefined; return result;
    } finally { if (fd !== undefined) fs.closeSync(fd); }
  }
  function source(relative, signal) {
    check(signal);
    const snapshot = snapshots.get(relative);
    const fd = open(relative, false, !!snapshot);
    try {
      if (fd !== undefined && (!stat(fd).isFile() || stat(fd).nlink > 1n)) error('unsupported regular file or hard link');
      if (snapshot) {
        if (fd !== undefined) admitted(fd);
        check(signal);
        return { text: snapshot.text, provenance: 'native editor snapshot', revision: snapshot.revision, buffer_id: snapshot.buffer_id, bom: snapshot.bom };
      }
      const before = stat(fd);
      if (before.size > BigInt(FILE)) error('file exceeds 1 MiB');
      const data = Buffer.alloc(Number(before.size) + 1);
      let count = 0;
      while (count < data.length) {
        check(signal);
        const n = fs.readSync(fd, data, count, data.length - count, count);
        if (!n) break;
        count += n;
      }
      admitted(fd); check(signal);
      const again = open(relative, false);
      try { if (observed(before) !== observed(stat(fd)) || observed(before) !== observed(stat(again)) || count !== Number(before.size)) error('source observation changed'); }
      finally { fs.closeSync(again); }
      if (data.subarray(0, count).includes(0)) error('binary file');
      let text;
      try { text = decoder.decode(data.subarray(0, count)); } catch { error('source is not UTF-8'); }
      return { text, provenance: 'disk', observation: observed(before), bom: text.startsWith('\uFEFF') };
    } finally { if (fd !== undefined) fs.closeSync(fd); }
  }
  function walk(relative, recursive, signal, visit) {
    const seen = new Set(); let entries = 0, complete = true, skipped = 0;
    function descend(dir, depth) {
      check(signal);
      const fd = open(dir, true);
      let iterator;
      try {
        const id = identity(stat(fd));
        if (seen.has(id)) { skipped++; return; } seen.add(id);
        iterator = fs.opendirSync(fdPath(fd));
        for (let entry; (entry = iterator.readSync()) !== null;) {
          check(signal); admitted(fd);
          if (++entries > 4096) { complete = false; return false; }
          if (denied.test(entry.name)) { skipped++; continue; }
          const name = dir === '.' ? entry.name : `${dir}/${entry.name}`;
          const absolute = `${grant.root}/${name}`;
          if (grant.private_roots.some(p => absolute === p || absolute.startsWith(p + '/'))) { skipped++; continue; }
          if (entry.isSymbolicLink()) { skipped++; continue; }
          if (visit(name, entry) === false) { complete = false; return false; }
          if (recursive && entry.isDirectory()) {
            if (depth === 16) { complete = false; continue; }
            if (descend(name, depth + 1) === false) return false;
          }
        }
        admitted(fd); check(signal);
      } finally { iterator?.closeSync(); fs.closeSync(fd); }
    }
    descend(relative, 0);
    return { complete, entries: Math.min(entries, 4096), skipped };
  }
  function read(args, signal) {
    const relative = args.path; parts(relative);
    const start = integer(args.start, 0, 0, FILE), limit = integer(args.limit, 32768, 1, 32768);
    const value = source(relative, signal), chars = Array.from(value.text);
    if (start > chars.length) error('range exceeds source');
    const end = Math.min(chars.length, start + limit);
    return { ...value, text: chars.slice(start, end).join(''), path: relative, range_scalars: [start, end], source_scalars: chars.length, complete: start === 0 && end === chars.length };
  }
  function list(args, signal) {
    const relative = args.path ?? '.', match = args.match ?? '';
    parts(relative);
    if (typeof match !== 'string' || match.length > 256 || match.includes('\0')) error('invalid literal path match');
    if (args.recursive !== undefined && typeof args.recursive !== 'boolean') error('invalid recursion');
    const limit = integer(args.limit, 128, 1, 256), items = [];
    let size = 0;
    const scan = walk(relative, args.recursive ?? false, signal, (name, entry) => {
      if (name.includes(match)) {
        size += Buffer.byteLength(name) + 128;
        if (size > 196608) return false;
        items.push({ path: name, kind: entry.isDirectory() ? 'directory' : entry.isFile() ? 'file' : 'other', provenance: 'disk directory' });
      }
      return items.length < limit;
    });
    return { items, ...scan, path: relative, match };
  }
  function search(args, signal) {
    const relative = args.path ?? '.', query = args.query;
    parts(relative);
    if (typeof query !== 'string' || !query || Buffer.byteLength(query) > 256 || query.includes('\0')) error('invalid literal query');
    const limit = integer(args.limit, 64, 1, 128), matches = [], visited = new Set();
    let bytes = 0, files = 0, skipped = 0, exhausted = false, size = 0;
    function inspect(name) {
      visited.add(name);
      if (++files > 128 || bytes >= RETURNED) { exhausted = true; return false; }
      let value;
      try { value = source(name, signal); } catch { check(signal); skipped++; return; }
      bytes += Buffer.byteLength(value.text);
      if (bytes > RETURNED) { exhausted = true; return false; }
      let line = 0;
      for (const text of value.text.split('\n')) {
        line++;
        const at = text.indexOf(query);
        if (at < 0) continue;
        size += Buffer.byteLength(name) + 4096;
        if (size > 196608) return false;
        matches.push({ path: name, line, text: Array.from(text.slice(Math.max(0, at - 128))).slice(0, 512).join(''), provenance: value.provenance, revision: value.revision, observation: value.observation });
        if (matches.length >= limit) return false;
      }
    }
    const scan = walk(relative, true, signal, (name, entry) => entry.isFile() ? inspect(name) : undefined);
    if (scan.complete) {
      for (const name of snapshots.keys()) {
        if (!visited.has(name) && (relative === '.' || name.startsWith(relative + '/')) && inspect(name) === false) { scan.complete = false; break; }
      }
    }
    return { matches, ...scan, complete: scan.complete && !exhausted && skipped === 0, skipped: scan.skipped + skipped, files: Math.min(files, 128), scanned_bytes: Math.min(bytes, RETURNED), query };
  }
  const operations = { rev_read: read, rev_list: list, rev_search: search };
  for (const name of TOOLS) {
    const properties = name === 'rev_read'
      ? { path: { type: 'string' }, start: { type: 'integer' }, limit: { type: 'integer' } }
      : name === 'rev_list'
        ? { path: { type: 'string' }, match: { type: 'string' }, recursive: { type: 'boolean' }, limit: { type: 'integer' } }
        : { path: { type: 'string' }, query: { type: 'string' }, limit: { type: 'integer' } };
    pi.registerTool({ name, label: name, description: name === 'rev_read' ? 'Read checked UTF-8 source (scalar start/limit, max 32768). Native snapshots override disk.' : name === 'rev_list' ? 'List checked source paths; optional literal substring match and recursion. Limits and completeness are explicit.' : 'Search UTF-8 sources for a literal case-sensitive query; native snapshots override disk. No regex or shell.',
      parameters: { type: 'object', properties, required: name === 'rev_read' ? ['path'] : name === 'rev_search' ? ['query'] : [], additionalProperties: false },
      async execute(_id, args, signal, _update, ctx) {
        try {
          check(signal);
          if (++calls > 32) { poisoned = true; error('32 tool calls exceeded'); }
          // Reserve the maximum per-call returned record before source IO.
          if (RETURNED - returned < 262144) { poisoned = true; error('returned discovery budget exhausted'); }
          const result = operations[name](args, signal);
          check(signal);
          const text = JSON.stringify({ version: 1, root: grant.root, ...result });
          const size = Buffer.byteLength(text);
          if (size > 262144 || returned + size > RETURNED) { poisoned = true; error('returned discovery budget exceeded'); }
          returned += size;
          return { content: [{ type: 'text', text }], details: { version: 1, bytes: size } };
        } catch (e) {
          if (poisoned) void ctx?.abort();
          // Filesystem errors can contain incidental paths. Keep them out of the
          // model transcript; explicit boundary failures are already sanitized.
          const text = e.message?.startsWith('rev discovery:') ? e.message : 'rev discovery: source unavailable or path changed';
          return { content: [{ type: 'text', text }], details: { version: 1, fatal: poisoned }, isError: true };
        }
      },
    });
  }
  pi.registerCommand('rev-discovery', {
    description: 'Admit the editor-owned read-only discovery grant',
    handler: async (raw, ctx) => {
      let nonce;
      try {
        if (grant) error('grant already admitted');
        const data = JSON.parse(raw); nonce = data.nonce;
        if (data.version !== 1 || !path.isAbsolute(data.root) || fs.realpathSync(data.root) !== data.root) error('invalid canonical root');
        parts(data.root.slice(1), true);
        const fd = fs.openSync(data.root, DIRECTORY);
        rootFd = fd; rootId = identity(stat(fd));
        if (rootId !== `${data.dev}:${data.ino}`) error('root observation changed');
        grant = { root: data.root, deadline: Date.now() + integer(data.timeout_ms, undefined, 1, 1800000), max_input_bytes: integer(data.max_input_bytes, undefined, 1, 4194304), private_roots: data.private_roots };
        if (!Array.isArray(grant.private_roots) || grant.private_roots.length > 16 || grant.private_roots.some(p => typeof p !== 'string' || !path.isAbsolute(p))) error('invalid private paths');
        admitted(rootFd);
        check();
        const tools = pi.getActiveTools().slice().sort();
        if (JSON.stringify(tools) !== JSON.stringify(TOOLS.slice().sort())) error('effective tool profile differs');
        ctx.ui.notify(JSON.stringify({ type: 'rev-discovery-v1', nonce, tools, admitted: true }), 'info');
      } catch {
        poisoned = true;
        ctx.ui.notify(JSON.stringify({ type: 'rev-discovery-v1', nonce, admitted: false }), 'error');
      }
    },
  });
  pi.on('before_agent_start', async (event, ctx) => {
    try {
    check();
    if (++turns > 1 || Buffer.byteLength(event.prompt) > grant.max_input_bytes) { poisoned = true; error('invalid invocation envelope'); }
    poisoned = true;
    const envelope = JSON.parse(event.prompt);
    const sources = envelope.dirty_sources;
    if (!Array.isArray(sources) || sources.length > 16) error('invalid native source map');
    for (const source of sources) {
      parts(source.path);
      if (snapshots.has(source.path) || source.complete !== true || typeof source.text !== 'string' || Buffer.byteLength(source.text) > FILE || source.text.includes('\0')) error('invalid native source snapshot');
      snapshots.set(source.path, source);
    }
    poisoned = false;
    } catch (e) { poisoned = true; void ctx.abort(); throw e; }
  });
  // Model turns include internal continuations; compaction never resets limits.
  let modelTurns = 0;
  pi.on('turn_start', async (_event, ctx) => {
    try { check(); if (++modelTurns > 8) { poisoned = true; error('eight model turns exceeded'); } }
    catch (e) { void ctx.abort(); throw e; }
  });
  pi.on('before_provider_request', async (_event, ctx) => {
    try { check(); } catch (e) { void ctx.abort(); throw e; }
  });
  pi.on('tool_call', async (event, ctx) => {
    try { check(); } catch (e) { void ctx.abort(); throw e; }
    if (!TOOLS.includes(event.toolName)) return { block: true, reason: 'Tool is outside the editor grant' };
  });
  pi.on('session_shutdown', async () => { if (rootFd !== undefined) { fs.closeSync(rootFd); rootFd = undefined; } });
}
