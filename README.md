# monzo-cli

[![CI](https://github.com/cesarferreira/monzo-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/cesarferreira/monzo-cli/actions/workflows/ci.yml)

> A modern CLI for Monzo Bank, rewritten in Rust

## Usage

### Balance

```bash
$ monzo balance

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
$ monzo transactions --since 7d

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
$ monzo pots

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
$ monzo insights categories --since 30d

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
$ monzo search "coffee" --since 90d
$ monzo search "Amazon" -n 50
```

### Export

```bash
$ monzo export -f csv -o spending.csv --since 90d
$ monzo export -f json -o data.json --since 1y
```

### JSON Mode

Every command supports `--json` for structured output, useful for scripting or piping into `jq`:

```bash
$ monzo --json balance | jq '.balance / 100'
$ monzo --json tx --since 30d -c eating_out
$ monzo --json pots
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
| `developers` | Open [Monzo developer portal](https://developers.monzo.com/apps/home) (OAuth apps) in browser |
| `apps` | Alias for `developers` |
| `auth login` | OAuth2 login (opens browser) |
| `auth refresh` | Refresh expired token |
| `auth set-token <token>` | Quick setup with playground token |
| `auth set-account <id>` | Set default account (`acc_…` from `monzo accounts` **ID** column) |
| `auth status` | Check if token is valid |

## Install

```bash
cargo install monzo-cli
```

Or via [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall monzo-cli
```

Or build from source:

```bash
cargo build --release
# Binary at target/release/monzo
```

## Setup

### Option 1: OAuth2 (recommended)

1. Create an OAuth client in the [Monzo developer apps console](https://developers.monzo.com/apps/home) (or run `monzo developers` to open it in your browser)
2. Set the redirect URL to `http://localhost:6789/callback`
3. Run:

```bash
monzo auth login --client-id YOUR_CLIENT_ID --client-secret YOUR_CLIENT_SECRET
```

### Option 2: Playground token (quick start)

Get a token from the [Monzo API Playground](https://developers.monzo.com/api/playground):

```bash
monzo auth set-token YOUR_ACCESS_TOKEN
```

Optional: set the default account in the same step (IDs from `monzo accounts`, see below):

```bash
monzo auth set-token YOUR_ACCESS_TOKEN --account-id acc_00009xxxxxxxx
```

### Default account

Monzo’s API needs an **account ID** on top of your access token: you can have several accounts (personal, joint, Flex, rewards, etc.), and commands like `balance`, `transactions`, and `pots` all use one **default** account stored in config.

After **`monzo auth login`**, the CLI tries to pick the first `uk_retail` account automatically. If that did not run (for example, you still had to approve login in the Monzo app), you may be logged in but see *“No account ID configured”* or similar when running other commands. Fix it without re-authenticating:

```bash
monzo accounts
monzo auth set-account acc_00009xxxxxxxx
```

**Use the value in the `ID` column** — it always starts with **`acc_`**. The **Description** column often shows a **`user_…`** ID or joint-account text; that is **not** an account ID. Passing a `user_…` value will be rejected by the CLI (and would cause API errors such as 403 if it were saved manually).

You can also set `account_id` in your config file (`monzo config` shows the path).

## Caveats

- Access tokens expire after ~6 hours. If you logged in via OAuth2, **tokens are refreshed automatically** when they expire - no manual action needed
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
