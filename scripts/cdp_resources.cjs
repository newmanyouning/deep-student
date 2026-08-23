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

            // Get the FULL page text to see the resource listing
            const fullPage = await send('Runtime.evaluate', {
                expression: `(function() {
                    // Get text from the main content area, excluding sidebar
                    const mainArea = document.querySelector('[class*="content"], [class*="main"], main, [class*="panel"], [class*="Panel"], [class*="workspace"], [class*="Workspace"]');
                    const area = mainArea || document.body;
                    const text = area.innerText;
                    const lines = text.split('\\n').filter(l => l.trim().length > 0);
                    return JSON.stringify(lines.slice(0, 100));
                })()`
            });
            console.log('=== Full page text (first 100 lines) ===');
            const lines = JSON.parse(fullPage?.result?.value || '[]');
            lines.forEach((l, i) => console.log((i+1) + ': ' + l));

            // Also dump the HTML structure of the resource grid
            const grid = await send('Runtime.evaluate', {
                expression: `(function() {
                    // Find the main content panel
                    const panels = document.querySelectorAll('[class*="panel"], [class*="Panel"], [class*="content"], [class*="Content"], [class*="grid"], [class*="Grid"]');
                    const htmls = [];
                    panels.forEach(p => {
                        const cls = p.className?.substring(0, 80);
                        const childCount = p.children.length;
                        const inner = p.innerHTML?.substring(0, 500);
                        if (childCount > 0) {
                            htmls.push({cls, childCount, innerPreview: inner});
                        }
                    });
                    return JSON.stringify(htmls.slice(0, 5));
                })()`
            });
            console.log('\n=== Panel HTML structure ===');
            const panels = JSON.parse(grid?.result?.value || '[]');
            panels.forEach(p => console.log(p.cls + ' | children:' + p.childCount + ' | inner:...' + (p.innerPreview || '').substring(0, 200)));

            ws.close();
            process.exit(0);
        });
    });
}).on('error', (e) => console.error(e));
