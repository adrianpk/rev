// Validation-only local provider. It yields controlled Pi events without HTTP.
import { createAssistantMessageEventStream, getCurrentTools, getCurrentSystemPrompt } from '@earendil-works/pi-ai';
import fs from 'node:fs';

export default function (pi) {
  let turns = 0;
  pi.registerProvider('rev-fixture', {
    baseUrl: 'https://invalid.example', apiKey: 'local-fixture', api: 'rev-fixture-api',
    models: [{ id: 'local', name: 'Local fixture', reasoning: false, input: ['text'], cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 }, contextWindow: 32768, maxTokens: 1024 }],
    streamSimple(model, context) {
      const stream = createAssistantMessageEventStream();
      const tools = getCurrentTools(context.messages).map(tool => tool.name).sort();
      if (JSON.stringify(tools) !== JSON.stringify(['rev_list','rev_read','rev_search']) || getCurrentSystemPrompt(context.messages).includes('RESOURCE-INSTRUCTION-MUST-NOT-LOAD')) throw new Error('Unexpected effective resource/tool profile');
      turns++;
      fs.appendFileSync('fixture-turns', `${turns}\n`);
      const infinite = fs.existsSync('fixture-infinite');
      const content = turns === 1 || infinite ? [
        { type: 'toolCall', id: `read-${turns}`, name: 'rev_read', arguments: { path: 'main.zig' } },
        { type: 'toolCall', id: `list-${turns}`, name: 'rev_list', arguments: {} },
        { type: 'toolCall', id: `search-${turns}`, name: 'rev_search', arguments: { query: 'needle' } },
      ] : [{ type: 'text', text: JSON.stringify({version:1,text:'Controlled offline result.'}) }];
      const message = { role: 'assistant', content, api: model.api, provider: model.provider, model: model.id, usage: { input:0,output:0,cacheRead:0,cacheWrite:0,totalTokens:0,cost:{input:0,output:0,cacheRead:0,cacheWrite:0,total:0} }, stopReason: turns === 1 || infinite ? 'toolUse' : 'stop', timestamp: Date.now() };
      queueMicrotask(() => { stream.push({type:'done',reason:message.stopReason,message}); stream.end(); });
      return stream;
    },
  });
}
