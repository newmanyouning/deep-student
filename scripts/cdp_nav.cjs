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
            await send('DOM.enable');

            // Find and click on sidebar navigation items
            const nav = await send('Runtime.evaluate', {
                expression: `(function() {
                    const sidebar = document.querySelector('[class*="sidebar"]');
                    if (!sidebar) return 'No sidebar';

                    // Find all clickable nav items
                    const buttons = sidebar.querySelectorAll('button, a, [role="button"], [role="tab"], [class*="navItem"], [class*="NavItem"]');
                    const items = [];
                    buttons.forEach((b, i) => {
                        const text = b.textContent?.trim()?.substring(0, 50);
                        const aria = b.getAttribute('aria-label');
                        const title = b.getAttribute('title');
                        if (text || aria) {
                            items.push({ index: i, text, aria, title, tag: b.tagName, classes: b.className?.substring(0, 60) });
                        }
                    });
                    return JSON.stringify(items.slice(0, 20));
                })()`
            });
            console.log('=== Sidebar Navigation ===');
            const navItems = JSON.parse(nav?.result?.value || '[]');
            navItems.forEach(item => console.log(`[${item.index}] ${item.text || item.aria || item.title} (${item.tag})`));

            // Find "学习" or "resource" or "library" or Books icon buttons
            const libBtn = await send('Runtime.evaluate', {
                expression: `(function() {
                    // Find buttons with icon-text related to learning/resources
                    const allBtns = document.querySelectorAll('button');
                    const matches = [];
                    allBtns.forEach((b, i) => {
                        const text = (b.textContent || '').trim();
                        const aria = b.getAttribute('aria-label') || '';
                        const title = b.getAttribute('title') || '';
                        const inner = b.innerHTML || '';
                        if (text.match(/学习|资源|库|教材|书本|notebook|library|book|file|folder|sidebar/i) ||
                            aria.match(/学习|资源|库|教材|书本|notebook|library|book|file|folder/i)) {
                            matches.push({ index: i, text: text.substring(0,40), aria: aria.substring(0,40), title: title.substring(0,40), classes: b.className?.substring(0,60) });
                        }
                    });
                    return JSON.stringify(matches.slice(0, 10));
                })()`
            });
            console.log('\n=== Learning/Resource Buttons ===');
            const libBtns = JSON.parse(libBtn?.result?.value || '[]');
            libBtns.forEach(b => console.log(`[${b.index}] text="${b.text}" aria="${b.aria}"`));

            // Check for DSTU finder or resource grid
            const finder = await send('Runtime.evaluate', {
                expression: `(function() {
                    const elements = document.querySelectorAll('[class*="finder"], [class*="Finder"], [class*="resourceGrid"], [class*="ResourceGrid"], [class*="dstu"], [class*="DSTU"], [class*="learningHub"], [class*="LearningHub"]');
                    const info = [];
                    elements.forEach(el => {
                        const text = el.textContent?.substring(0, 100);
                        info.push({ classes: el.className?.substring(0, 80), text });
                    });
                    return JSON.stringify(info.slice(0, 10));
                })()`
            });
            console.log('\n=== Finder/Resource Elements ===');
            const finderInfo = JSON.parse(finder?.result?.value || '[]');
            finderInfo.forEach(f => console.log('  ' + f.classes + ' | ' + (f.text || '').substring(0, 60)));

            ws.close();
            process.exit(0);
        });
    });
}).on('error', (e) => console.error(e));
