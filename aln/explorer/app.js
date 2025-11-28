/**
 * ALN Explorer Application Logic
 * 
 * Handles UI updates, API polling, chart rendering, and wallet integration
 */

// Configuration
const CONFIG = {
  API_URL: 'http://localhost:3000',
  WS_URL: 'ws://localhost:3001',
  POLL_INTERVAL: 5000, // 5 seconds
  STORAGE_PREFIX: 'ALN_'
};

// State
let state = {
  connected: false,
  wsConnection: null,
  walletAddress: null,
  walletBalance: '0',
  migrationHistory: [],
  activityData: [],
  proposals: []
};

// Charts
let migragraphChart = null;
let activityChart = null;

// =============================================================================
// Initialization
// =============================================================================

document.addEventListener('DOMContentLoaded', () => {
  console.log('🚀 ALN Explorer initializing...');
  
  initCharts();
  loadStoredData();
  connectWebSocket();
  startPolling();
  setupEventListeners();

  console.log('✅ ALN Explorer ready');
});

// =============================================================================
// Storage Helpers
// =============================================================================

function setStorage(key, value) {
  try {
    localStorage.setItem(CONFIG.STORAGE_PREFIX + key, JSON.stringify(value));
  } catch (err) {
    console.warn('LocalStorage unavailable, using cookie:', err);
    // Fallback to cookie
    document.cookie = `${CONFIG.STORAGE_PREFIX}${key}=${encodeURIComponent(JSON.stringify(value))};max-age=31536000;path=/`;
  }
}

function getStorage(key) {
  try {
    const value = localStorage.getItem(CONFIG.STORAGE_PREFIX + key);
    return value ? JSON.parse(value) : null;
  } catch (err) {
    // Fallback to cookie
    const match = document.cookie.match(new RegExp(`${CONFIG.STORAGE_PREFIX}${key}=([^;]+)`));
    if (match) {
      try {
        return JSON.parse(decodeURIComponent(match[1]));
      } catch (e) {
        return null;
      }
    }
    return null;
  }
}

function loadStoredData() {
  state.walletAddress = getStorage('walletAddress');
  state.walletBalance = getStorage('walletBalance') || '0';
  state.migrationHistory = getStorage('migrationHistory') || [];
  state.activityData = getStorage('activityData') || [];
  
  if (state.walletAddress) {
    updateWalletUI();
  }
  
  if (state.migrationHistory.length > 0) {
    updateMigragraph();
  }
}

// =============================================================================
// Chart Initialization
// =============================================================================

function initCharts() {
  // Activity Chart
  const activityCtx = document.getElementById('activity_chart').getContext('2d');
  activityChart = new Chart(activityCtx, {
    type: 'line',
    data: {
      labels: [],
      datasets: [{
        label: 'Transactions per Block',
        data: [],
        borderColor: 'rgb(37, 99, 235)',
        backgroundColor: 'rgba(37, 99, 235, 0.1)',
        tension: 0.4,
        fill: true
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: true,
      scales: {
        y: {
          beginAtZero: true,
          ticks: { color: '#94a3b8' },
          grid: { color: 'rgba(255, 255, 255, 0.1)' }
        },
        x: {
          ticks: { color: '#94a3b8' },
          grid: { color: 'rgba(255, 255, 255, 0.1)' }
        }
      },
      plugins: {
        legend: {
          labels: { color: '#f1f5f9' }
        }
      }
    }
  });

  // Migragraph Chart
  const migragraphCtx = document.getElementById('migragraph_chart').getContext('2d');
  migragraphChart = new Chart(migragraphCtx, {
    type: 'line',
    data: {
      labels: [],
      datasets: [{
        label: 'Cumulative Migrated (ALN)',
        data: [],
        borderColor: 'rgb(124, 58, 237)',
        backgroundColor: 'rgba(124, 58, 237, 0.1)',
        tension: 0.4,
        fill: true
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: true,
      scales: {
        y: {
          beginAtZero: true,
          ticks: { color: '#94a3b8' },
          grid: { color: 'rgba(255, 255, 255, 0.1)' }
        },
        x: {
          ticks: { color: '#94a3b8' },
          grid: { color: 'rgba(255, 255, 255, 0.1)' }
        }
      },
      plugins: {
        legend: {
          labels: { color: '#f1f5f9' }
        }
      }
    }
  });
}

// =============================================================================
// WebSocket Connection
// =============================================================================

function connectWebSocket() {
  try {
    state.wsConnection = new WebSocket(CONFIG.WS_URL);

    state.wsConnection.onopen = () => {
      console.log('✅ WebSocket connected');
      state.connected = true;
      updateConnectionStatus();
    };

    state.wsConnection.onmessage = (event) => {
      const message = JSON.parse(event.data);
      handleWebSocketMessage(message);
    };

    state.wsConnection.onclose = () => {
      console.log('❌ WebSocket disconnected');
      state.connected = false;
      updateConnectionStatus();
      
      // Reconnect after 5 seconds
      setTimeout(connectWebSocket, 5000);
    };

    state.wsConnection.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  } catch (err) {
    console.error('Failed to connect WebSocket:', err);
  }
}

function handleWebSocketMessage(message) {
  switch (message.type) {
    case 'status':
      updateStatus(message.data);
      break;
    case 'newBlock':
      updateStatus({ height: message.data.block.header.height });
      addActivityDataPoint(message.data.block);
      fetchRecentBlocks();
      break;
    default:
      console.log('Unknown message type:', message.type);
  }
}

// =============================================================================
// API Polling
// =============================================================================

function startPolling() {
  fetchStatus();
  fetchRecentBlocks();
  
  setInterval(() => {
    if (!state.connected) {
      fetchStatus();
      fetchRecentBlocks();
    }
  }, CONFIG.POLL_INTERVAL);
}

async function fetchStatus() {
  try {
    const response = await fetch(`${CONFIG.API_URL}/status`);
    const data = await response.json();
    
    if (data.success) {
      updateStatus(data.data);
    }
  } catch (err) {
    console.error('Failed to fetch status:', err);
    updateConnectionStatus(false);
  }
}

async function fetchRecentBlocks() {
  try {
    const response = await fetch(`${CONFIG.API_URL}/recent-blocks?limit=20`);
    const data = await response.json();
    
    if (data.success) {
      updateBlocksList(data.data);
      updateActivityChart(data.data);
    }
  } catch (err) {
    console.error('Failed to fetch recent blocks:', err);
  }
}

// =============================================================================
// UI Updates
// =============================================================================

function updateStatus(data) {
  document.getElementById('block_height').textContent = data.height || 0;
  
  if (data.latestBlockHash) {
    const hashElem = document.getElementById('latest_block_hash');
    hashElem.textContent = data.latestBlockHash.substring(0, 16) + '...';
    hashElem.title = data.latestBlockHash;
  }
  
  document.getElementById('mempool_size').textContent = `${data.mempoolSize || 0} tx`;
  
  const statusElem = document.getElementById('node_status');
  if (data.isRunning || state.connected) {
    statusElem.textContent = '🟢 Online';
    statusElem.className = 'value badge success';
  } else {
    statusElem.textContent = '🔴 Offline';
    statusElem.className = 'value badge failure';
  }
  
  document.getElementById('last_update').textContent = new Date().toLocaleTimeString();
  
  // Store status
  setStorage('lastStatus', data);
}

function updateConnectionStatus(connected = state.connected) {
  const statusElem = document.getElementById('node_status');
  if (connected) {
    statusElem.textContent = '🟢 Online';
    statusElem.className = 'value badge success';
  } else {
    statusElem.textContent = '🔴 Offline';
    statusElem.className = 'value badge failure';
  }
}

function updateBlocksList(blocks) {
  const listElem = document.getElementById('blocks_list');
  
  if (blocks.length === 0) {
    listElem.innerHTML = '<p class="empty-state">No blocks yet</p>';
    return;
  }
  
  listElem.innerHTML = blocks.slice().reverse().slice(0, 10).map(block => `
    <div class="block-item">
      <div class="block-height">Block #${block.height}</div>
      <div class="block-hash">${block.hash.substring(0, 32)}...</div>
      <div>${block.txCount} transactions</div>
    </div>
  `).join('');
}

function updateActivityChart(blocks) {
  if (blocks.length === 0) return;
  
  // Take last 20 blocks
  const recentBlocks = blocks.slice(-20);
  
  activityChart.data.labels = recentBlocks.map(b => `#${b.height}`);
  activityChart.data.datasets[0].data = recentBlocks.map(b => b.txCount);
  activityChart.update();
  
  // Store for persistence
  state.activityData = recentBlocks;
  setStorage('activityData', recentBlocks);
}

function addActivityDataPoint(block) {
  state.activityData.push({
    height: block.header.height,
    txCount: block.transactions.length,
    timestamp: block.header.timestamp
  });
  
  // Keep only last 50
  if (state.activityData.length > 50) {
    state.activityData = state.activityData.slice(-50);
  }
  
  updateActivityChart(state.activityData);
}

function updateMigragraph() {
  if (state.migrationHistory.length === 0) return;
  
  migragraphChart.data.labels = state.migrationHistory.map((_, i) => `Migration ${i + 1}`);
  
  // Calculate cumulative
  let cumulative = 0;
  const cumulativeData = state.migrationHistory.map(m => {
    cumulative += parseFloat(m.amount);
    return cumulative;
  });
  
  migragraphChart.data.datasets[0].data = cumulativeData;
  migragraphChart.update();
}

function pushMigragraphPoint(assetAmountMoved) {
  state.migrationHistory.push({
    amount: assetAmountMoved,
    timestamp: Date.now()
  });
  
  setStorage('migrationHistory', state.migrationHistory);
  updateMigragraph();
  
  const statusElem = document.getElementById('migration_status');
  statusElem.textContent = '✅ Active';
  statusElem.className = 'value badge success';
}

// =============================================================================
// Wallet Functions
// =============================================================================

function updateWalletUI() {
  if (state.walletAddress) {
    document.getElementById('wallet_address').textContent = 
      state.walletAddress.substring(0, 20) + '...';
    document.getElementById('wallet_address').title = state.walletAddress;
    document.getElementById('wallet_balance').textContent = `${state.walletBalance} ALN`;
    document.getElementById('wallet_actions').style.display = 'block';
  }
}

function generateWallet() {
  // Simple mock wallet generation (in production, use proper crypto)
  const randomBytes = new Uint8Array(20);
  crypto.getRandomValues(randomBytes);
  const hex = Array.from(randomBytes).map(b => b.toString(16).padStart(2, '0')).join('');
  
  state.walletAddress = `aln1${hex}`;
  state.walletBalance = '0';
  
  setStorage('walletAddress', state.walletAddress);
  setStorage('walletBalance', state.walletBalance);
  
  updateWalletUI();
  alert(`New wallet generated!\n\n${state.walletAddress}\n\n⚠️ This is a development wallet. Private keys never leave your browser.`);
}

async function sendTransaction(to, amount, memo) {
  if (!state.walletAddress) {
    alert('Please generate or connect a wallet first');
    return;
  }
  
  const chainlexeme = {
    header: {
      op_code: 'transfer',
      from: state.walletAddress,
      to: to,
      nonce: Math.floor(Math.random() * 1000000)
    },
    data: {
      asset: 'ALN',
      amount: amount,
      memo: memo || '',
      constraints: []
    },
    footer: {
      signature: `ed25519:0x${Math.random().toString(16).substring(2)}`, // Mock signature
      timestamp: Math.floor(Date.now() / 1000),
      gas_limit: 21000,
      gas_price: 100
    }
  };
  
  try {
    const response = await fetch(`${CONFIG.API_URL}/tx`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(chainlexeme)
    });
    
    const result = await response.json();
    
    if (result.success) {
      alert(`✅ Transaction submitted!\n\nTx Hash: ${result.data.txHash.substring(0, 16)}...`);
      document.getElementById('send_form').reset();
    } else {
      alert(`❌ Transaction failed:\n\n${result.error}`);
    }
  } catch (err) {
    alert(`❌ Network error:\n\n${err.message}`);
  }
}

// =============================================================================
// Event Listeners
// =============================================================================

function setupEventListeners() {
  document.getElementById('generate_wallet_btn').addEventListener('click', generateWallet);
  
  document.getElementById('send_form').addEventListener('submit', (e) => {
    e.preventDefault();
    const to = document.getElementById('send_to').value;
    const amount = document.getElementById('send_amount').value;
    const memo = document.getElementById('send_memo').value;
    sendTransaction(to, amount, memo);
  });
  
  document.getElementById('create_proposal_btn').addEventListener('click', () => {
    alert('Governance proposal creation UI coming soon!\n\nFor now, use the API directly or submit via chainlexeme.');
  });
}
