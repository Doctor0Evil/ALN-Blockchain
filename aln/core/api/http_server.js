/**
 * ALN HTTP/WebSocket API Server
 * 
 * Exposes blockchain node functionality via REST and WebSocket
 */

const express = require('express');
const WebSocket = require('ws');
const { Logger } = require('../logging/logger');
const { parseAlnDocument } = require('../runtime/aln_parser');

class HttpServer {
  constructor(consensus, stateStore, options = {}) {
    this.consensus = consensus;
    this.stateStore = stateStore;
    const { policyEngine = null, energyIntegration = null, ...config } = options;
    this.policyEngine = policyEngine;
    this.energyIntegration = energyIntegration;
    this.config = {
      apiPort: config.apiPort || 3000,
      wsPort: config.wsPort || 3001,
      ...config
    };
    this.logger = new Logger(config.nodeId || 'api_server');
    this.app = express();
    this.wsServer = null;
    this.wsClients = new Set();
  }

  /**
   * Initialize Express app and routes
   */
  initialize() {
    // Middleware
    this.app.use(express.json({ limit: '10mb' }));
    this.app.use((req, res, next) => {
      // CORS
      res.header('Access-Control-Allow-Origin', '*');
      res.header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
      res.header('Access-Control-Allow-Headers', 'Content-Type');
      
      // Request logging
      this.logger.info('HTTP request', {
        method: req.method,
        path: req.path,
        ip: req.ip
      });
      
      next();
    });

    // Routes
    this._setupRoutes();
  }

  /**
   * Setup API routes
   */
  _setupRoutes() {
    // GET /status - Node status
    this.app.get('/status', (req, res) => {
      const status = this.consensus.getStatus();
      res.json({
        success: true,
        data: status
      });
    });

    // GET /account/:addr - Account details
    this.app.get('/account/:addr', async (req, res) => {
      try {
        const { addr } = req.params;
        const account = await this.stateStore.getAccount(addr);
        
        if (!account) {
          return res.status(404).json({
            success: false,
            error: 'Account not found'
          });
        }

        res.json({
          success: true,
          data: account
        });
      } catch (err) {
        this.logger.error('Account query failed', { error: err.message });
        res.status(500).json({
          success: false,
          error: err.message
        });
      }
    });

    // POST /tx - Submit transaction (supports chat-native metadata)
    this.app.post('/tx', async (req, res) => {
      try {
        const chainlexeme = req.body;
        
        // Parse if submitted as text
        let parsed = chainlexeme;
        if (typeof chainlexeme === 'string') {
          parsed = parseAlnDocument(chainlexeme);
        }

        // Normalize chat-native metadata when provided separately
        if (parsed && parsed.header) {
          if (req.body.chat_context_id && !parsed.header.chat_context_id) {
            parsed.header.chat_context_id = req.body.chat_context_id;
          }
          if (req.body.transcript_hash && !parsed.header.transcript_hash) {
            parsed.header.transcript_hash = req.body.transcript_hash;
          }
          if (req.body.jurisdiction_tags && !parsed.header.jurisdiction_tags) {
            parsed.header.jurisdiction_tags = req.body.jurisdiction_tags;
          }
        }

        if (this.policyEngine) {
          const policyResult = this.policyEngine.validateTransaction(parsed);
          if (!policyResult.allowed) {
            return res.status(400).json({
              success: false,
              error: 'Policy check failed',
              details: policyResult.reason
            });
          }
        }

        const result = this.consensus.submitTransaction(parsed);
        
        if (result.success) {
          res.json({
            success: true,
            data: {
              txHash: result.txHash,
              mempoolSize: result.mempoolSize,
              chat_context_id: parsed.header.chat_context_id || null,
              transcript_hash: parsed.header.transcript_hash || null
            }
          });
        } else {
          res.status(400).json({
            success: false,
            error: result.error,
            details: result.details
          });
        }
      } catch (err) {
        this.logger.error('Transaction submission failed', { error: err.message });
        res.status(500).json({
          success: false,
          error: err.message
        });
      }
    });

    // GET /block/:height - Block details
    this.app.get('/block/:height', (req, res) => {
      try {
        const height = parseInt(req.params.height);
        const block = this.consensus.getBlock(height);
        
        if (!block) {
          return res.status(404).json({
            success: false,
            error: 'Block not found'
          });
        }

        res.json({
          success: true,
          data: block.toJSON()
        });
      } catch (err) {
        this.logger.error('Block query failed', { error: err.message });
        res.status(500).json({
          success: false,
          error: err.message
        });
      }
    });

    // GET /recent-blocks - Recent blocks for activity chart
    this.app.get('/recent-blocks', (req, res) => {
      try {
        const limit = parseInt(req.query.limit || 50);
        const status = this.consensus.getStatus();
        const currentHeight = status.height;
        
        const blocks = [];
        for (let i = Math.max(0, currentHeight - limit); i <= currentHeight; i++) {
          const block = this.consensus.getBlock(i);
          if (block) {
            blocks.push({
              height: block.header.height,
              hash: block.header.hash,
              timestamp: block.header.timestamp,
              txCount: block.transactions.length
            });
          }
        }

        res.json({
          success: true,
          data: blocks
        });
      } catch (err) {
        this.logger.error('Recent blocks query failed', { error: err.message });
        res.status(500).json({
          success: false,
          error: err.message
        });
      }
    });

    // Health check
    this.app.get('/health', (req, res) => {
      res.json({ status: 'ok', timestamp: Date.now() });
    });

    // Metrics
    this.app.get('/metrics', (req, res) => {
      const status = this.consensus.getStatus();
      res.json({
        success: true,
        data: {
          block_height: status.height,
          mempool_size: status.mempoolSize,
          ws_clients: this.wsClients.size,
          uptime_seconds: process.uptime()
        }
      });
    });

    // Policy endpoints
    this.app.get('/policy/user/:did', (req, res) => {
      if (!this.policyEngine) {
        return res.status(503).json({ success: false, error: 'Policy engine unavailable' });
      }
      const policy = this.policyEngine.getEffectivePolicy(req.params.did);
      res.json({ success: true, data: policy });
    });

    this.app.get('/energy/state/:did', (req, res) => {
      if (!this.energyIntegration) {
        return res.status(503).json({ success: false, error: 'Energy integration unavailable' });
      }
      const state = this.energyIntegration.getEnergyState(req.params.did);
      res.json({ success: true, data: state });
    });

    this.app.post('/policy/check', (req, res) => {
      if (!this.policyEngine) {
        return res.status(503).json({ success: false, error: 'Policy engine unavailable' });
      }
      const { user, action_id: actionId, energy_state: runtimeEnergy } = req.body || {};
      if (!user || !actionId) {
        return res.status(400).json({ success: false, error: 'user and action_id required' });
      }

      const energyState = runtimeEnergy || this.energyIntegration?.getEnergyState(user);
      const decision = this.policyEngine.isActionAllowed(user, actionId, energyState);
      res.json({ success: true, data: decision });
    });
  }

  /**
   * Start HTTP server
   */
  start() {
    this.initialize();

    // Start HTTP server
    this.app.listen(this.config.apiPort, () => {
      this.logger.info(`HTTP API listening on port ${this.config.apiPort}`);
    });

    // Start WebSocket server
    this.startWebSocket();

    // Subscribe to new blocks
    this.consensus.onNewBlock((blockData) => {
      this.broadcastToClients({
        type: 'newBlock',
        data: blockData
      });
    });
  }

  /**
   * Start WebSocket server
   */
  startWebSocket() {
    this.wsServer = new WebSocket.Server({ port: this.config.wsPort });

    this.wsServer.on('connection', (ws) => {
      this.logger.info('WebSocket client connected');
      this.wsClients.add(ws);

      ws.on('close', () => {
        this.logger.info('WebSocket client disconnected');
        this.wsClients.delete(ws);
      });

      ws.on('error', (err) => {
        this.logger.error('WebSocket error', { error: err.message });
        this.wsClients.delete(ws);
      });

      // Send initial status
      const status = this.consensus.getStatus();
      ws.send(JSON.stringify({
        type: 'status',
        data: status
      }));
    });

    this.logger.info(`WebSocket server listening on port ${this.config.wsPort}`);
  }

  /**
   * Broadcast message to all WebSocket clients
   */
  broadcastToClients(message) {
    const payload = JSON.stringify(message);
    
    this.wsClients.forEach((client) => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(payload);
      }
    });
  }
}

module.exports = HttpServer;
