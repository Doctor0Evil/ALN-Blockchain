# ALN Blockchain Workspace - Setup Complete! 🎉

## ✅ What Has Been Created

Your ALN blockchain workspace is now fully scaffolded with:

### 📦 Core Modules

1. **`/aln/core/`** - Blockchain Runtime
   - ALN syntax parser & validator
   - State management with LevelDB
   - Block and ledger structures  
   - Solo consensus engine
   - HTTP/WebSocket API server
   - QPU.Math+ safety hooks
   - Structured logging with error codes
   - CLI for node management

2. **`/aln/explorer/`** - Web Explorer UI
   - Real-time status dashboard
   - Migragraph chart (Canto→ALN migrations)
   - Activity chart (blocks & transactions)
   - Non-custodial wallet interface
   - Governance proposals viewer

3. **`/aln/wallet/`** - Transaction Builder
   - Non-custodial key management
   - Chainlexeme transaction builder
   - Transfer, governance, migration tx types

4. **`/aln/dao/`** - Governance System
   - CHATAI token specification
   - Governogram proposal format
   - Voting and delegation

5. **`/aln/migration/`** - Cross-Chain Bridge
   - Migration state tracking
   - Migragraph data structures
   - Canto integration specs

6. **`/aln/tests/`** - Test Suite
   - Unit tests for parser & safety hooks
   - Jest configuration

7. **`/aln/tools/`** - Development Tools
   - ALN linter for chainlexemes

## 🚀 Next Steps

### Step 1: Install Node.js (if not already installed)

Download and install Node.js 18+ from: https://nodejs.org/

Verify installation:
```powershell
node --version
npm --version
```

### Step 2: Install Dependencies

```powershell
cd 'c:\Users\Hunter\Repos\ALN+Blockchain'
npm install
```

### Step 3: Initialize and Start Node

```powershell
cd aln\core
node cli\aln_node_cli.js init
node cli\aln_node_cli.js start
```

### Step 4: Start Explorer (in new terminal)

```powershell
cd aln\explorer
npm start
```

Then open: http://localhost:8080

### Step 5: Check Status

```powershell
node cli\aln_node_cli.js status
```

Or use curl:
```powershell
curl http://localhost:3000/status
```

## 📁 Project Structure

```
ALN+Blockchain/
├── .github/
│   ├── copilot-instructions.md     # Workspace instructions
│   └── workflows/
│       └── aln-ci.yml              # CI/CD pipeline
├── aln/
│   ├── core/                       # Blockchain runtime
│   │   ├── api/                    # HTTP/WS server
│   │   ├── cli/                    # Node CLI
│   │   ├── config/                 # Nanotopologene profile
│   │   ├── consensus/              # Solo consensus
│   │   ├── ledger/                 # Block model
│   │   ├── logging/                # Structured logging
│   │   ├── runtime/                # ALN parser
│   │   ├── safety/                 # QPU.Math+ hooks
│   │   ├── spec/                   # ALN syntax spec
│   │   ├── state/                  # State store
│   │   └── package.json
│   ├── explorer/                   # Web UI
│   │   ├── index.html
│   │   ├── app.js
│   │   ├── style.css
│   │   ├── server.js
│   │   └── package.json
│   ├── wallet/                     # Transaction builder
│   │   ├── tx_builder.js
│   │   └── package.json
│   ├── dao/                        # Governance
│   │   ├── contracts/
│   │   │   └── CHATAI.aln
│   │   ├── spec/
│   │   │   └── governogram.aln
│   │   └── package.json
│   ├── migration/                  # Cross-chain bridge
│   │   ├── spec/
│   │   │   └── migration_state.aln
│   │   └── package.json
│   ├── tests/                      # Test suite
│   │   ├── unit/
│   │   │   ├── aln_parser.test.js
│   │   │   └── qpu_math_hooks.test.js
│   │   └── package.json
│   ├── tools/                      # Dev tools
│   │   ├── aln_linter.js
│   │   └── package.json
│   └── docs/
│       └── developer_onboarding.aln
├── package.json                    # Root workspace config
├── README.aln.md                   # Main documentation
├── jest.config.json
├── .eslintrc.json
├── .prettierrc
├── .gitignore
└── LICENSE

```

## 🔑 Key Concepts

### Chainlexemes
Minimal ALN instruction units encoding blockchain state changes:
```aln
[header]
op_code: transfer
from: aln1abc...
to: aln1xyz...
nonce: 42

[data]
asset: ALN
amount: 1000000

[footer]
signature: ed25519:0x...
timestamp: 1732723200
```

### Migragraph
Time-ordered visualization of Canto→ALN asset migrations tracked in real-time.

### Governogram
Structured DAO proposal format for parameter changes, treasury spends, and contract upgrades.

### QPU.Math+
Safety verification system ensuring conservation laws and transaction limits.

## 🧪 Testing

```powershell
# Run all tests
npm test

# Run unit tests only
npm run test:unit --workspace=aln/tests

# Lint ALN files
node aln\tools\aln_linter.js aln\
```

## 🛠️ Building WASM Artifacts (Contracts)

Prerequisites:
- Rust toolchain (stable) with wasm target: `rustup target add wasm32-unknown-unknown`
- wasm-opt (binaryen): install via `sudo apt-get install -y binaryen` on Ubuntu, or `choco install binaryen` on Windows if using Chocolatey.

Build & optimize contracts locally:
```powershell
# build all contract crates for wasm
cd 'c:\Users\Hunter\Repos\ALN+Blockchain'
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown -p aln_auet
cargo build --release --target wasm32-unknown-unknown -p aln_csp
cargo build --release --target wasm32-unknown-unknown -p aln_registry
cargo build --release --target wasm32-unknown-unknown -p aln_bridge
cargo build --release --target wasm32-unknown-unknown -p energy_router

# optimize
wasm-opt -Oz -o artifacts/aln_auet.optimized.wasm target/wasm32-unknown-unknown/release/aln_auet.wasm
wasm-opt -Oz -o artifacts/aln_csp.optimized.wasm target/wasm32-unknown-unknown/release/aln_csp.wasm
wasm-opt -Oz -o artifacts/aln_registry.optimized.wasm target/wasm32-unknown-unknown/release/aln_registry.wasm
wasm-opt -Oz -o artifacts/aln_bridge.optimized.wasm target/wasm32-unknown-unknown/release/aln_bridge.wasm
wasm-opt -Oz -o artifacts/energy_router.optimized.wasm target/wasm32-unknown-unknown/release/energy_router.wasm

# compute provenance
cargo build -p did_provenance --release
./target/release/did_provenance prove-wasm artifacts/aln_auet.wasm aln_auet

```

## WASM Size & Gas Analysis

After building and optimizing WASM artifacts, run the analysis to measure size and ensure artifacts fit threshold:

```powershell
# default threshold 2 MiB
cargo build -p wasm_analysis --release
.
	arget\release\wasm_analysis artifacts 2097152
```

If any artifact exceeds the threshold, consider using additional code-splitting or off-chain verification to reduce on-chain verification costs. For gas estimation and more precise checks, run benchmarks (see `rust-benchmarks`) and consider deploying to a local test chain to measure gas consumption on execution.


The CI runs the above and stores artifacts in the `artifacts/` folder. The `artifacts/` folder contains the following files per contract:
- `artifacts/<crate>.wasm` - the unoptimized wasm file
- `artifacts/<crate>.optimized.wasm` - the wasm-opt optimized file
- `artifacts/<crate>.provenance.json` - DID + hash + git commit provenance

## CI Merge Gates
We require the following CI jobs to pass before merges to `main`:
- `ubs-policy`
- `did-admin-check`
- `rust-integration`
- `wasm-artifacts`
- `indexer-tests` (when indexer-related changes are present)

Review `CI.md` for instructions on configuring branch protection.


### Rust cw-multi-test Integration Tests (Rust/CosmWasm)

Run the cw-multi-test integration tests which validate cross-contract claim & spend flows.

```powershell
# Windows (PowerShell)
.
\scripts\test_all.ps1

# Unix-like (bash)
./scripts/test_all.sh
```

If you prefer running the Rust tests alone:

```powershell
cd tests/integration
cargo test --manifest-path Cargo.toml --verbose
```

## 📚 Documentation

- **API Reference**: See `README.aln.md`
- **Developer Guide**: `aln/docs/developer_onboarding.aln`
- **ALN Syntax**: `aln/core/spec/aln-syntax.aln`
- **Error Codes**: `aln/core/logging/errors.aln`
- **Governogram Format**: `aln/dao/spec/governogram.aln`

## 🛠️ Development Commands

```powershell
# Start node + explorer in dev mode
npm run dev

# Start only blockchain node
npm run start:node

# Start only explorer UI
npm run start:explorer

# Run linter
npm run lint

# Build all modules
npm run build
```

## 🔐 Security Features

✅ **Non-custodial wallet** - Keys never leave browser  
✅ **Conservation checks** - Token balance verification  
✅ **Limit enforcement** - Per-tx and per-account limits  
✅ **Governance safety** - DAO-controlled upgrades  
✅ **Structured logging** - Audit trail for all actions  

## 🌉 Migration Support

- Canto→ALN bridge with proof verification
- Migragraph tracking and visualization
- Reversibility checks and user consent
- Cryptographically verifiable state transitions

## 📊 Monitoring

- **Status Dashboard**: http://localhost:8080
- **API Endpoint**: http://localhost:3000/status
- **WebSocket Events**: ws://localhost:3001
- **Metrics**: http://localhost:3000/metrics

## 🤝 Contributing

1. Create feature branch: `git checkout -b feature/your-feature`
2. Make changes and test: `npm test`
3. Lint code: `npm run lint`
4. Commit: `git commit -m "feat: description"`
5. Push and create PR

## 📄 License

MIT License - See LICENSE file

## 🆘 Troubleshooting

### Node won't start
```powershell
# Reinitialize data directory
node cli\aln_node_cli.js init --force
```

### Explorer not connecting
- Verify node is running: `curl http://localhost:3000/status`
- Check CORS settings in `aln/core/api/http_server.js`

### Tests failing
```powershell
# Clean install dependencies
npm ci
```

## 🎯 Project Status

✅ Workspace scaffolded  
✅ Core runtime implemented  
✅ Explorer UI created  
✅ Wallet integration ready  
✅ DAO governance specs defined  
✅ Migration bridge structured  
✅ Testing framework configured  
✅ CI/CD pipeline defined  
✅ Documentation complete  

**Next**: Install Node.js and run `npm install` to begin development!

---

**aln_version**: 1.0  
**nanotopologene_profile**: surgical-grade  
**ai_firmware_version**: ALN.QPU.Math+  
**created**: 2024-11-27
