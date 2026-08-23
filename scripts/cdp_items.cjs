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

            // Get ALL divs inside the learning hub view layer
            const allItems = await send('Runtime.evaluate', {
                expression: `(function() {
                    const hub = document.querySelector('[data-view-layer-shell="learning-hub"]');
                    if (!hub) return 'No learning hub layer found';

                    // Get all text from within the hub
                    const text = hub.innerText;
                    const lines = text.split('\\n').filter(l => l.trim().length > 0);

                    // Also check for specific file item components
                    const fileItems = hub.querySelectorAll('[class*="file"], [class*="File"], [class*="item"], [class*="Item"], [class*="row"], [class*="Row"]');
                    const itemTexts = [];
                    fileItems.forEach(el => {
                        const t = el.textContent?.trim()?.substring(0, 60);
                        const cls = el.className?.substring(0, 40);
                        if (t && t.length > 2) itemTexts.push(t + ' [' + cls + ']');
                    });

                    return JSON.stringify({
                        hubText: lines.slice(0, 40),
                        itemCount: fileItems.length,
                        itemSamples: itemTexts.slice(0, 15)
                    });
                })()`
            });
            console.log('=== Learning Hub Items ===');
            const info = JSON.parse(allItems?.result?.value || '{}');
            console.log('Found items in hub:', info.itemCount);
            info.hubText?.forEach(l => console.log('  ' + l));
            if (info.itemSamples?.length > 0) {
                console.log('\n=== File Items ===');
                info.itemSamples.forEach(s => console.log('  ' + s));
            }

            // Also try to find the virtual list
            const virtList = await send('Runtime.evaluate', {
                expression: `(function() {
                    // Look for virtual list container
                    const containers = document.querySelectorAll('[class*="virtual"], [class*="Virtual"], [class*="vlist"], [class*="VList"], [role="list"], [role="grid"], [class*="listContainer"], [class*="ListContainer"]');
                    const info = [];
                    containers.forEach(el => {
                        const children = el.children.length;
                        const text = el.textContent?.substring(0, 150);
                        const dataItems = el.querySelectorAll('[data-index], [data-id], [data-resource-id]');
                        info.push({
                            cls: el.className?.substring(0, 60),
                            children,
                            textPreview: text,
                            dataItems: dataItems.length
                        });
                    });
                    return JSON.stringify(info.slice(0, 10));
                })()`
            });
            console.log('\n=== Virtual List Containers ===');
            const virtInfo = JSON.parse(virtList?.result?.value || '[]');
            virtInfo.forEach(v => console.log(v.cls + ' | children:' + v.children + ' | dataItems:' + v.dataItems + ' | text:' + (v.textPreview || '').substring(0, 80)));

            ws.close();
            process.exit(0);
        });
    });
}).on('error', (e) => console.error(e));
