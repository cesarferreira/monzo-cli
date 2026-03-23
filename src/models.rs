use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Accounts ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AccountsResponse {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub created: Option<DateTime<Utc>>,
    #[serde(rename = "type", default)]
    pub account_type: String,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub closed: bool,
}

// ── Balance ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Balance {
    pub balance: i64,
    pub total_balance: i64,
    pub currency: String,
    pub spend_today: i64,
    #[serde(default)]
    pub balance_including_flexible_savings: i64,
}

// ── Transactions ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    pub id: String,
    pub amount: i64,
    pub currency: String,
    pub created: DateTime<Utc>,
    #[serde(default)]
    pub settled: Option<String>,
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub merchant: Option<Merchant>,
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub is_load: bool,
    #[serde(default)]
    pub decline_reason: Option<String>,
    #[serde(default)]
    pub account_balance: Option<i64>,
    #[serde(default)]
    pub local_amount: Option<i64>,
    #[serde(default)]
    pub local_currency: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Merchant {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub emoji: Option<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub address: Option<Address>,
    #[serde(default)]
    pub online: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Address {
    #[serde(default)]
    pub short_formatted: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub latitude: Option<f64>,
    #[serde(default)]
    pub longitude: Option<f64>,
}

// ── Pots ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PotsResponse {
    pub pots: Vec<Pot>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pot {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub style: Option<String>,
    pub balance: i64,
    pub currency: String,
    #[serde(default)]
    pub created: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated: Option<DateTime<Utc>>,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub goal_amount: Option<i64>,
    #[serde(rename = "type", default)]
    pub pot_type: Option<String>,
    #[serde(default)]
    pub round_up: bool,
}

// ── Webhooks ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WebhooksResponse {
    pub webhooks: Vec<Webhook>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Webhook {
    pub id: String,
    pub account_id: String,
    pub url: String,
}

// ── WhoAmI ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WhoAmI {
    #[allow(dead_code)]
    pub authenticated: bool,
    pub client_id: String,
    pub user_id: String,
}

// ── OAuth Token ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: String,
    pub user_id: String,
    pub client_id: Option<String>,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

impl Transaction {
    pub fn display_name(&self) -> &str {
        if let Some(ref m) = self.merchant {
            if !m.name.is_empty() {
                return &m.name;
            }
        }
        &self.description
    }

    pub fn is_expense(&self) -> bool {
        self.amount < 0 && !self.is_load
    }

    pub fn is_declined(&self) -> bool {
        self.decline_reason.is_some()
    }
}

/// Format a minor-unit amount (pence) to a currency string
pub fn format_money(amount: i64, currency: &str) -> String {
    let symbol = match currency {
        "GBP" => "\u{00a3}",
        "EUR" => "\u{20ac}",
        "USD" => "$",
        _ => currency,
    };
    let abs = amount.unsigned_abs();
    let major = abs / 100;
    let minor = abs % 100;
    if amount < 0 {
        format!("-{symbol}{major}.{minor:02}")
    } else {
        format!("{symbol}{major}.{minor:02}")
    }
}
