#!/usr/bin/env bash
set -euo pipefail

# Example deploy commands for a local ALN chain using 'alnd' binary
# Requires env: RPC/ALN_CHAIN_ID/KEY_NAME

ALN_RPC=${ALN_RPC:-"http://localhost:26657"}
ALN_CHAIN_ID=${ALN_CHAIN_ID:-"aln-local-1"}
KEY_NAME=${KEY_NAME:-"validator"}

# 1) Store AU.ET
# Verify DID admin matches operator
cargo run -p did_provenance -- did-admin-check || { echo "DID admin check failed"; exit 1; }
# CODE_ID_AUET=$(alnd tx wasm store artifacts/auet.wasm --from $KEY_NAME --chain-id $ALN_CHAIN_ID --node $ALN_RPC -y --output json | jq -r ".code_id")

# 2) Instantiate AU.ET (example)
# alnd tx wasm instantiate $CODE_ID_AUET "$(cat artifacts/aln_init_auet.json)" --label "aln-auet" --from $KEY_NAME --chain-id $ALN_CHAIN_ID --node $ALN_RPC --no-admin -y

# 3) Repeat for CSP
# CODE_ID_CSP=$(alnd tx wasm store artifacts/csp.wasm --from $KEY_NAME --chain-id $ALN_CHAIN_ID --node $ALN_RPC -y --output json | jq -r ".code_id")
# alnd tx wasm instantiate $CODE_ID_CSP "$(cat artifacts/aln_init_csp.json)" --label "aln-csp" --from $KEY_NAME --chain-id $ALN_CHAIN_ID --node $ALN_RPC --no-admin -y

# NOTE: Replace with actual CLI of ALN (alnd / kujarid) as appropriate.

echo "Deploy script (template) executed. Customize and uncomment store/instantiate lines before use."
