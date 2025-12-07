# ALN CSP (ALN20)

Cybernetic Strategy Points (CSP) ALN-20 token with strong transfer restrictions.

- Based on CW20 but transfer restricted (soulbound by default).
- No public mint after instantiate (constructor-only allocation).
- Use the `whitelist` approach if a limited set of protocol addresses must transfer CSP for protocol flows.
- Metadata identical to AU.ET (source provenance and snapshot root).

Tests should enforce: CSP transfers by normal users fail; protocol-approved transfers succeed when sender/recipient match policy.
