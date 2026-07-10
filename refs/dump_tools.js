const { spawn } = require('child_process');
const proc = spawn(
  'C:\\Users\\hk\\.kimaki\\projects\\vpp-rag\\target\\debug\\wm-cli.exe',
  ['mcp', '--project', 'C:\\Users\\hk\\.kimaki\\projects\\vpp-rag'],
  { stdio: ['pipe', 'pipe', 'pipe'] }
);
let buf = '';
proc.stdout.on('data', d => { buf += d.toString(); });
proc.stderr.on('data', d => {});
function send(m) { proc.stdin.write(JSON.stringify(m) + '\n'); }
function waitFor(id) {
  return new Promise((resolve, reject) => {
    const start = Date.now();
    const check = () => {
      const lines = buf.split('\n').filter(l => l.trim());
      for (const line of lines) {
        try { const r = JSON.parse(line); if (r.id === id) { resolve(r); return; } } catch {}
      }
      if (Date.now() - start > 15000) reject(new Error('timeout'));
      else setTimeout(check, 50);
    };
    check();
  });
}
(async () => {
  send({jsonrpc:'2.0', id:1, method:'initialize', params:{protocolVersion:'2024-11-05', capabilities:{}, clientInfo:{name:'t', version:'1'}}});
  await waitFor(1);
  send({jsonrpc:'2.0', method:'notifications/initialized'});
  await new Promise(r => setTimeout(r, 500));
  send({jsonrpc:'2.0', id:2, method:'tools/list', params:{}});
  const r = await waitFor(2);
  console.log(JSON.stringify(r.result.tools, null, 2));
  proc.kill();
})();
