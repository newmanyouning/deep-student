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

            // Navigate to Learning Hub
            await send('Runtime.evaluate', {
                expression: `(function() {
                    const sidebar = document.querySelector('[class*="sidebar"]');
                    const buttons = sidebar.querySelectorAll('button');
                    for (const btn of buttons) {
                        if (btn.textContent.trim() === '学习资源') { btn.click(); return 'navigated'; }
                    }
                    return 'not found';
                })()`
            });
            await new Promise(r => setTimeout(r, 3000));

            // Click All Textbooks and check items
            await send('Runtime.evaluate', {
                expression: `(function() {
                    const allBtns = document.querySelectorAll('button, a, [role="button"]');
                    for (const btn of allBtns) {
                        if (btn.textContent.trim() === '全部教材') { btn.click(); return 'clicked'; }
                    }
                    return 'not found';
                })()`
            });
            await new Promise(r => setTimeout(r, 3000));

            const check = await send('Runtime.evaluate', {
                expression: `(function() {
                    const hub = document.querySelector('[data-view-layer-shell="learning-hub"]');
                    if (!hub) return JSON.stringify({error: 'no hub'});
                    const text = hub.innerText;
                    const lines = text.split('\\n').filter(l => l.trim().length > 0);
                    const pdfCount = lines.filter(l => l.toLowerCase().endsWith('.pdf')).length;
                    const itemCount = (text.match(/\\d+\\s*个项目/g) || ['0'])[0];
                    // Check for OCR-related UI elements
                    const ocrButtons = hub.querySelectorAll('[class*="ocr"], [class*="OCR"], [class*="scan"]');
                    return JSON.stringify({
                        pdfItems: pdfCount,
                        itemCountText: itemCount,
                        totalLines: lines.length,
                        hasOcrElements: ocrButtons.length > 0,
                        sampleLines: lines.slice(0, 5)
                    });
                })()`
            });
            console.log('=== Final Verification ===');
            console.log(check?.result?.value);
            ws.close();
            process.exit(0);
        });
    });
}).on('error', (e) => console.error(e));
