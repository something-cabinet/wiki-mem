import { spawn, type ChildProcess } from 'node:child_process';
import { createInterface } from 'node:readline';

let wmProcess: ChildProcess | null = null;
let requestId = 1;
let pending = new Map<number, { resolve: (v: any) => void; reject: (e: Error) => void }>();
let buffer = '';

export async function startWm(projectRoot?: string) {
  if (wmProcess) return;

  const args = ['run', '--', 'serve'];
  if (projectRoot) args.push('--project', projectRoot);

  // Find the wm binary — try target/debug first, then PATH
  const binPath = process.env.WM_BIN || '../target/debug/wm-cli.exe';

  wmProcess = spawn(binPath, ['serve', ...(projectRoot ? ['--project', projectRoot] : [])], {
    cwd: projectRoot || process.cwd(),
    stdio: ['pipe', 'pipe', 'pipe'],
    windowsHide: true,
  });

  const rl = createInterface({ input: wmProcess.stdout! });
  rl.on('line', (line: string) => {
    try {
      const msg = JSON.parse(line);
      if (msg.id && pending.has(msg.id)) {
        const p = pending.get(msg.id)!;
        pending.delete(msg.id);
        if (msg.error) {
          p.reject(new Error(msg.error.message || JSON.stringify(msg.error)));
        } else {
          p.resolve(msg.result);
        }
      }
    } catch {
      // ignore non-JSON output (stray logs)
    }
  });

  wmProcess.stderr?.on('data', (data: Buffer) => {
    // wm logs go to stderr — we can capture them for debugging
    if (process.env.DEBUG_WM) {
      console.error(`[wm] ${data.toString().trim()}`);
    }
  });

  wmProcess.on('exit', (code) => {
    console.error(`wm serve exited with code ${code}`);
    wmProcess = null;
  });

  // Send initialize request and wait for response
  await sendRequest('initialize', {});
}

export async function stopWm() {
  if (wmProcess) {
    wmProcess.kill();
    wmProcess = null;
  }
}

export async function callTool(name: string, args: Record<string, any> = {}): Promise<any> {
  // Ensure we're initialized
  if (!wmProcess) {
    await startWm();
  }

  return sendRequest('tools/call', { name, arguments: args });
}

async function sendRequest(method: string, params: any): Promise<any> {
  return new Promise((resolve, reject) => {
    const id = requestId++;

    const request = JSON.stringify({
      jsonrpc: '2.0',
      method,
      params,
      id,
    });

    pending.set(id, { resolve, reject });

    if (!wmProcess || !wmProcess.stdin) {
      reject(new Error('wm process not running'));
      return;
    }

    wmProcess.stdin.write(request + '\n');

    // Timeout after 30s
    setTimeout(() => {
      if (pending.has(id)) {
        pending.delete(id);
        reject(new Error('Request timed out'));
      }
    }, 30000);
  });
}
