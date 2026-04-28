# damm-v2-fee-routing

Permissionless fee routing for [Meteora DLMM V2](https://github.com/MeteoraAg/damm-v2). Create zero-liquidity "honorary" positions that accrue fees in the quote token, then route those fees between investors and creators based on a configurable split.

Built for [Superteam Earn](https://superteam.fun/earn/listing/build-permissionless-fee-routing-anchor-program-for-meteora-dlmm-v2).

## How it works

1. **Initialize Honorary Position** — Creates an empty DAMM V2 position owned by a program-derived PDA. The position has zero liquidity but still accrues swap fees from the pool in the quote token.
2. **Claim Fees** — Calls Meteora's `claim_position_fee` via CPI. Collects accrued fees from the position into treasury accounts.
3. **Route Fees** — Splits claimed fees between the creator and a vault based on `investor_fee_share_bps`. Remaining fees go to the creator.
4. **Close Position** — Closes the honorary position and recovers rent.

## Program ID

```
HuuZH7f52k6mrjdqyV3vaK6xoY2FWwirdbqBeJu8qfFc
```

## Instructions

### initialize_honorary_position

Creates a vault config PDA and an honorary position in a Meteora DLMM V2 pool.

| Param | Type | Description |
|-------|------|-------------|
| `y0_total_allocation` | `u64` | Total investor allocation at TGE |
| `investor_fee_share_bps` | `u16` | Investor fee share in basis points (0-10000) |

**Validation:**
- Pool must support quote-only fee collection (`collect_fee_mode` 0 or 1)
- Base mint < Quote mint (token order matches pool)
- `investor_fee_share_bps` ≤ 10000

### claim_fee

CPIs to Meteora's `claim_position_fee` to collect accrued fees.

Requires treasury token accounts derived from the Meteora treasury program. These are provided by the caller.

### route_fees

Splits tokens in the position authority's quote token account between the vault (investor share) and the creator.

### close_position

CPIs to Meteora's `close_position` to close the honorary position. Closes the vault config PDA and returns rent to the payer.

## PDA Derivation

| PDA | Seeds | Description |
|-----|-------|-------------|
| `vault_config` | `["vault", vault_pubkey]` | Stores pool, position, and fee split config |
| `position_authority` | `["vault", vault_pubkey, "position_authority"]` | Signs CPIs to Meteora program |
| `position` | `["position", position_nft_mint]` (from cp-amm) | The honorary position account |

## Development

### Prerequisites

- Solana CLI 1.18+
- Anchor 0.31.0
- Node.js 18+
- Yarn

### Build

```bash
cargo build
cargo build-sbf
```

### Test

```bash
# Unit/localt tests
anchor test

# Against devnet (requires SOL)
anchor test --provider.cluster devnet
```

### Deploy

```bash
anchor deploy --provider.cluster devnet
```

## Architecture

```
User / Client
    │
    ├─ initialize_honorary_position ─► CPI: cp_amm::create_position
    │                                  └─ Creates VaultConfig PDA
    │
    ├─ claim_fee ─► CPI: cp_amm::claim_position_fee
    │              └─ Emits FeesClaimed event
    │
    ├─ route_fees ─► Token transfers via position_authority PDA
    │               └─ Emits FeesRouted event
    │
    └─ close_position ─► CPI: cp_amm::close_position
                        └─ Closes VaultConfig PDA
```

## Surfpool Runbooks

This project includes [Surfpool](https://surfpool.run) runbooks for reproducible deployments:

```bash
# Start a surfnet (local fork)
surfpool start

# Deploy program
surfpool run deployment

# Initialize honorary position
surfpool run init \
  --pool-address <POOL> \
  --base-mint <BASE> \
  --quote-mint <QUOTE> \
  --y0-total-allocation <AMOUNT> \
  --investor-fee-share-bps <BPS>
```

## License

ISC
