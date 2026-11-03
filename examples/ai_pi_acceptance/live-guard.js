// Validation-only cap for the single authorized live input-only operation.
import fs from 'node:fs';
export default function (pi) {
  let turns = 0, calls = 0;
  const record = () => fs.writeFileSync('live-counts', JSON.stringify({ turns, tool_calls: calls }));
  record();
  pi.on('turn_start', (_event, ctx) => {
    turns++;
    record();
    if (turns > 4) { void ctx.abort(); throw new Error('live acceptance: four model turns exceeded'); }
  });
  pi.on('tool_call', (_event, ctx) => {
    calls++;
    record();
    if (calls > 8) { void ctx.abort(); throw new Error('live acceptance: eight context calls exceeded'); }
  });
}
