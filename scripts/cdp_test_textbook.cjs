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

            // First ensure we're on the learning hub
            const sidebar = await send('Runtime.evaluate', {
                expression: `(function() {
                    const sidebar = document.querySelector('[class*="sidebar"]');
                    const buttons = sidebar.querySelectorAll('button');
                    for (const btn of buttons) {
                        if (btn.textContent.trim() === '学习资源') {
                            btn.click();
                            return 'navigated to learning hub';
                        }
                    }
                    return 'learning hub button not found';
                })()`
            });
            console.log('Navigate:', sidebar?.result?.value);
            await new Promise(r => setTimeout(r, 3000));

            // Now click "全部教材"
            const textbookClick = await send('Runtime.evaluate', {
                expression: `(function() {
                    const allButtons = document.querySelectorAll('button, a, [role="button"], [class*="item"], [class*="Item"], [class*="entry"], [class*="Entry"], [class*="nav"]');
                    for (const btn of allButtons) {
                        const text = btn.textContent?.trim();
                        if (text === '全部教材') {
                            btn.click();
                            return 'Clicked ' + text;
                        }
                    }
                    // Try broader search
                    const body = document.body.innerText;
                    const hasTextbook = body.includes('全部教材');
                    return '全部教材 button ' + (hasTextbook ? 'exists in text but' : 'not ') + 'not clickable via button/a';
                })()`
            });
            console.log('Click textbook:', textbookClick?.result?.value);
            await new Promise(r => setTimeout(r, 3000));

            // Check resource listing
            const listing = await send('Runtime.evaluate', {
                expression: `(function() {
                    const body = document.body;
                    const allText = body.innerText;
                    const lines = allText.split('\\n').filter(l => l.trim().length > 0);

                    // Find resource items - look for clickable elements
                    const clickables = document.querySelectorAll('[class*="item"], [class*="Item"], [class*="row"], [class*="Row"], [class*="entry"], [class*="Entry"], [class*="card"], [class*="Card"]');
                    const resourceTexts = [];
                    clickables.forEach(el => {
                        const t = el.textContent?.trim()?.substring(0, 80);
                        if (t && (t.includes('PDF') || t.includes('pdf') || t.includes('教材') ||
                            t.includes('课本') || t.includes('参考资料') || t.includes('.pdf'))) {
                            resourceTexts.push(t);
                        }
                    });

                    // Also list all visible item names
                    const allItems = [];
                    clickables.forEach(el => {
                        const t = el.textContent?.trim()?.substring(0, 50);
                        if (t && t.length > 2 && !allItems.includes(t)) {
                            allItems.push(t);
                        }
                    });

                    return JSON.stringify({
                        visibleLinesAroundTextbookSection: lines.filter(l => l.match(/教材|textbook|pdf|PDF|课本/i)).slice(0, 10),
                        pdfResourceItems: resourceTexts.slice(0, 10),
                        allDistinctItems: allItems.slice(0, 30),
                        totalLines: lines.length
                    });
                })()`
            });
            console.log('\n=== Textbook Resource Listing ===');
            const listingInfo = JSON.parse(listing?.result?.value || '{}');
            console.log('Total visible lines:', listingInfo.totalLines);
            console.log('PDF-related items:', listingInfo.pdfResourceItems.length);
            listingInfo.pdfResourceItems?.forEach(t => console.log('  PDF: ' + t));
            console.log('\nAll items (first 30):');
            listingInfo.allDistinctItems?.forEach(t => console.log('  - ' + t));

            ws.close();
            process.exit(0);
        });
    });
}).on('error', (e) => console.error(e));
