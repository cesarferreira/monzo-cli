# monzo-cli Agent Skill

Use this skill when the user asks anything about their Monzo account: balance, transactions, spending, pots, exports, or auth. Execute the `monzo` CLI directly — do not fabricate data.

---

## Prerequisites

The binary is `monzo`. All commands require a valid authenticated session. If a command fails with "No access token" or "No account ID", follow the auth and account setup steps below before retrying.

---

## Auth & Setup

### Check status first
```bash
monzo auth status
```
If valid and an Account ID is shown, proceed. If not, authenticate.

### OAuth login (recommended)
```bash
monzo auth login --client-id CLIENT_ID --client-secret CLIENT_SECRET
```
⚠️ After the browser opens, **the user must approve the login in the Monzo mobile app immediately**. The token is not issued until they tap Approve. The CLI waits — do not close it.

### Playground token (quick, no client needed)
```bash
monzo auth set-token TOKEN
```
Tokens from the [playground](https://developers.monzo.com/api/playground) expire in ~8 hours and don't auto-refresh.

### Set the default account

Most commands operate on one default account. If missing, list all accounts first:
```bash
monzo accounts
```
The **ID column** holds the correct value — always starts with `acc_`. The Description column (`user_…`) is **not** an account ID and will cause 403 errors.
```bash
monzo auth set-account acc_00009xxxxxxxx
```
This does **not** change the token; only the stored account preference is updated.

### Refresh an expired OAuth token
```bash
monzo auth refresh
```
OAuth tokens auto-refresh when expired if client credentials are stored. Playground tokens do not auto-refresh.

---

## Global flags

| Flag | Effect |
|------|--------|
| `--json` | Machine-readable JSON output. Use with `jq` for scripting. Works on every command. |

---

## Balance

```bash
monzo balance
monzo --json balance
```

Returns: current balance, total balance (including pots), spend today, and total including savings.

---

## Accounts

```bash
monzo accounts
monzo --json accounts
```

Lists all accounts (personal, joint, Flex, rewards). Use the **ID column** (`acc_…`) with `monzo auth set-account`.

---

## Transactions

### List
```bash
monzo transactions                       # last 30, default window
monzo tx                                 # alias
monzo transactions -n 50                 # show 50
monzo transactions --since 7d            # last 7 days
monzo transactions --from 2025-01-01 --to 2025-01-31   # calendar range
monzo transactions -c eating_out         # filter by category
monzo transactions -q "Tesco"            # search by merchant/description
monzo transactions --min 10 --max 100    # amount range in £
monzo --json transactions                # JSON output
```

**Date flags** (interchangeable names):
- `--since` / `--from` — lower bound: `YYYY-MM-DD` or relative (`7d`, `2w`, `3m`, `1y`)
- `--before` / `--to` — upper bound: `YYYY-MM-DD` is treated as **inclusive** (the CLI adds one day for the API call)

**Known categories**: `eating_out`, `groceries`, `transport`, `bills`, `entertainment`, `shopping`, `cash`, `transfers`, `personal_care`, `holidays`, `general`

### Single transaction detail
```bash
monzo transaction tx_00009xxxxxxxx
monzo --json transaction tx_00009xxxxxxxx
```

### Annotate a transaction
```bash
monzo annotate tx_00009xxxxxxxx note "team lunch"
monzo annotate tx_00009xxxxxxxx note ""    # deletes the key
```

---

## Search

Full-text across merchant name, description, notes, and category.

```bash
monzo search "Amazon"
monzo search "Amazon" -n 50
monzo search "coffee" --from 2025-01-01 --to 2025-01-31
monzo --json search "Netflix"
```

`--to` requires `--from` (or `--since`). Default window when only `--from` is set: 90 days from that date.

---

## Pots

```bash
monzo pots                                # list all pots
monzo pots deposit "Holiday Fund" 50      # deposit £50
monzo pots deposit "Holiday" 50           # partial name match works
monzo pots withdraw "Holiday Fund" 20     # withdraw £20
monzo --json pots
```

Pot names are matched case-insensitively and by substring. If the match is ambiguous, a list of candidates is shown — be more specific.

Pots with "added security" cannot be withdrawn from via the API.

---

## Spending Insights

All insight subcommands accept the same `--from`/`--to` (or `--since`/`--before`) flags. Flags may appear **before or after** the subcommand.

```bash
monzo insights                            # full report (default 30d)
monzo insights report
monzo insights categories
monzo insights merchants
monzo insights daily
monzo insights weekly
monzo insights predict
monzo insights recurring

# With time window
monzo insights categories --since 90d
monzo insights categories --from 2025-01-01 --to 2025-01-31
monzo insights --from 2025-01-01 categories    # flag before subcommand also works
```

| Subcommand | Output |
|------------|--------|
| `report` | Full breakdown: categories, merchants, daily totals |
| `categories` | Spending per category with % and bar chart |
| `merchants` | Top 15 merchants by total spend |
| `daily` | Spending per calendar day |
| `weekly` | Spending per week |
| `predict` | Projected month-end total vs historical average |
| `recurring` | Detected subscriptions and recurring payments |

---

## Export

```bash
monzo export -f csv -o spending.csv
monzo export -f json -o spending.json
monzo export -f csv -o jan-2025.csv --from 2025-01-01 --to 2025-01-31
monzo export -f csv -o last-year.csv --since 1y
```

`--to` requires `--from`. Default window: 90 days. Output path is required (`-o`).

CSV columns: `date`, `description`, `merchant`, `category`, `amount`, `currency`, `notes`.

---

## Webhooks

```bash
monzo webhooks                                          # list
monzo webhooks add https://example.com/hook             # register
monzo webhooks remove whk_00009xxxxxxxx                 # delete by ID
monzo --json webhooks
```

---

## Feed

Push a custom notification card to the Monzo app.

```bash
monzo feed "Title here"
monzo feed "Title here" --body "Optional body text"
```

---

## Config

Show where the config file lives (useful for manual editing or debugging):

```bash
monzo config
```

Config is a TOML file. You can set `account_id` directly if the CLI can't auto-detect it.

---

## Developer portal

```bash
monzo developers    # opens https://developers.monzo.com/apps/home
monzo apps          # alias
```

---

## Date / time reference

| Format | Meaning | Example |
|--------|---------|---------|
| `YYYY-MM-DD` | Start of that calendar day (UTC) | `2025-01-01` |
| `Nd` | N days ago | `7d`, `30d` |
| `Nw` | N weeks ago | `2w` |
| `Nm` | N × 30 days ago | `3m` |
| `Ny` | N × 365 days ago | `1y` |
| `--to YYYY-MM-DD` | Inclusive last day (API receives midnight of next day) | `--to 2025-01-31` |

---

## JSON + jq recipes

```bash
# Current balance in pounds
monzo --json balance | jq '.balance / 100'

# Total including savings
monzo --json balance | jq '(.balance + .total_balance) / 100'

# List pot names and balances
monzo --json pots | jq '.[] | {name, balance: (.balance / 100)}'

# Transactions over £50 last month
monzo --json transactions --since 30d | jq '[.[] | select(.amount < -5000)]'

# Total spent on eating_out this month
monzo --json transactions --since 30d -c eating_out \
  | jq '[.[] | .amount] | add | . / 100 * -1'

# Export January 2025 as JSON, then query with jq
monzo --json transactions --from 2025-01-01 --to 2025-01-31 \
  | jq '[.[] | select(.amount < 0)]'
```

---

## Error reference

| Error | Cause | Fix |
|-------|-------|-----|
| `No access token configured` | Not authenticated | `monzo auth login` or `monzo auth set-token TOKEN` |
| `No account ID configured` | Account not set | `monzo accounts` then `monzo auth set-account acc_…` |
| `API error: 403 Forbidden` | Wrong account ID (e.g. `user_…` instead of `acc_…`) | Re-run `monzo accounts`, copy the **ID** column, `monzo auth set-account acc_…` |
| `Token is invalid or expired` | Token expired and can't refresh | `monzo auth refresh` or re-login |
| `Multiple pots match "..."` | Ambiguous pot name | Use a more specific substring or the full name |
| `unexpected argument '--since'` (insights) | Clap version mismatch | Rebuild with `cargo build --release` |
