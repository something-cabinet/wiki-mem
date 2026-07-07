import { startWm, stopWm } from '$lib/server/wm-bridge';

// Start wm engine on server startup
startWm().then(() => console.log('WM engine started')).catch(e => console.error('WM engine failed:', e));

// Stop on exit
process.on('exit', () => stopWm());
