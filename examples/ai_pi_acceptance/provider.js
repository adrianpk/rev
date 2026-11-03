// Validation only: a deterministic provider in the installed Pi runtime, no HTTP.
import { createAssistantMessageEventStream, getCurrentTools, getCurrentSystemPrompt } from '@earendil-works/pi-ai';
import fs from 'node:fs';
const plain = message => typeof message.content === 'string' ? message.content : (message.content ?? []).filter(block => block.type === 'text').map(block => block.text).join('\n');
const assert = (value, reason) => { if (!value) throw new Error(`acceptance: ${reason}`); };
export default function (pi) {
  let previousUsers = 0, turn = 0;
  fs.writeFileSync('fixture-pid', String(process.pid));
  fs.writeFileSync('fixture-limit', fs.readFileSync('/proc/self/limits', 'utf8').split('\n').find(line => line.startsWith('Max file size')));
  pi.registerProvider('rev-acceptance', {
    baseUrl: 'https://invalid.example', apiKey: 'owned-local-fixture', api: 'rev-acceptance-api',
    models: [{ id: 'local', name: 'Offline acceptance', reasoning: false, input: ['text'], cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 }, contextWindow: 32768, maxTokens: 1024 }],
    streamSimple(model, context) {
      const stream = createAssistantMessageEventStream();
      const tools = getCurrentTools(context.messages).map(tool => tool.name).sort();
      assert(JSON.stringify(tools) === JSON.stringify(['rev_list', 'rev_read', 'rev_search']), 'effective tools');
      assert(!getCurrentSystemPrompt(context.messages).includes('RESOURCE-INSTRUCTION-MUST-NOT-LOAD'), 'automatic context loaded');
      const users = context.messages.filter(message => message.role === 'user');
      if (users.length !== previousUsers) { previousUsers = users.length; turn = 0; }
      turn++;
      fs.writeFileSync('fixture-context', JSON.stringify({ users: users.length, turn, prior: users.slice(0, -1).some(message => plain(message).includes('ACCEPTANCE-FIRST')) }));
      let content;
      if (turn === 1) {
        content = [
          { type: 'toolCall', id: `read-${users.length}`, name: 'rev_read', arguments: { path: 'main.zig' } },
          { type: 'toolCall', id: `list-${users.length}`, name: 'rev_list', arguments: { match: '.zig' } },
          { type: 'toolCall', id: `search-${users.length}`, name: 'rev_search', arguments: { query: 'needle' } },
          ...['../outside', 'escape', '.env', 'missing', 'binary', 'large'].map((path, i) => ({ type: 'toolCall', id: `deny-${users.length}-${i}`, name: 'rev_read', arguments: { path } })),
        ];
      } else {
        assert(turn === 2, 'unexpected iteration');
        const result = id => context.messages.find(message => message.role === 'toolResult' && message.toolCallId === `${id}-${users.length}`);
        const read = JSON.parse(plain(result('read')));
        // Assert actual returned bytes and semantics; provenance alone is insufficient.
        assert(read.text === fs.readFileSync('expected-main', 'utf8') && read.complete === true, 'read snapshot bytes');
        const list = JSON.parse(plain(result('list')));
        assert(list.items.some(item => item.path === 'main.zig') && !list.items.some(item => item.path === '.env'), 'list results');
        const search = JSON.parse(plain(result('search')));
        assert(search.matches.some(item => item.path === 'main.zig' && item.text.includes('needle')), 'search results');
        for (let i = 0; i < 6; i++) {
          const denied = context.messages.find(message => message.role === 'toolResult' && message.toolCallId === `deny-${users.length}-${i}`);
          assert(denied && plain(denied).startsWith('rev discovery:') && !plain(denied).includes('MUST-NOT-LOAD'), 'read denial');
        }
        fs.writeFileSync('fixture-tools', JSON.stringify({ allowed: 3, rejected: 6, provenance: read.provenance, users: users.length }));
        content = [{ type: 'text', text: JSON.stringify({ version: 1, text: '// Controlled offline proposal.\n' }) }];
      }
      const message = { role: 'assistant', content, api: model.api, provider: model.provider, model: model.id, usage: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, totalTokens: 0, cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 } }, stopReason: turn === 1 ? 'toolUse' : 'stop', timestamp: Date.now() };
      queueMicrotask(() => { stream.push({ type: 'done', reason: message.stopReason, message }); stream.end(); });
      return stream;
    },
  });
}
