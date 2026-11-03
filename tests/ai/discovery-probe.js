// Validation-only wrapper: actual Pi registration/runtime, controlled direct
// tool execution, no provider call. Never loaded by the production profile.
import fs from 'node:fs';
import discovery from '../../src/ai/pi/discovery.js';

export default function (pi) {
  const tools = new Map(), handlers = new Map();
  discovery({ ...pi,
    registerTool: tool => { tools.set(tool.name, tool); pi.registerTool(tool); },
    on: (name, handler) => { handlers.set(name, handler); pi.on(name, handler); },
  });
  pi.registerCommand('rev-probe', {
    handler: async (raw, ctx) => {
      const data = JSON.parse(raw);
      let result;
      const read = fs.readSync;
      try {
        if (data.envelope) {
          await handlers.get('before_agent_start')({ prompt: JSON.stringify(data.envelope) }, ctx);
          result = { installed: true };
        } else if (data.turns) {
          for (let i = 0; i < data.turns; i++) await handlers.get('turn_start')({}, ctx);
          result = { turns: data.turns };
        } else {
          if (data.race) {
            let fired = false;
            fs.readSync = (...args) => {
              if (!fired) {
                fired = true;
                fs.renameSync(data.race.path, data.race.path + '-old');
                fs.symlinkSync(data.race.escape, data.race.path);
              }
              return read(...args);
            };
          }
          result = await tools.get(data.tool).execute('probe', data.args, undefined);
        }
      } catch { result = { isError: true }; }
      finally { fs.readSync = read; }
      ctx.ui.notify(JSON.stringify({ type: 'rev-probe-result', result }), 'info');
    },
  });
}
