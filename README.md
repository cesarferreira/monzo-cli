# monzo-cli

[![CI](https://github.com/cesarferreira/monzo-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/cesarferreira/monzo-cli/actions/workflows/ci.yml)

> A modern CLI for Monzo Bank, rewritten in Rust

## Usage

### Balance

```bash
$ monzo-cli balance

  ┌────────────────────┬───────────┐
  │                    │ Amount    │
  ├────────────────────┼───────────┤
  │ Balance            │   £490.00 │
  │ Total Balance      │ £1,250.00 │
  │ Spent Today        │   -£10.50 │
  │ Including Savings  │ £3,500.00 │
  └────────────────────┴───────────┘
```

### Transactions

```bash
$ monzo-cli transactions --since 7d

  ┌──────────────────┬────────────────────┬────────────┬──────────┬───────────┐
  │ Date             │ Description        │ Category   │   Amount │   Balance │
  ├──────────────────┼────────────────────┼────────────┼──────────┼───────────┤
  │ 2026-03-23 09:15 │ Pret A Manger      │ eating_out │   -£3.50 │   £490.00 │
  │ 2026-03-22 18:30 │ Tesco              │ groceries  │  -£45.20 │   £493.50 │
  │ 2026-03-22 08:00 │ TfL                │ transport  │   -£2.80 │   £538.70 │
  │ 2026-03-21 20:15 │ Netflix            │ bills      │   -£9.99 │   £541.50 │
  └──────────────────┴────────────────────┴────────────┴──────────┴───────────┘
  4 transactions
```

### Pots

```bash
$ monzo-cli pots

  ┌──────────────────┬───────────┬─────────────────┬──────────┬──────────┐
  │ Name             │   Balance │            Goal │ Type     │ Round-up │
  ├──────────────────┼───────────┼─────────────────┼──────────┼──────────┤
  │ Holiday Fund     │   £500.00 │ £1,000.00 (50%) │ -        │ -        │
  │ Emergency        │ £1,200.00 │               - │ -        │ -        │
  │ Coin Jar         │    £47.30 │               - │ -        │ Yes      │
  └──────────────────┴───────────┴─────────────────┴──────────┴──────────┘
  3 pots | Total: £1,747.30
```

### Spending Insights

```bash
$ monzo-cli insights categories --since 30d

  Spending by Category
  ┌──────────────────────┬──────────┬────────────┬─────────────────┐
  │ Category             │    Spent │ % of Total │ Bar             │
  ├──────────────────────┼──────────┼────────────┼─────────────────┤
  │ groceries            │ -£245.00 │        35% │ ████████████████│
  │ eating_out           │ -£180.50 │        26% │ █████████████   │
  │ transport            │  -£95.00 │        14% │ ███████         │
  │ bills                │  -£85.99 │        12% │ ██████          │
  │ entertainment        │  -£45.00 │         6% │ ███             │
  └──────────────────────┴──────────┴────────────┴─────────────────┘
  Total: -£651.49
```

### Search

```bash
$ monzo-cli search "coffee" --since 90d
$ monzo-cli search "Amazon" -n 50
```

### Export

```bash
$ monzo-cli export -f csv -o spending.csv --since 90d
$ monzo-cli export -f json -o data.json --since 1y
```

### JSON Mode

Every command supports `--json` for structured output, useful for scripting or piping into `jq`:

```bash
$ monzo-cli --json balance | jq '.balance / 100'
$ monzo-cli --json tx --since 30d -c eating_out
$ monzo-cli --json pots
```

## All Commands

| Command | Description |
|---|---|
| `balance` | Show account balance, spend today, savings |
| `transactions` | List transactions with filters (category, amount, date, search) |
| `tx` | Alias for `transactions` |
| `transaction <id>` | Show full transaction detail |
| `annotate <id> <key> <value>` | Add metadata to a transaction |
| `pots` | List pots with balances and goals |
| `pots deposit <name> <amount>` | Deposit into a pot |
| `pots withdraw <name> <amount>` | Withdraw from a pot |
| `search <query>` | Full-text search across transactions |
| `insights` | Full spending report |
| `insights categories` | Spending breakdown by category |
| `insights merchants` | Top merchants by total spend |
| `insights daily` | Daily spending chart |
| `insights weekly` | Weekly spending totals |
| `insights predict` | Monthly projection with historical comparison |
| `insights recurring` | Detect subscriptions and recurring payments |
| `export` | Export transactions to CSV or JSON |
| `webhooks` | List, add, or remove webhooks |
| `feed <title>` | Push a notification to your Monzo app |
| `accounts` | List all accounts |
| `auth login` | OAuth2 login (opens browser) |
| `auth refresh` | Refresh expired token |
| `auth set-token <token>` | Quick setup with playground token |
| `auth status` | Check if token is valid |

## Install

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
# Binary at target/release/monzo-cli
```

## Setup

### Option 1: OAuth2 (recommended)

1. Create an OAuth client at [developers.monzo.com](https://developers.monzo.com)
2. Set the redirect URL to `http://localhost:6789/callback`
3. Run:

```bash
monzo-cli auth login --client-id YOUR_CLIENT_ID --client-secret YOUR_CLIENT_SECRET
```

### Option 2: Playground token (quick start)

Get a token from the [Monzo API Playground](https://developers.monzo.com/api/playground):

```bash
monzo-cli auth set-token YOUR_ACCESS_TOKEN
```

## Caveats

- Access tokens expire after ~6 hours. Use `monzo-cli auth refresh` to renew (confidential clients only)
- After initial auth, you have a 5-minute window to fetch full transaction history (Monzo SCA restriction)
- Pots with "added security" cannot be withdrawn from via the API

## Contributing

I welcome and encourage all pull requests. Here are some basic rules to follow:
  1. If its a feature, bugfix, or anything please only change code to what you specify.
  2. Please keep PR titles easy to read and descriptive of changes.
  3. Pull requests _must_ be made against `main` branch.
  4. Check for existing [issues](https://github.com/cesarferreira/monzo-cli/issues) first, before filing an issue.
  5. Have fun!

### Created & Maintained By
[Cesar Ferreira](https://github.com/cesarferreira) ([@cesarmcferreira](https://www.twitter.com/cesarmcferreira))
