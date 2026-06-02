#!/usr/bin/env node

/**
 * PaddleOCR MCP Server
 *
 * Implements the Model Context Protocol (MCP) over stdio to wrap the
 * PaddleOCR REST API (AI Studio) as a local MCP tool server.
 *
 * ## API Endpoints
 * - Submit job: POST https://paddleocr.aistudio-app.com/api/v2/ocr/jobs
 * - Poll status: GET https://paddleocr.aistudio-app.com/api/v2/ocr/jobs/{jobId}
 * - Download result: GET {resultUrl}
 *
 * ## Available Models
 * - PaddleOCR-VL-1.6 (default)
 * - PaddleOCR-VL-1.5
 * - PP-OCRv5
 * - PP-StructureV3
 *
 * ## Usage
 *   export PADDLEOCR_API_TOKEN="your_token_here"
 *   node scripts/mcp-paddleocr-server.mjs
 *
 * Or with the app's MCP config:
 *   {
 *     "command": "node",
 *     "args": ["scripts/mcp-paddleocr-server.mjs"],
 *     "env": { "PADDLEOCR_API_TOKEN": "..." }
 *   }
 *
 * @see https://paddlepaddle.github.io/PaddleOCR/
 */

// ============================================================================
// Imports
// ============================================================================

// ============================================================================
// Constants
// ============================================================================

const API_BASE = 'https://paddleocr.aistudio-app.com/api/v2';
const POLL_INTERVAL_MS = 3_000;
const MAX_POLL_ATTEMPTS = 120; // 6 minutes
const SERVER_NAME = 'paddleocr-mcp';
const SERVER_VERSION = '0.1.0';
const PROTOCOL_VERSION = '2025-06-18';

const MODELS = [
  { id: 'PaddleOCR-VL-1.6',    description: 'Latest VL model, high accuracy, supports layout + markdown' },
  { id: 'PaddleOCR-VL-1.5',    description: 'VL 1.5, 109 languages, 94.5% accuracy, supports layout + markdown' },
  { id: 'PP-OCRv5',            description: 'Lightweight OCR, faster but text-over-image output only' },
  { id: 'PP-StructureV3',      description: 'Document layout analysis + OCR, outputs markdown with structure' },
];

// ============================================================================
// Logging
// ============================================================================

function log(...args) {
  // MCP stdio protocol: all logging must go to stderr
  process.stderr.write(`[${SERVER_NAME}] ${args.join(' ')}\n`);
}

// ============================================================================
// REST API Client
// ============================================================================

class PaddleOcrClient {
  #token;

  constructor(token) {
    this.#token = token;
  }

  /** Submit an OCR job for a local file (multipart upload) */
  async submitFile(filePath, model) {
    const fs = await import('node:fs');
    const fileBuffer = fs.readFileSync(filePath);
    const fileName = filePath.split(/[\\/]/).pop() || 'document.pdf';

    // Build multipart form manually to avoid external dependencies
    const boundary = `----FormBoundary${Date.now()}`;
    const encoder = new TextEncoder();

    const modelPart = this.#encodeField('model', model, boundary);
    const optionalPayload = JSON.stringify({
      useDocOrientationClassify: false,
      useDocUnwarping: false,
      useChartRecognition: model.includes('PP-OCRv5'),
      useTextlineOrientation: model.includes('PP-OCRv5'),
    });
    const optionalPart = this.#encodeField('optionalPayload', optionalPayload, boundary);
    const filePart = this.#encodeFile('file', fileName, fileBuffer, boundary);
    const closing = encoder.encode(`\r\n--${boundary}--\r\n`);

    const body = Buffer.concat([
      Buffer.from(modelPart),
      Buffer.from(optionalPart),
      filePart,
      closing,
    ]);

    const resp = await fetch(`${API_BASE}/ocr/jobs`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.#token}`,
        'Content-Type': `multipart/form-data; boundary=${boundary}`,
      },
      body,
    });

    if (!resp.ok) {
      const text = await resp.text();
      throw new Error(`Job submission failed (${resp.status}): ${text}`);
    }

    const data = await resp.json();
    return data.data.jobId;
  }

  /** Submit an OCR job for a remote URL */
  async submitUrl(fileUrl, model) {
    const optionalPayload = {
      useDocOrientationClassify: false,
      useDocUnwarping: false,
      useChartRecognition: model.includes('PP-OCRv5'),
      useTextlineOrientation: model.includes('PP-OCRv5'),
    };

    const resp = await fetch(`${API_BASE}/ocr/jobs`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.#token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model,
        fileUrl,
        optionalPayload,
      }),
    });

    if (!resp.ok) {
      const text = await resp.text();
      throw new Error(`URL job submission failed (${resp.status}): ${text}`);
    }

    const data = await resp.json();
    return data.data.jobId;
  }

  /** Poll a job until completion, return the result JSON URL */
  async pollJob(jobId) {
    for (let attempt = 0; attempt < MAX_POLL_ATTEMPTS; attempt++) {
      const resp = await fetch(`${API_BASE}/ocr/jobs/${jobId}`, {
        headers: { 'Authorization': `Bearer ${this.#token}` },
      });

      if (!resp.ok) {
        throw new Error(`Poll failed (${resp.status})`);
      }

      const status = await resp.json();
      const state = status.data.state;

      if (state === 'done') {
        const resultUrl = status.data.resultUrl?.jsonUrl;
        if (!resultUrl) throw new Error('Result URL missing');
        log(`Job ${jobId} completed: ${status.data.extractProgress?.extractedPages ?? 0} pages`);
        return resultUrl;
      }

      if (state === 'failed') {
        throw new Error(`Job failed: ${status.data.errorMsg || 'Unknown error'}`);
      }

      await sleep(POLL_INTERVAL_MS);
    }

    throw new Error(`Job ${jobId} timed out after ${MAX_POLL_ATTEMPTS} attempts`);
  }

  /** Download and parse JSONL results */
  async downloadResult(jsonlUrl) {
    const resp = await fetch(jsonlUrl);
    if (!resp.ok) {
      throw new Error(`Download failed (${resp.status})`);
    }

    const text = await resp.text();
    const lines = text.trim().split('\n').filter(Boolean);
    const pages = [];

    for (const line of lines) {
      try {
        const parsed = JSON.parse(line);
        const result = parsed.result;

        // VL series / StructureV3: layoutParsingResults -> markdown
        if (result.layoutParsingResults) {
          for (const item of result.layoutParsingResults) {
            const md = item.markdown || {};
            pages.push({
              pageIndex: pages.length,
              markdownText: md.text || '',
              images: Object.entries(md.images || {}).map(([name, url]) => ({ name, url })),
            });
          }
        }

        // PP-OCRv5: ocrResults -> ocrImage
        if (result.ocrResults) {
          for (const item of result.ocrResults) {
            pages.push({
              pageIndex: pages.length,
              markdownText: '',
              images: [{ name: `ocr_page_${pages.length}`, url: item.ocrImage }],
            });
          }
        }
      } catch (e) {
        log(`Warning: failed to parse JSONL line: ${e.message}`);
      }
    }

    return pages;
  }

  /** Convenience: submit + poll + download */
  async ocrFile(filePath, model) {
    const jobId = await this.submitFile(filePath, model);
    log(`Job submitted: ${jobId} for file ${filePath}`);
    const jsonlUrl = await this.pollJob(jobId);
    return this.downloadResult(jsonlUrl);
  }

  /** Convenience: submit URL + poll + download */
  async ocrUrl(fileUrl, model) {
    const jobId = await this.submitUrl(fileUrl, model);
    log(`Job submitted: ${jobId} for URL ${fileUrl}`);
    const jsonlUrl = await this.pollJob(jobId);
    return this.downloadResult(jsonlUrl);
  }

  // ======================================================================
  // Internal helpers
  // ======================================================================

  #encodeField(name, value, boundary) {
    const encoder = new TextEncoder();
    const header = `\r\n--${boundary}\r\nContent-Disposition: form-data; name="${name}"\r\n\r\n${value}`;
    return encoder.encode(header);
  }

  #encodeFile(fieldName, fileName, buffer, boundary) {
    const encoder = new TextEncoder();
    const header = encoder.encode(
      `\r\n--${boundary}\r\nContent-Disposition: form-data; name="${fieldName}"; filename="${fileName}"\r\nContent-Type: application/octet-stream\r\n\r\n`
    );
    return Buffer.concat([Buffer.from(header), buffer]);
  }
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// ============================================================================
// MCP Server (JSON-RPC 2.0 over stdio)
// ============================================================================

class McpServer {
  #client;
  #requestId = 0;
  #buffer = '';

  constructor(client) {
    this.#client = client;
  }

  /** Start reading JSON-RPC requests from stdin */
  start() {
    process.stdin.setEncoding('utf-8');
    process.stdin.on('data', (chunk) => {
      this.#buffer += chunk;
      this.#processBuffer();
    });
    process.stdin.on('end', () => {
      // Clean exit
    });

    // Log that we started
    log(`Server started (protocol: ${PROTOCOL_VERSION})`);

    // If we're idle for a while, stderr heartbeat so the parent knows we're alive
    setInterval(() => {
      log(`Heartbeat: alive`);
    }, 60_000).unref();
  }

  #processBuffer() {
    // Content-Length framing: "Content-Length: N\r\n\r\n{body}"
    let idx;
    while ((idx = this.#buffer.indexOf('\r\n\r\n')) !== -1) {
      const header = this.#buffer.slice(0, idx);
      const match = header.match(/Content-Length:\s*(\d+)/i);
      if (!match) {
        // JSONL fallback: try line-by-line
        const nlIdx = this.#buffer.indexOf('\n');
        if (nlIdx === -1) break;
        const line = this.#buffer.slice(0, nlIdx).trim();
        this.#buffer = this.#buffer.slice(nlIdx + 1);
        if (line) this.#handleMessage(line);
        continue;
      }

      const contentLength = parseInt(match[1], 10);
      const bodyStart = idx + 4;
      const totalLen = bodyStart + contentLength;

      if (this.#buffer.length < totalLen) break; // wait for more data

      const body = this.#buffer.slice(bodyStart, totalLen);
      this.#buffer = this.#buffer.slice(totalLen);

      if (body.trim()) {
        this.#handleMessage(body);
      }
    }

    // Also try JSONL mode for remaining buffer
    const lines = this.#buffer.split('\n');
    if (lines.length > 1) {
      // Keep the last (potentially incomplete) line in buffer
      this.#buffer = lines.pop();
      for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed) this.#handleMessage(trimmed);
      }
    }
  }

  #handleMessage(raw) {
    let msg;
    try {
      msg = JSON.parse(raw);
    } catch (e) {
      log(`Failed to parse message: ${e.message}`);
      return;
    }

    // Notification (no id)
    if (msg.method && msg.id === undefined) {
      this.#handleNotification(msg).catch(e => log(`Notification error: ${e.message}`));
      return;
    }

    // Request (has id)
    if (msg.method && msg.id !== undefined && msg.id !== null) {
      this.#handleRequest(msg).catch(e => {
        log(`Request error: ${e.message}`);
        this.#sendError(msg.id, -32603, `Internal error: ${e.message}`);
      });
      return;
    }

    log(`Unknown message type: ${JSON.stringify(raw).slice(0, 200)}`);
  }

  async #handleNotification(msg) {
    // MCP spec requires responding to "initialized" notification
    if (msg.method === 'initialized') {
      log('Client initialized');
    }
    // We don't currently emit notifications that need handling
  }

  async #handleRequest(msg) {
    const { method, params, id } = msg;

    switch (method) {
      case 'initialize':
        await this.#handleInitialize(id, params);
        break;

      case 'tools/list':
        await this.#handleToolsList(id, params);
        break;

      case 'tools/call':
        await this.#handleToolCall(id, params);
        break;

      case 'ping':
        this.#sendResult(id, {});
        break;

      default:
        log(`Unknown method: ${method}`);
        this.#sendError(id, -32601, `Method not found: ${method}`);
    }
  }

  async #handleInitialize(id, params) {
    const clientVersion = params?.protocolVersion || 'unknown';
    log(`Initialize requested (client protocol: ${clientVersion})`);

    this.#sendResult(id, {
      protocolVersion: PROTOCOL_VERSION,
      capabilities: {
        tools: {
          listChanged: false,
        },
      },
      serverInfo: {
        name: SERVER_NAME,
        version: SERVER_VERSION,
      },
    });
  }

  async #handleToolsList(id, _params) {
    const tools = [
      {
        name: 'paddleocr_ocr_file',
        description: `Submit a local file to PaddleOCR for text extraction. Supports PDFs and images. Returns markdown text per page plus extracted images. Available models: ${MODELS.map(m => m.id).join(', ')}.`,
        inputSchema: {
          type: 'object',
          properties: {
            filePath: {
              type: 'string',
              description: 'Absolute path to the local file (PDF, PNG, JPG, etc.)',
            },
            model: {
              type: 'string',
              description: 'PaddleOCR model to use',
              enum: MODELS.map(m => m.id),
              default: 'PaddleOCR-VL-1.6',
            },
          },
          required: ['filePath'],
        },
      },
      {
        name: 'paddleocr_ocr_url',
        description: `Submit an online document URL to PaddleOCR for text extraction. The URL must be directly accessible by the PaddleOCR API (not behind auth). Returns markdown text per page plus extracted images. Available models: ${MODELS.map(m => m.id).join(', ')}.`,
        inputSchema: {
          type: 'object',
          properties: {
            fileUrl: {
              type: 'string',
              description: 'Publicly accessible URL of the document (PDF, PNG, JPG, etc.)',
            },
            model: {
              type: 'string',
              description: 'PaddleOCR model to use',
              enum: MODELS.map(m => m.id),
              default: 'PaddleOCR-VL-1.6',
            },
          },
          required: ['fileUrl'],
        },
      },
      {
        name: 'paddleocr_list_models',
        description: 'List all available PaddleOCR models with their descriptions.',
        inputSchema: {
          type: 'object',
          properties: {},
        },
      },
      {
        name: 'paddleocr_check_health',
        description: 'Check whether the PaddleOCR API token is configured and valid by submitting a lightweight probe.',
        inputSchema: {
          type: 'object',
          properties: {},
        },
      },
    ];

    this.#sendResult(id, { tools });
  }

  async #handleToolCall(id, params) {
    const { name, arguments: args } = params || {};

    if (!name) {
      this.#sendError(id, -32602, 'Missing tool name');
      return;
    }

    try {
      switch (name) {
        case 'paddleocr_ocr_file':
          return await this.#callOcrFile(id, args);
        case 'paddleocr_ocr_url':
          return await this.#callOcrUrl(id, args);
        case 'paddleocr_list_models':
          return await this.#callListModels(id);
        case 'paddleocr_check_health':
          return await this.#callCheckHealth(id);
        default:
          this.#sendError(id, -32601, `Unknown tool: ${name}`);
      }
    } catch (e) {
      log(`Tool ${name} error: ${e.message}`);
      this.#sendResult(id, {
        content: [{ type: 'text', text: `Error: ${e.message}` }],
        isError: true,
      });
    }
  }

  async #callOcrFile(id, args) {
    const filePath = args?.filePath;
    if (!filePath) {
      this.#sendError(id, -32602, 'Missing required argument: filePath');
      return;
    }

    // Check if file exists
    try {
      await import('node:fs').then(fs => fs.promises.access(filePath));
    } catch {
      this.#sendError(id, -32602, `File not found: ${filePath}`);
      return;
    }

    const model = args?.model || 'PaddleOCR-VL-1.6';

    // Send progress notification
    this.#sendNotification('tools/call_progress', {
      id: String(id),
      progress: 0,
      message: `Submitting ${filePath} with model ${model}...`,
    });

    log(`OCR file: ${filePath} (model: ${model})`);
    const pages = await this.#client.ocrFile(filePath, model);

    this.#sendNotification('tools/call_progress', {
      id: String(id),
      progress: 100,
      message: `Completed: ${pages.length} pages`,
    });

    const resultText = pages.map((p, i) => {
      const header = `--- Page ${i + 1} ---`;
      const text = p.markdownText || '[No text extracted]';
      const images = p.images.length > 0
        ? `\n[Images: ${p.images.map(img => img.url).join(', ')}]`
        : '';
      return `${header}\n${text}${images}`;
    }).join('\n\n');

    this.#sendResult(id, {
      content: [
        {
          type: 'text',
          text: JSON.stringify({
            model,
            totalPages: pages.length,
            text: pages.map(p => p.markdownText).join('\n\n---\n\n'),
            images: pages.flatMap(p => p.images),
          }, null, 2),
        },
      ],
    });
  }

  async #callOcrUrl(id, args) {
    const fileUrl = args?.fileUrl;
    if (!fileUrl) {
      this.#sendError(id, -32602, 'Missing required argument: fileUrl');
      return;
    }

    const model = args?.model || 'PaddleOCR-VL-1.6';

    log(`OCR URL: ${fileUrl} (model: ${model})`);
    const pages = await this.#client.ocrUrl(fileUrl, model);

    this.#sendResult(id, {
      content: [
        {
          type: 'text',
          text: JSON.stringify({
            model,
            totalPages: pages.length,
            text: pages.map(p => p.markdownText).join('\n\n---\n\n'),
            images: pages.flatMap(p => p.images),
          }, null, 2),
        },
      ],
    });
  }

  async #callListModels(id) {
    this.#sendResult(id, {
      content: [
        {
          type: 'text',
          text: JSON.stringify(MODELS, null, 2),
        },
      ],
    });
  }

  async #callCheckHealth(id) {
    const token = process.env.PADDLEOCR_API_TOKEN || '';
    if (!token) {
      this.#sendResult(id, {
        content: [{ type: 'text', text: 'PADDLEOCR_API_TOKEN is not configured. Set the environment variable to use PaddleOCR.' }],
        isError: true,
      });
      return;
    }

    try {
      // Try reaching the API base to verify connectivity
      const resp = await fetch(`${API_BASE}/ocr/jobs`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          model: 'PaddleOCR-VL-1.6',
          fileUrl: 'https://example.com/test',
        }),
      });

      // A valid token will reach the API; an invalid one returns 401
      const reachable = resp.status !== 0;
      const tokenValid = resp.status !== 401 && resp.status !== 403;
      const statusCode = resp.status;

      this.#sendResult(id, {
        content: [
          {
            type: 'text',
            text: JSON.stringify({
              configured: true,
              apiReachable: reachable,
              tokenValid,
              statusCode,
              message: !tokenValid
                ? `API returned status ${statusCode} -- token may be invalid`
                : `API is reachable (status ${statusCode})`,
            }, null, 2),
          },
        ],
      });
    } catch (e) {
      this.#sendResult(id, {
        content: [{ type: 'text', text: `Health check failed: ${e.message}` }],
        isError: true,
      });
    }
  }

  // ======================================================================
  // JSON-RPC Response Helpers
  // ======================================================================

  #sendResult(id, result) {
    this.#writeMessage({
      jsonrpc: '2.0',
      id,
      result,
    });
  }

  #sendError(id, code, message, data) {
    const err = { code, message };
    if (data !== undefined) err.data = data;

    this.#writeMessage({
      jsonrpc: '2.0',
      id,
      error: err,
    });
  }

  #sendNotification(method, params) {
    this.#writeMessage({
      jsonrpc: '2.0',
      method,
      params,
    });
  }

  #writeMessage(msg) {
    const body = JSON.stringify(msg);
    const frame = `Content-Length: ${Buffer.byteLength(body, 'utf-8')}\r\n\r\n${body}`;
    process.stdout.write(frame);
  }
}

// ============================================================================
// Main
// ============================================================================

function main() {
  const token = process.env.PADDLEOCR_API_TOKEN || '';

  if (!token) {
    log('WARNING: PADDLEOCR_API_TOKEN environment variable is not set.');
    log('Set it to your PaddleOCR AI Studio API token to enable OCR functionality.');
    log('Usage: PADDLEOCR_API_TOKEN=your_token node scripts/mcp-paddleocr-server.mjs');
  }

  const client = new PaddleOcrClient(token);
  const server = new McpServer(client);
  server.start();

  log(`MCP server ready (${SERVER_NAME} v${SERVER_VERSION})`);
  log(`Token configured: ${token ? 'yes' : 'no'}`);
}

main();
