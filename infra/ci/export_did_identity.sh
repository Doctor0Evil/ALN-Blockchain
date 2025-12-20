#!/usr/bin/env bash
# Export ALN_PRIMARY_DID from infra/identity/did-config.json if available.
set -euo pipefail
CONFIG_FILE="infra/identity/did-config.json"
ALN_DID=""

if ! command -v jq >/dev/null 2>&1; then
  echo "WARNING: 'jq' not found. Skipping DID export."
  exit 0
fi

if [ ! -f "$CONFIG_FILE" ]; then
  echo "WARNING: DID config $CONFIG_FILE not found. Skipping DID export."
  exit 0
fi

ALN_DID=$(jq -r '.primary_did // empty' "$CONFIG_FILE" 2>/dev/null || true)
if [ -z "$ALN_DID" ]; then
  echo "WARNING: primary_did not found in $CONFIG_FILE. Skipping DID export."
  exit 0
fi

export ALN_PRIMARY_DID="$ALN_DID"
# Print a non-sensitive log line
echo "ALN_PRIMARY_DID=$ALN_PRIMARY_DID"
