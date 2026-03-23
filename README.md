# monzo-cli (Rust)

A modern, fast CLI for [Monzo Bank](https://monzo.com) — query balances, transactions, pots, get spending insights, and export data. Built in Rust for speed and reliability.

## Features

- **Accounts** — list all your Monzo accounts
- **Balance** — current balance, total balance, spend today, savings
- **Transactions** — list, filter by category/amount/date, search by merchant
- **Pots** — list pots with goals/progress, deposit, withdraw
- **Search** — full-text search across transactions
- **Insights** — spending analytics, category breakdown, top merchants, daily/weekly charts, recurring payment detection, monthly spend prediction with historical comparison
- **Export** — CSV or JSON export for external analysis
- **Webhooks** — register, list, and delete webhooks
- **Feed Items** — push custom notifications to your Monzo app
- **JSON mode** — `--json` flag on every command for agent/script consumption
- **OAuth2** — full login flow with browser redirect, token refresh support

## Install

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
# Binary at target/release/monzo-cli
```

## Authentication

### Option 1: OAuth2 Flow (recommended for confidential clients)

1. Create an OAuth client at [developers.monzo.com](https://developers.monzo.com)
2. Set the redirect URL to `http://localhost:6789/callback`
3. Run:

```bash
monzo-cli auth login --client-id YOUR_CLIENT_ID --client-secret YOUR_CLIENT_SECRET
```

This opens your browser, handles the callback, exchanges the code for tokens, and auto-detects your account. You'll need to approve the login in the Monzo app.

### Option 2: Developer Playground Token (quick setup)

1. Get a token from the [Monzo API Playground](https://developers.monzo.com/api/playground)
2. Run:

```bash
monzo-cli auth set-token YOUR_ACCESS_TOKEN
```

Note: playground tokens expire after ~6 hours.

### Token Refresh

```bash
monzo-cli auth refresh   # Refresh expired token (confidential clients only)
monzo-cli auth status    # Check if your token is valid
```

Environment variables `MONZO_CLIENT_ID` and `MONZO_CLIENT_SECRET` are also supported.

## Usage

```bash
# Basics
monzo-cli balance
monzo-cli accounts
monzo-cli config                    # Show config file location

# Transactions
monzo-cli transactions              # Last 30 transactions
monzo-cli tx -n 50                  # Last 50 (alias: tx)
monzo-cli tx --since 7d             # Last 7 days
monzo-cli tx --since 2024-01-01     # Since a specific date
monzo-cli tx -c eating_out          # Filter by category
monzo-cli tx -q "Tesco"             # Search by merchant
monzo-cli tx --min 50 --max 200     # Amount range (in pounds)
monzo-cli tx --since 30d -c groceries --json  # Combine filters + JSON

# Transaction detail
monzo-cli transaction tx_00009abc...
monzo-cli annotate tx_00009abc... notes "Birthday dinner"

# Search
monzo-cli search "coffee" --since 90d
monzo-cli search "Amazon" -n 50

# Pots
monzo-cli pots                      # List all pots with balances & goals
monzo-cli pots deposit "Holiday" 50.00
monzo-cli pots withdraw "Emergency" 100.00

# Insights & Analytics
monzo-cli insights                  # Full report
monzo-cli insights categories       # Spending by category with bar chart
monzo-cli insights merchants        # Top merchants by total spend
monzo-cli insights daily            # Daily spending chart
monzo-cli insights weekly           # Weekly spending totals
monzo-cli insights predict          # Monthly projection + historical comparison
monzo-cli insights recurring        # Detect subscriptions & recurring payments
monzo-cli insights --since 90d      # Insights over custom period

# Export
monzo-cli export -f csv -o spending.csv --since 90d
monzo-cli export -f json -o data.json --since 1y

# Webhooks
monzo-cli webhooks                  # List webhooks
monzo-cli webhooks add https://example.com/hook
monzo-cli webhooks remove whk_...

# Feed items (push to Monzo app)
monzo-cli feed "Hello from CLI" --body "This is a test notification"

# JSON output (for agents/scripts)
monzo-cli --json balance
monzo-cli --json pots
monzo-cli --json tx --since 30d
```

## Agent / Script Integration

Every command supports `--json` for structured output, making it easy to pipe into `jq`, feed to an AI agent, or integrate with automation:

```bash
# Get balance as JSON
monzo-cli --json balance | jq '.balance / 100'

# Export last month's transactions for analysis
monzo-cli --json tx --since 30d > /tmp/transactions.json

# Search and process results
monzo-cli --json search "Uber" | jq '[.[] | .amount] | add / 100'
```

## Categories

Monzo uses these transaction categories:
`general`, `eating_out`, `groceries`, `transport`, `cash`, `bills`, `entertainment`, `shopping`, `holidays`, `expenses`

## Config

Config is stored at:
- **macOS**: `~/Library/Application Support/com.cesarferreira.monzo-cli/config.toml`
- **Linux**: `~/.config/monzo-cli/config.toml`

## API Notes

- Access tokens expire after ~6 hours
- After initial auth, you have a 5-minute window to fetch full transaction history (SCA restriction)
- The API defaults to 30 transactions per request (max 100)
- Pots with "added security" cannot be withdrawn from via the API
- Rate limiting returns HTTP 429 — the CLI will tell you to retry

## License

MIT
