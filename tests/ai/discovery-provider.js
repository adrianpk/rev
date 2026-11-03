// Validation-only local provider. It yields controlled Pi events without HTTP.
import { createAssistantMessageEventStream, getCurrentTools, getCurrentSystemPrompt } from '@earendil-works/pi-ai';
import fs from 'node:fs';

export default function (pi) {
  let turns = 0, previousUsers = 0, invocationTurns = 0;
  fs.writeFileSync('fixture-pid', String(process.pid));
  fs.writeFileSync('fixture-limit', fs.readFileSync('/proc/self/limits','utf8').split('\n').find(line=>line.startsWith('Max file size')));
  pi.registerProvider('rev-fixture', {
    baseUrl: 'https://invalid.example', apiKey: 'local-fixture', api: 'rev-fixture-api',
    models: [{ id: 'local', name: 'Local fixture', reasoning: false, input: ['text'], cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 }, contextWindow: 32768, maxTokens: 1024 }],
    streamSimple(model, context) {
      const stream = createAssistantMessageEventStream();
      const tools = getCurrentTools(context.messages).map(tool => tool.name).sort();
      if (JSON.stringify(tools) !== JSON.stringify(['rev_list','rev_read','rev_search']) || getCurrentSystemPrompt(context.messages).includes('RESOURCE-INSTRUCTION-MUST-NOT-LOAD')) throw new Error('Unexpected effective resource/tool profile');
      turns++;
      const users=context.messages.filter(message=>message.role==='user');
      fs.writeFileSync('fixture-context',JSON.stringify({users:users.length,has_second:users.some(message=>{const text=typeof message.content==='string'?message.content:(message.content??[]).filter(block=>block.type==='text').map(block=>block.text).join('\n');try{return JSON.parse(text).instruction==='second';}catch{return false;}})}));
      fs.appendFileSync('fixture-turns', `${turns}\n`);
      if (users.length !== previousUsers) { previousUsers=users.length; invocationTurns=0; }
      invocationTurns++;
      const repeated=fs.existsSync('fixture-tools-per-invocation');
      const toolsNow=repeated?invocationTurns===1:turns===1;
      const infinite = fs.existsSync('fixture-infinite');
      const content = toolsNow || infinite ? [
        ...(repeated ? Array.from({length:16},(_,n)=>({ type:'toolCall',id:`extra-${turns}-${n}`,name:'rev_read',arguments:{path:'main.zig'} })) : []),
        { type: 'toolCall', id: `read-${turns}`, name: 'rev_read', arguments: { path: 'main.zig' } },
        { type: 'toolCall', id: `list-${turns}`, name: 'rev_list', arguments: {} },
        { type: 'toolCall', id: `search-${turns}`, name: 'rev_search', arguments: { query: 'needle' } },
      ] : [{ type: 'text', text: JSON.stringify({version:1,text:'Controlled offline result.'}) }];
      const message = { role: 'assistant', content, api: model.api, provider: model.provider, model: model.id, usage: { input:0,output:0,cacheRead:0,cacheWrite:0,totalTokens:0,cost:{input:0,output:0,cacheRead:0,cacheWrite:0,total:0} }, stopReason: toolsNow || infinite ? 'toolUse' : 'stop', timestamp: Date.now() };
      queueMicrotask(() => { stream.push({type:'done',reason:message.stopReason,message}); stream.end(); });
      return stream;
    },
  });
}
