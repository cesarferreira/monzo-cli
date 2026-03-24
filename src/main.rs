mod analytics;
mod auth;
mod client;
mod config;
mod display;
mod models;

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;

use client::MonzoClient;
use config::Config;
use models::format_money;

const DEVELOPER_PORTAL_APPS_URL: &str = "https://developers.monzo.com/apps/home";

/// Load config and auto-refresh the token if expired (and refresh credentials are available).
/// Returns an up-to-date (Config, MonzoClient) pair ready to use.
async fn authenticated() -> Result<(Config, MonzoClient)> {
    let mut config = Config::load()?;

    if config.is_token_expired() {
        let can_refresh = config.client_id.as_deref().is_some_and(|s| !s.is_empty())
            && config.client_secret.as_deref().is_some_and(|s| !s.is_empty())
            && config.refresh_token.as_deref().is_some_and(|s| !s.is_empty());

        if can_refresh {
            eprintln!("{}", "Token expired, refreshing automatically...".dimmed());
            let token = MonzoClient::refresh_token(
                config.client_id.as_deref().unwrap(),
                config.client_secret.as_deref().unwrap(),
                config.refresh_token.as_deref().unwrap(),
            )
            .await?;

            config.access_token = Some(token.access_token);
            config.refresh_token = token.refresh_token;
            if let Some(expires_in) = token.expires_in {
                config.token_expires_at = Some(Utc::now().timestamp() + expires_in);
            }
            config.save()?;
            eprintln!("{}", "Token refreshed.".green().dimmed());
        }
    }

    let client = MonzoClient::new(&config)?;
    Ok((config, client))
}

#[derive(Parser)]
#[command(name = "monzo", version, about = "Modern CLI for Monzo Bank")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output as JSON (for agent/script consumption)
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage authentication
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// List your accounts
    Accounts,

    /// Show account balance
    Balance,

    /// List and search transactions
    #[command(alias = "tx")]
    Transactions {
        /// Number of transactions to show
        #[arg(short = 'n', long, default_value = "30")]
        limit: u32,

        /// Lower bound: YYYY-MM-DD or duration (e.g. 7d, 1m). Alias: --from
        #[arg(short, long, visible_alias = "from")]
        since: Option<String>,

        /// Upper bound: inclusive calendar day YYYY-MM-DD (converted for the API). Alias: --to
        #[arg(short, long, visible_alias = "to")]
        before: Option<String>,

        /// Filter by category (eating_out, groceries, transport, etc.)
        #[arg(short, long)]
        category: Option<String>,

        /// Search by merchant name or description
        #[arg(short = 'q', long)]
        search: Option<String>,

        /// Minimum amount in pounds (e.g. 10.00)
        #[arg(long)]
        min: Option<f64>,

        /// Maximum amount in pounds (e.g. 50.00)
        #[arg(long)]
        max: Option<f64>,
    },

    /// Show transaction detail
    Transaction {
        /// Transaction ID
        id: String,
    },

    /// Annotate a transaction with metadata
    Annotate {
        /// Transaction ID
        id: String,
        /// Metadata key
        key: String,
        /// Metadata value (empty to delete)
        value: String,
    },

    /// Manage money pots
    Pots {
        #[command(subcommand)]
        action: Option<PotAction>,
    },

    /// Search transactions
    Search {
        /// Search query (matches merchant name, description, notes)
        query: String,

        /// Number of results
        #[arg(short = 'n', long, default_value = "20")]
        limit: u32,

        /// Lower bound: YYYY-MM-DD or duration (default 90d). Alias: --from
        #[arg(short, long, visible_alias = "from")]
        since: Option<String>,

        /// Upper bound: inclusive YYYY-MM-DD. Alias: --to
        #[arg(long, visible_alias = "to")]
        before: Option<String>,
    },

    /// Spending insights and analytics
    #[command(args_conflicts_with_subcommands = false)]
    Insights {
        #[command(subcommand)]
        action: Option<InsightAction>,

        /// Lower bound: YYYY-MM-DD or duration (default 30d). Alias: --from
        #[arg(short, long, global = true, visible_alias = "from")]
        since: Option<String>,

        /// Upper bound: inclusive YYYY-MM-DD. Alias: --to
        #[arg(long, global = true, visible_alias = "to")]
        before: Option<String>,
    },

    /// Manage webhooks
    Webhooks {
        #[command(subcommand)]
        action: Option<WebhookAction>,
    },

    /// Send a feed item to your Monzo app
    Feed {
        /// Title of the feed item
        title: String,
        /// Body text
        #[arg(short, long, default_value = "")]
        body: String,
    },

    /// Export transactions to CSV or JSON
    Export {
        /// Output format
        #[arg(short, long, default_value = "csv")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Lower bound: YYYY-MM-DD or duration (default 90d). Alias: --from
        #[arg(short, long, visible_alias = "from")]
        since: Option<String>,

        /// Upper bound: inclusive YYYY-MM-DD. Alias: --to
        #[arg(long, visible_alias = "to")]
        before: Option<String>,
    },

    /// Show where your config is stored
    Config,

    /// Open the Monzo developer portal (OAuth apps) in your browser
    #[command(visible_alias = "apps")]
    Developers,
}

#[derive(Subcommand)]
enum AuthAction {
    /// Login via OAuth2 (opens browser)
    Login {
        /// OAuth client ID (from the developer portal)
        #[arg(long, env = "MONZO_CLIENT_ID")]
        client_id: String,
        /// OAuth client secret
        #[arg(long, env = "MONZO_CLIENT_SECRET")]
        client_secret: String,
    },
    /// Refresh an expired access token
    Refresh,
    /// Manually set an access token (from Monzo developer playground)
    SetToken {
        /// Access token
        token: String,
        /// Account ID (optional, will auto-detect if omitted)
        #[arg(long)]
        account_id: Option<String>,
    },
    /// Set default account (use IDs from `monzo accounts`; does not change your token)
    SetAccount {
        /// Account ID (e.g. acc_00009…)
        account_id: String,
    },
    /// Check token status
    Status,
}

#[derive(Subcommand)]
enum PotAction {
    /// List all pots
    List,
    /// Deposit money into a pot
    Deposit {
        /// Pot name or ID
        pot: String,
        /// Amount in pounds (e.g. 10.50)
        amount: f64,
    },
    /// Withdraw money from a pot
    Withdraw {
        /// Pot name or ID
        pot: String,
        /// Amount in pounds (e.g. 10.50)
        amount: f64,
    },
}

#[derive(Subcommand)]
enum InsightAction {
    /// Full spending report
    Report,
    /// Spending by category
    Categories,
    /// Top merchants by spend
    Merchants,
    /// Daily spending chart
    Daily,
    /// Weekly spending chart
    Weekly,
    /// Monthly prediction based on current spend rate
    Predict,
    /// Detect recurring payments / subscriptions
    Recurring,
}

#[derive(Subcommand)]
enum WebhookAction {
    /// List registered webhooks
    List,
    /// Register a new webhook
    Add {
        /// Webhook URL
        url: String,
    },
    /// Delete a webhook
    Remove {
        /// Webhook ID
        id: String,
    },
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{}: {err:#}", "error".red().bold());
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { action } => handle_auth(action).await,
        Commands::Config => handle_config(),
        Commands::Accounts => handle_accounts(cli.json).await,
        Commands::Balance => handle_balance(cli.json).await,
        Commands::Transactions {
            limit,
            since,
            before,
            category,
            search,
            min,
            max,
        } => {
            handle_transactions(cli.json, limit, since, before, category, search, min, max)
                .await
        }
        Commands::Transaction { id } => handle_transaction_detail(cli.json, &id).await,
        Commands::Annotate { id, key, value } => handle_annotate(&id, &key, &value).await,
        Commands::Pots { action } => handle_pots(cli.json, action).await,
        Commands::Search {
            query,
            limit,
            since,
            before,
        } => handle_search(cli.json, &query, limit, since.as_deref(), before.as_deref()).await,
        Commands::Insights {
            action,
            since,
            before,
        } => handle_insights(action, since.as_deref(), before.as_deref()).await,
        Commands::Webhooks { action } => handle_webhooks(cli.json, action).await,
        Commands::Feed { title, body } => handle_feed(&title, &body).await,
        Commands::Export {
            format,
            output,
            since,
            before,
        } => handle_export(&format, &output, since.as_deref(), before.as_deref()).await,
        Commands::Developers => handle_developers(),
    }
}

// ── Auth ────────────────────────────────────────────────────────────────────

async fn handle_auth(action: AuthAction) -> Result<()> {
    match action {
        AuthAction::Login {
            client_id,
            client_secret,
        } => auth::login_flow(&client_id, &client_secret).await,
        AuthAction::Refresh => auth::refresh_flow().await,
        AuthAction::SetToken { token, account_id } => {
            auth::set_token(&token, account_id.as_deref())
        }
        AuthAction::SetAccount { account_id } => auth::set_default_account(&account_id),
        AuthAction::Status => {
            let config = Config::load()?;
            let client = MonzoClient::new(&config)?;
            match client.whoami().await {
                Ok(info) => {
                    println!("{}", "Token is valid".green().bold());
                    println!("  User ID:    {}", info.user_id);
                    println!("  Client ID:  {}", info.client_id);
                    if let Some(acc) = &config.account_id {
                        println!("  Account ID: {acc}");
                    }
                    if config.is_token_expired() {
                        println!(
                            "  {}",
                            "Warning: token may be expired based on local timestamp"
                                .yellow()
                        );
                    }
                }
                Err(e) => {
                    println!("{}: {e}", "Token is invalid or expired".red().bold());
                }
            }
            Ok(())
        }
    }
}

// ── Config ──────────────────────────────────────────────────────────────────

fn handle_config() -> Result<()> {
    let dir = Config::config_dir()?;
    println!("Config directory: {}", dir.display());
    println!("Config file:      {}", dir.join("config.toml").display());
    Ok(())
}

fn handle_developers() -> Result<()> {
    println!("Opening {} …", DEVELOPER_PORTAL_APPS_URL);
    open::that(DEVELOPER_PORTAL_APPS_URL)
        .with_context(|| format!("failed to open {}", DEVELOPER_PORTAL_APPS_URL))?;
    Ok(())
}

// ── Accounts ────────────────────────────────────────────────────────────────

async fn handle_accounts(json: bool) -> Result<()> {
    let (config, client) = authenticated().await?;
    let accounts = client.accounts(None).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&accounts)?);
    } else {
        display::print_accounts(&accounts);
        if config.account_id.is_none() && !accounts.is_empty() {
            println!(
                "\n{}",
                "Tip: set your default account with `monzo auth set-account <id>` (use the ID column, acc_…)"
                    .dimmed()
            );
        }
    }
    Ok(())
}

// ── Balance ─────────────────────────────────────────────────────────────────

async fn handle_balance(json: bool) -> Result<()> {
    let (config, client) = authenticated().await?;
    let account_id = config.account_id()?;
    let balance = client.balance(account_id).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&balance)?);
    } else {
        display::print_balance(&balance);
    }
    Ok(())
}

// ── Transactions ────────────────────────────────────────────────────────────

async fn handle_transactions(
    json: bool,
    limit: u32,
    since: Option<String>,
    before: Option<String>,
    category: Option<String>,
    search: Option<String>,
    min: Option<f64>,
    max: Option<f64>,
) -> Result<()> {
    let (config, client) = authenticated().await?;
    let account_id = config.account_id()?;

    let since_str = since.as_deref().map(|s| parse_since(s)).transpose()?;
    let before_str = before
        .as_deref()
        .map(parse_before_query_param)
        .transpose()?;

    let mut txs = client
        .transactions(
            account_id,
            since_str.as_deref(),
            before_str.as_deref(),
            Some(limit.min(100)),
        )
        .await?;

    // Apply local filters
    if let Some(ref cat) = category {
        txs.retain(|tx| tx.category.eq_ignore_ascii_case(cat));
    }
    if let Some(ref q) = search {
        let q_lower = q.to_lowercase();
        txs.retain(|tx| {
            tx.display_name().to_lowercase().contains(&q_lower)
                || tx.description.to_lowercase().contains(&q_lower)
                || tx.notes.to_lowercase().contains(&q_lower)
        });
    }
    if let Some(min_amount) = min {
        let min_pence = (min_amount * 100.0) as i64;
        txs.retain(|tx| tx.amount.abs() >= min_pence);
    }
    if let Some(max_amount) = max {
        let max_pence = (max_amount * 100.0) as i64;
        txs.retain(|tx| tx.amount.abs() <= max_pence);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&txs)?);
    } else {
        display::print_transactions(&txs);
    }
    Ok(())
}

async fn handle_transaction_detail(json: bool, id: &str) -> Result<()> {
    let (_config, client) = authenticated().await?;
    let tx = client.transaction(id).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&tx)?);
    } else {
        display::print_transaction_detail(&tx);
    }
    Ok(())
}

async fn handle_annotate(id: &str, key: &str, value: &str) -> Result<()> {
    let (_config, client) = authenticated().await?;
    let tx = client.annotate_transaction(id, key, value).await?;
    println!("Transaction annotated.");
    display::print_transaction_detail(&tx);
    Ok(())
}

// ── Pots ────────────────────────────────────────────────────────────────────

async fn handle_pots(json: bool, action: Option<PotAction>) -> Result<()> {
    let (config, client) = authenticated().await?;
    let account_id = config.account_id()?;

    match action.unwrap_or(PotAction::List) {
        PotAction::List => {
            let pots = client.pots(account_id).await?;
            let balance = client.balance(account_id).await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&pots)?);
            } else {
                display::print_pots(&pots, &balance.currency);
            }
        }
        PotAction::Deposit { pot, amount } => {
            let pots = client.pots(account_id).await?;
            let pot_obj = find_pot(&pots, &pot)?;
            let amount_pence = (amount * 100.0) as i64;

            let result = client
                .deposit_into_pot(&pot_obj.id, account_id, amount_pence)
                .await?;
            println!(
                "Deposited {} into \"{}\"",
                format_money(amount_pence, &result.currency).green(),
                result.name
            );
            println!(
                "New pot balance: {}",
                format_money(result.balance, &result.currency).green()
            );
        }
        PotAction::Withdraw { pot, amount } => {
            let pots = client.pots(account_id).await?;
            let pot_obj = find_pot(&pots, &pot)?;
            let amount_pence = (amount * 100.0) as i64;

            let result = client
                .withdraw_from_pot(&pot_obj.id, account_id, amount_pence)
                .await?;
            println!(
                "Withdrew {} from \"{}\"",
                format_money(amount_pence, &result.currency).yellow(),
                result.name
            );
            println!(
                "New pot balance: {}",
                format_money(result.balance, &result.currency).green()
            );
        }
    }
    Ok(())
}

fn find_pot<'a>(pots: &'a [models::Pot], name_or_id: &str) -> Result<&'a models::Pot> {
    // Try exact ID match first
    if let Some(pot) = pots.iter().find(|p| p.id == name_or_id) {
        return Ok(pot);
    }
    // Then case-insensitive name match
    let lower = name_or_id.to_lowercase();
    let matches: Vec<_> = pots
        .iter()
        .filter(|p| p.name.to_lowercase().contains(&lower))
        .collect();

    match matches.len() {
        0 => anyhow::bail!("No pot found matching \"{name_or_id}\""),
        1 => Ok(matches[0]),
        _ => {
            let names: Vec<_> = matches.iter().map(|p| p.name.as_str()).collect();
            anyhow::bail!(
                "Multiple pots match \"{name_or_id}\": {}. Be more specific.",
                names.join(", ")
            );
        }
    }
}

// ── Search ──────────────────────────────────────────────────────────────────

async fn handle_search(
    json: bool,
    query: &str,
    limit: u32,
    since: Option<&str>,
    before: Option<&str>,
) -> Result<()> {
    let (config, client) = authenticated().await?;
    let account_id = config.account_id()?;

    if before.is_some() && since.is_none() {
        anyhow::bail!("--to/--before requires --since/--from (lower bound).");
    }

    let since_str = parse_since(since.unwrap_or("90d"))?;
    let before_str = before.map(parse_before_query_param).transpose()?;
    let txs = client
        .all_transactions(account_id, Some(&since_str), before_str.as_deref())
        .await?;

    let q_lower = query.to_lowercase();
    let mut results: Vec<_> = txs
        .into_iter()
        .filter(|tx| {
            tx.display_name().to_lowercase().contains(&q_lower)
                || tx.description.to_lowercase().contains(&q_lower)
                || tx.notes.to_lowercase().contains(&q_lower)
                || tx.category.to_lowercase().contains(&q_lower)
        })
        .collect();

    results.truncate(limit as usize);

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        if results.is_empty() {
            println!("No transactions matching \"{}\"", query);
        } else {
            println!(
                "{} results for \"{}\":\n",
                results.len(),
                query.bold()
            );
            display::print_transactions(&results);
        }
    }
    Ok(())
}

// ── Insights ────────────────────────────────────────────────────────────────

async fn handle_insights(
    action: Option<InsightAction>,
    since: Option<&str>,
    before: Option<&str>,
) -> Result<()> {
    let (config, client) = authenticated().await?;
    let account_id = config.account_id()?;

    if before.is_some() && since.is_none() {
        anyhow::bail!("--to/--before requires --since/--from (lower bound).");
    }

    let since_str = parse_since(since.unwrap_or("30d"))?;
    let before_str = before.map(parse_before_query_param).transpose()?;
    let txs = client
        .all_transactions(account_id, Some(&since_str), before_str.as_deref())
        .await?;

    if txs.is_empty() {
        println!("No transactions found in the given period.");
        return Ok(());
    }

    match action.unwrap_or(InsightAction::Report) {
        InsightAction::Report => analytics::full_report(&txs),
        InsightAction::Categories => analytics::category_breakdown(&txs),
        InsightAction::Merchants => analytics::top_merchants(&txs, 15),
        InsightAction::Daily => analytics::daily_spending(&txs),
        InsightAction::Weekly => analytics::weekly_spending(&txs),
        InsightAction::Predict => analytics::predict_monthly(&txs),
        InsightAction::Recurring => analytics::detect_recurring(&txs),
    }
    Ok(())
}

// ── Webhooks ────────────────────────────────────────────────────────────────

async fn handle_webhooks(json: bool, action: Option<WebhookAction>) -> Result<()> {
    let (config, client) = authenticated().await?;
    let account_id = config.account_id()?;

    match action.unwrap_or(WebhookAction::List) {
        WebhookAction::List => {
            let whs = client.webhooks(account_id).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&whs)?);
            } else {
                display::print_webhooks(&whs);
            }
        }
        WebhookAction::Add { url } => {
            let wh = client.register_webhook(account_id, &url).await?;
            println!("Webhook registered: {}", wh.id);
        }
        WebhookAction::Remove { id } => {
            client.delete_webhook(&id).await?;
            println!("Webhook deleted.");
        }
    }
    Ok(())
}

// ── Feed ────────────────────────────────────────────────────────────────────

async fn handle_feed(title: &str, body: &str) -> Result<()> {
    let (config, client) = authenticated().await?;
    let account_id = config.account_id()?;

    client
        .create_feed_item(account_id, title, body, None)
        .await?;
    println!("Feed item sent to your Monzo app.");
    Ok(())
}

// ── Export ───────────────────────────────────────────────────────────────────

async fn handle_export(
    format: &str,
    output: &str,
    since: Option<&str>,
    before: Option<&str>,
) -> Result<()> {
    let (config, client) = authenticated().await?;
    let account_id = config.account_id()?;

    if before.is_some() && since.is_none() {
        anyhow::bail!("--to/--before requires --since/--from (lower bound).");
    }

    let since_str = parse_since(since.unwrap_or("90d"))?;
    let before_str = before.map(parse_before_query_param).transpose()?;
    let txs = client
        .all_transactions(account_id, Some(&since_str), before_str.as_deref())
        .await?;

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&txs)?;
            std::fs::write(output, json)?;
        }
        "csv" => {
            let mut wtr = csv::Writer::from_path(output)?;
            wtr.write_record(["date", "description", "merchant", "category", "amount", "currency", "notes"])?;
            for tx in &txs {
                let merchant = tx
                    .merchant
                    .as_ref()
                    .map(|m| m.name.as_str())
                    .unwrap_or("");
                wtr.write_record([
                    &tx.created.format("%Y-%m-%d %H:%M:%S").to_string(),
                    &tx.description,
                    merchant,
                    &tx.category,
                    &format!("{:.2}", tx.amount as f64 / 100.0),
                    &tx.currency,
                    &tx.notes,
                ])?;
            }
            wtr.flush()?;
        }
        _ => anyhow::bail!("Unsupported format: {format}. Use 'csv' or 'json'."),
    }

    println!(
        "Exported {} transactions to {}",
        txs.len(),
        output.bold()
    );
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Inclusive calendar end date (YYYY-MM-DD) → RFC3339 instant for Monzo's `before` (exclusive).
fn parse_to_inclusive_end_date(s: &str) -> Result<String> {
    let date = chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
        .with_context(|| format!("Invalid date (expected YYYY-MM-DD): {s}"))?;
    let next = date
        .succ_opt()
        .with_context(|| format!("Invalid or out-of-range date: {s}"))?;
    let dt = next
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    Ok(dt.to_rfc3339())
}

/// `before` / `--to`: plain `YYYY-MM-DD` is treated as an inclusive last day; anything else is sent as-is (RFC3339 or Monzo transaction id).
fn parse_before_query_param(s: &str) -> Result<String> {
    let t = s.trim();
    if t.len() == 10 && t.as_bytes()[4] == b'-' && t.as_bytes()[7] == b'-' {
        if chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d").is_ok() {
            return parse_to_inclusive_end_date(t);
        }
    }
    Ok(t.to_string())
}

/// Parse a "since" value that can be a date (YYYY-MM-DD) or a duration (7d, 2w, 3m, 1y)
fn parse_since(s: &str) -> Result<String> {
    // Try parsing as a date first
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        return Ok(dt.to_rfc3339());
    }

    // Parse as duration
    let (num, unit) = s.split_at(s.len() - 1);
    let num: i64 = num
        .parse()
        .with_context(|| format!("Invalid duration: {s}. Use formats like 7d, 2w, 3m, 1y"))?;

    let duration = match unit {
        "d" => Duration::days(num),
        "w" => Duration::weeks(num),
        "m" => Duration::days(num * 30),
        "y" => Duration::days(num * 365),
        _ => anyhow::bail!("Unknown duration unit: {unit}. Use d (days), w (weeks), m (months), y (years)"),
    };

    let since = Utc::now() - duration;
    Ok(since.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::test_helpers::*;

    // ── parse_since ─────────────────────────────────────────────────────

    #[test]
    fn parse_since_date_format() {
        let result = parse_since("2024-06-15").unwrap();
        assert!(result.contains("2024-06-15"));
    }

    #[test]
    fn parse_since_days() {
        let result = parse_since("7d").unwrap();
        // Should be a valid RFC3339 timestamp
        let parsed = chrono::DateTime::parse_from_rfc3339(&result);
        assert!(parsed.is_ok());
        // Should be roughly 7 days ago
        let dt = parsed.unwrap();
        let diff = Utc::now().signed_duration_since(dt);
        assert!((diff.num_days() - 7).abs() <= 1);
    }

    #[test]
    fn parse_since_weeks() {
        let result = parse_since("2w").unwrap();
        let parsed = chrono::DateTime::parse_from_rfc3339(&result).unwrap();
        let diff = Utc::now().signed_duration_since(parsed);
        assert!((diff.num_days() - 14).abs() <= 1);
    }

    #[test]
    fn parse_since_months() {
        let result = parse_since("3m").unwrap();
        let parsed = chrono::DateTime::parse_from_rfc3339(&result).unwrap();
        let diff = Utc::now().signed_duration_since(parsed);
        assert!((diff.num_days() - 90).abs() <= 1);
    }

    #[test]
    fn parse_since_year() {
        let result = parse_since("1y").unwrap();
        let parsed = chrono::DateTime::parse_from_rfc3339(&result).unwrap();
        let diff = Utc::now().signed_duration_since(parsed);
        assert!((diff.num_days() - 365).abs() <= 1);
    }

    #[test]
    fn parse_since_invalid_unit() {
        assert!(parse_since("5x").is_err());
    }

    #[test]
    fn parse_since_invalid_number() {
        assert!(parse_since("abcd").is_err());
    }

    #[test]
    fn parse_since_invalid_date() {
        assert!(parse_since("not-a-date").is_err());
    }

    // ── parse_to_inclusive_end_date / parse_before_query_param ──────────

    #[test]
    fn parse_to_inclusive_end_january() {
        let end = parse_to_inclusive_end_date("2024-01-31").unwrap();
        assert!(end.starts_with("2024-02-01"));
    }

    #[test]
    fn parse_before_plain_date_is_inclusive_end() {
        let b = parse_before_query_param("2024-01-31").unwrap();
        assert!(b.contains("2024-02-01"));
    }

    #[test]
    fn parse_before_passes_through_rfc3339() {
        let raw = "2024-01-31T23:59:59Z";
        assert_eq!(parse_before_query_param(raw).unwrap(), raw);
    }

    // ── find_pot ────────────────────────────────────────────────────────

    #[test]
    fn find_pot_by_exact_id() {
        let pots = vec![make_pot("Holiday", 5000), make_pot("Emergency", 10000)];
        let found = find_pot(&pots, &pots[1].id).unwrap();
        assert_eq!(found.name, "Emergency");
    }

    #[test]
    fn find_pot_by_name_case_insensitive() {
        let pots = vec![make_pot("Holiday Fund", 5000), make_pot("Emergency", 10000)];
        let found = find_pot(&pots, "holiday").unwrap();
        assert_eq!(found.name, "Holiday Fund");
    }

    #[test]
    fn find_pot_by_partial_name() {
        let pots = vec![make_pot("Holiday Fund", 5000), make_pot("Emergency", 10000)];
        let found = find_pot(&pots, "Emerg").unwrap();
        assert_eq!(found.name, "Emergency");
    }

    #[test]
    fn find_pot_no_match() {
        let pots = vec![make_pot("Holiday", 5000)];
        assert!(find_pot(&pots, "Savings").is_err());
    }

    #[test]
    fn find_pot_ambiguous_match() {
        let pots = vec![
            make_pot("Holiday Fund", 5000),
            make_pot("Holiday Savings", 3000),
        ];
        let result = find_pot(&pots, "Holiday");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Multiple pots"));
    }

    #[test]
    fn find_pot_empty_list() {
        let pots: Vec<models::Pot> = vec![];
        assert!(find_pot(&pots, "anything").is_err());
    }
}
