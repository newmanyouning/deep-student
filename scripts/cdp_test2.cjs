#!/usr/bin/env node
const http = require('http');
const { WebSocket } = require('ws');

http.get('http://localhost:9222/json', (res) => {
    let data = '';
    res.on('data', chunk => data += chunk);
    res.on('end', async () => {
        const pages = JSON.parse(data);
        const page = pages.find(p => p.type === 'page');
        const ws = new WebSocket(page.webSocketDebuggerUrl);

        const pending = {};
        let id = 1;
        function send(method, params) {
            return new Promise(resolve => {
                const i = id++;
                pending[i] = resolve;
                ws.send(JSON.stringify({ id: i, method, params }));
            });
        }
        ws.on('message', (data) => {
            const msg = JSON.parse(data.toString());
            if (msg.id && pending[msg.id]) pending[msg.id](msg.result);
        });

        ws.on('open', async () => {
            await send('Runtime.enable');

            // Test 1: Find the Tauri IPC invoke function in the bundled code
            const tauriTest = await send('Runtime.evaluate', {
                expression: `(function() {
                    try {
                        // Try accessing through the module system
                        const mods = Object.keys(window).filter(k => k.includes('tauri') || k.includes('TAURI') || k.includes('__'));
                        return 'Window keys with tauri: ' + mods.join(', ');
                    } catch(e) { return 'Error: ' + e; }
                })()`
            });
            console.log('=== Tauri API Access ===');
            console.log(tauriTest?.result?.value);

            // Test 2: Access React fiber to find the finderStore
            const reactTest = await send('Runtime.evaluate', {
                expression: `(function() {
                    try {
                        const root = document.getElementById('root');
                        const fiberKey = Object.keys(root || {}).find(k => k.startsWith('__reactFiber'));
                        if (!fiberKey) return 'No React fiber found';
                        return 'React fiber found: ' + fiberKey;
                    } catch(e) { return 'Error: ' + e; }
                })()`
            });
            console.log('\n=== React Fiber ===');
            console.log(reactTest?.result?.value);

            // Test 3: Check the DOM for resource items with broader selectors
            const domTest = await send('Runtime.evaluate', {
                expression: `(function() {
                    try {
                        // Get all text content from major UI regions
                        const sidebar = document.querySelector('[class*="sidebar"]');
                        const mainContent = document.querySelector('[class*="content"], [class*="main"], main');

                        // Look for any elements containing 'PDF' or 'pdf' or document-like names
                        const allText = document.body.innerText;
                        const pdfLines = allText.split('\\n').filter(l => l.match(/pdf|PDF|教材|textbook|扫描/i)).slice(0, 5);

                        return JSON.stringify({
                            sidebarClasses: sidebar?.className?.substring(0, 200),
                            hasMainContent: !!mainContent,
                            pdfRelatedContent: pdfLines
                        });
                    } catch(e) { return 'Error: ' + e; }
                })()`
            });
            console.log('\n=== DOM Content ===');
            const domInfo = JSON.parse(domTest?.result?.value || '{}');
            console.log(JSON.stringify(domInfo, null, 2));

            // Test 4: Try to invoke a Tauri command through the IPC bridge
            const ipcTest = await send('Runtime.evaluate', {
                expression: `(async function() {
                    try {
                        // Tauri 2 uses __TAURI_INTERNALS__ for IPC
                        const internals = window.__TAURI_INTERNALS__;
                        if (!internals) return 'No __TAURI_INTERNALS__ found';

                        // Try to call dstu_list through IPC
                        const { invoke } = internals;
                        if (typeof invoke === 'function') {
                            const result = await invoke('dstu_list', {
                                path: '/',
                                options: { limit: 10, offset: 0 }
                            });
                            const textbooks = (result || []).filter(n => n.node_type === 'textbook' || n.node_type === 'Textbook');
                            const files = (result || []).filter(n => n.node_type === 'file' || n.node_type === 'File');
                            return 'Total: ' + result.length + ', Textbooks: ' + textbooks.length + ', Files: ' + files.length;
                        } else {
                            return '__TAURI_INTERNALS__.invoke not a function, keys: ' + Object.keys(internals).join(', ');
                        }
                    } catch(e) { return 'Error: ' + e.message || String(e); }
                })()`
            });
            console.log('\n=== IPC Test ===');
            console.log(ipcTest?.result?.value);

            ws.close();
            process.exit(0);
        });
    });
}).on('error', (e) => console.error(e));
