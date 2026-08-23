#!/usr/bin/env node
const http = require('http');

// Get available pages from CDP
http.get('http://localhost:9222/json', (res) => {
    let data = '';
    res.on('data', chunk => data += chunk);
    res.on('end', () => {
        try {
            const pages = JSON.parse(data);
            console.log('Available pages:', pages.length);
            pages.forEach((p, i) => {
                console.log(`[${i}] ${p.title} - ${p.url}`);
                console.log(`    type: ${p.type}, devtoolsFrontendUrl: ${p.devtoolsFrontendUrl ? 'yes' : 'no'}`);
            });
        } catch(e) {
            console.error('Parse error:', e.message);
            console.log('Raw:', data.substring(0, 500));
        }
    });
}).on('error', (e) => console.error('Connection error:', e.message));
