# Unreal Engine Integration for ALN Augmented Policies

## Overview
Provides Blueprint/C++ nodes for UE clients to check augmented user policies via RPC calls to ALN chain.

## Nodes (Blueprint-accessible)

### GetAugmentedUserPolicy
- **Input**: DID or wallet address
- **Output**: `AugmentedUserPolicy` struct
- **RPC**: Calls `policy_engine.getEffectivePolicy(user)`

### GetAugmentedEnergyState
- **Input**: DID or wallet address
- **Output**: `AugmentedEnergyState` struct
- **RPC**: Queries on-chain energy state

### CheckAugmentationActionAllowed
- **Input**: ActionId (string), User (DID/wallet), EnergyState (optional)
- **Output**: Allowed (bool), Reason (string)
- **RPC**: Calls `policy_engine.isActionAllowed(user, action_id, energy_state)`

## Usage Example (Blueprint)
```
1. On augmentation activation attempt:
   - Call CheckAugmentationActionAllowed(ActionId="ENHANCED_VISION", User=PlayerDID)
   - If Allowed=false, display Reason to player and block activation
   - If Allowed=true, proceed with augmentation

2. Periodically (every 5 minutes):
   - Call GetAugmentedEnergyState(User=PlayerDID)
   - Update UI with current energy levels
   - Warn if approaching thresholds
```

## RPC Configuration (Planq-style)
- **Chain ID**: 7070 (or ALN mainnet when live)
- **RPC URL**: http://localhost:3000 (dev) or configured endpoint
- **Symbol**: ALN

## Caching
- Policy and energy state results cached for 30 seconds
- Action checks are real-time (no cache)

## Audit Transparency
- UE client logs all allow/deny outcomes locally
- Player can view audit history in UI (Settings > Augmentation Audit Log)
