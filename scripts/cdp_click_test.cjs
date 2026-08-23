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

            // Click "学习资源" button (index 1 from previous scan)
            const clickResult = await send('Runtime.evaluate', {
                expression: `(function() {
                    const sidebar = document.querySelector('[class*="sidebar"]');
                    const buttons = sidebar.querySelectorAll('button');
                    // Find button with text "学习资源"
                    for (const btn of buttons) {
                        if (btn.textContent.trim() === '学习资源') {
                            btn.click();
                            return 'Clicked "' + btn.textContent.trim() + '"';
                        }
                    }
                    return 'Button not found';
                })()`
            });
            console.log('Click:', clickResult?.result?.value);

            // Wait 3 seconds for the view to update
            await new Promise(r => setTimeout(r, 3000));

            // Check what's now visible
            const view = await send('Runtime.evaluate', {
                expression: `(function() {
                    // Check for finder/resource grid
                    const finder = document.querySelector('[class*="finder"], [class*="Finder"], [class*="resourceGrid"], [class*="ResourceGrid"]');
                    const allText = document.body.innerText;
                    const lines = allText.split('\\n').filter(l => l.trim().length > 0).slice(0, 30);

                    // Count visible resource items
                    const items = document.querySelectorAll('[class*="resourceItem"], [class*="ResourceItem"], [class*="fileItem"], [class*="FileItem"], [class*="listItem"]');
                    const filenames = [];
                    items.forEach(i => {
                        const t = i.textContent?.trim()?.substring(0, 60);
                        if (t) filenames.push(t);
                    });

                    return JSON.stringify({
                        finderExists: !!finder,
                        finderClass: finder?.className?.substring(0, 100),
                        resourceItemsCount: items.length,
                        resourceItemSamples: filenames.slice(0, 10),
                        visibleLines: lines.slice(0, 15)
                    });
                })()`
            });
            console.log('\n=== After clicking 学习资源 ===');
            const viewInfo = JSON.parse(view?.result?.value || '{}');
            console.log('Finder exists:', viewInfo.finderExists);
            console.log('Resource items:', viewInfo.resourceItemsCount);
            console.log('Samples:', viewInfo.resourceItemSamples?.slice(0, 10));

            // If no resources visible, try checking the entire page for any content
            if (viewInfo.resourceItemsCount === 0) {
                console.log('\nPage content preview:');
                viewInfo.visibleLines?.forEach(l => console.log('  ' + l));
            }

            ws.close();
            process.exit(0);
        });
    });
}).on('error', (e) => console.error(e));
