# Energy Market Semantics (AU.ET, CSP, ERP)

This document outlines standard actions and semantics for AU.ET, CSP, and ERP consumptions and their usage in ALN dApps.

Bindings:
- AU.ET: augmented-user energy tokens, fungible, used as daily energy budget reservoir.
- CSP: cybernetic strategy points, soulbound or restricted, used for permanent upgrades.
- ERP: energy-resource-points, optional single-use resource points used in active ability activation (implementation detail).

Key functions:
- increase_energy_cap: AU.ET balances increase a user's daily cap by alpha units per AU.ET.
- unlock_ability: Spend CSP to unlock a persistent ability; uses the CSP token and emits audit event.
- use_ability: Spend ERP or AU.ET to use a timed ability; decrements user's energy balance and emits events.

Router model:
- One `contracts/energy_router` module receives AU.ET/CSP via `Spend` messages and emits events describing the action.
- The router acts as a controlled escrow; only registered modules and protocol contracts can be designated as action destinations.

Event types:
- EnergyIncrease { address, amount, new_cap }
- AbilityUnlocked { address, ability_id, cost_csp }
- AbilityUsed { address, ability_id, cost_energy }

Integration:
- Add `contracts/energy_router` as the allowed spend recipient in the AU.ET and CSP whitelists for protocol flows.
- Document how apps call into `energy_router` to manage energy related flows.

Economics:
- Set base `E_base` and per-AU.ET amplification `alpha` in a governance controlled config.
- CSP cost curve uses geometric growth formula implemented in contract-side or off-chain checks.
