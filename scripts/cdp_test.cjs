#!/usr/bin/env node
const http = require('http');
const WebSocket = require('ws');

// Step 1: Get the main page's WebSocket URL
http.get('http://localhost:9222/json', (res) => {
    let data = '';
    res.on('data', chunk => data += chunk);
    res.on('end', async () => {
        const pages = JSON.parse(data);
        const page = pages.find(p => p.type === 'page');
        if (!page) { console.log('No page target found'); return; }

        console.log('Connected to:', page.title);
        const ws = new WebSocket(page.webSocketDebuggerUrl);

        ws.on('open', () => {
            runTests(ws);
        });

        ws.on('message', (data) => {
            const msg = JSON.parse(data.toString());
            if (msg.id && pending[msg.id]) {
                pending[msg.id](msg.result);
            }
        });
    });
}).on('error', (e) => console.error(e));

const pending = {};
let cmdId = 1;

function send(ws, method, params) {
    return new Promise(resolve => {
        const id = cmdId++;
        pending[id] = resolve;
        ws.send(JSON.stringify({ id, method, params }));
    });
}

async function runTests(ws) {
    // Enable Runtime
    await send(ws, 'Runtime.enable');

    // Test 1: Check app is loaded
    const title = await send(ws, 'Runtime.evaluate', { expression: 'document.title' });
    console.log('\n=== Test 1: App Title ===');
    console.log('Title:', title?.result?.value || 'N/A');

    // Test 2: Check if Learning Hub sidebar exists
    const sidebar = await send(ws, 'Runtime.evaluate', {
        expression: `(function() {
            const el = document.querySelector('[data-testid="learning-hub-sidebar"], .learning-hub-sidebar, [class*="learningHub"], [class*="sidebar"]');
            return el ? 'Sidebar found: ' + el.className.substring(0, 100) : 'No sidebar found';
        })()`
    });
    console.log('\n=== Test 2: Sidebar ===');
    console.log(sidebar?.result?.value || 'N/A');

    // Test 3: Check for resource list items (textbooks/files)
    const items = await send(ws, 'Runtime.evaluate', {
        expression: `(function() {
            // Look for any list items that might be resources
            const items = document.querySelectorAll('[data-resource-type], [data-item-type], [class*="finder"], [class*="resourceItem"], [class*="listItem"]');
            const types = [];
            items.forEach(i => {
                const type = i.getAttribute('data-resource-type') || i.getAttribute('data-item-type') || i.className.substring(0, 50);
                const text = i.textContent?.substring(0, 80);
                types.push({type, text});
            });
            return JSON.stringify(types.slice(0, 10));
        })()`
    });
    console.log('\n=== Test 3: Resource Items ===');
    console.log(items?.result?.value || 'No items');

    // Test 4: Check Tauri invoke — call dstu_list to verify backend
    const dstuList = await send(ws, 'Runtime.evaluate', {
        expression: `(async function() {
            try {
                const { invoke } = window.__TAURI__?.core || {};
                if (!invoke) return 'TAURI core not loaded';
                const result = await invoke('dstu_list', {
                    path: '/',
                    options: { limit: 20, offset: 0 }
                });
                const textbooks = result.filter(n => n.node_type === 'Textbook' || n.node_type === 'textbook');
                const files = result.filter(n => n.node_type === 'File' || n.node_type === 'file');
                const total = result.length;
                return 'Total items: ' + total + ', Textbooks: ' + textbooks.length + ', Files: ' + files.length;
            } catch(e) { return 'Error: ' + e.message || String(e); }
        })()`
    });
    console.log('\n=== Test 4: dstu_list root ===');
    console.log(dstuList?.result?.value || 'N/A');

    // Test 5: Verify a specific file_ resource exists via dstu_get
    const fileGet = await send(ws, 'Runtime.evaluate', {
        expression: `(async function() {
            try {
                const { invoke } = window.__TAURI__?.core || {};
                if (!invoke) return 'TAURI core not loaded';
                // Try to list with file type filter
                const result = await invoke('dstu_list', {
                    path: '/',
                    options: { typeFilter: 'file', limit: 5, offset: 0 }
                });
                return 'File-typed items: ' + result.length + ', IDs: ' + result.map(n => n.id).join(', ');
            } catch(e) { return 'Error: ' + e.message || String(e); }
        })()`
    });
    console.log('\n=== Test 5: dstu_list typeFilter=file ===');
    console.log(fileGet?.result?.value || 'N/A');

    // Test 6: Verify OCR availability
    const ocrCheck = await send(ws, 'Runtime.evaluate', {
        expression: `(async function() {
            try {
                const { invoke } = window.__TAURI__?.core || {};
                if (!invoke) return 'TAURI core not loaded';
                const result = await invoke('check_ocr_availability');
                return JSON.stringify(result);
            } catch(e) { return 'Error: ' + e.message || String(e); }
        })()`
    });
    console.log('\n=== Test 6: OCR Availability ===');
    console.log(ocrCheck?.result?.value || 'N/A');

    console.log('\n=== All Tests Complete ===');
    ws.close();
    process.exit(0);
}
