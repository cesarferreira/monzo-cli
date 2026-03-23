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

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use chrono::Utc;

    pub fn make_tx(amount: i64, description: &str, category: &str) -> Transaction {
        Transaction {
            id: format!("tx_{}", uuid::Uuid::new_v4()),
            amount,
            currency: "GBP".to_string(),
            created: Utc::now(),
            settled: None,
            description: description.to_string(),
            category: category.to_string(),
            notes: String::new(),
            merchant: None,
            metadata: serde_json::Map::new(),
            is_load: false,
            decline_reason: None,
            account_balance: Some(10000),
            local_amount: None,
            local_currency: None,
        }
    }

    pub fn make_tx_with_merchant(
        amount: i64,
        merchant_name: &str,
        category: &str,
    ) -> Transaction {
        let mut tx = make_tx(amount, merchant_name, category);
        tx.merchant = Some(Merchant {
            id: "merch_test".to_string(),
            name: merchant_name.to_string(),
            logo: None,
            emoji: Some("\u{2615}".to_string()),
            category: category.to_string(),
            address: None,
            online: false,
        });
        tx
    }

    pub fn make_tx_at(
        amount: i64,
        description: &str,
        category: &str,
        date: chrono::DateTime<Utc>,
    ) -> Transaction {
        let mut tx = make_tx(amount, description, category);
        tx.created = date;
        tx
    }

    pub fn make_pot(name: &str, balance: i64) -> Pot {
        Pot {
            id: format!("pot_{}", name.to_lowercase().replace(' ', "_")),
            name: name.to_string(),
            style: None,
            balance,
            currency: "GBP".to_string(),
            created: None,
            updated: None,
            deleted: false,
            goal_amount: None,
            pot_type: None,
            round_up: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helpers::*;

    // ── format_money ────────────────────────────────────────────────────

    #[test]
    fn format_money_positive_gbp() {
        assert_eq!(format_money(12345, "GBP"), "\u{00a3}123.45");
    }

    #[test]
    fn format_money_negative_gbp() {
        assert_eq!(format_money(-500, "GBP"), "-\u{00a3}5.00");
    }

    #[test]
    fn format_money_zero() {
        assert_eq!(format_money(0, "GBP"), "\u{00a3}0.00");
    }

    #[test]
    fn format_money_one_penny() {
        assert_eq!(format_money(1, "GBP"), "\u{00a3}0.01");
    }

    #[test]
    fn format_money_eur() {
        assert_eq!(format_money(1050, "EUR"), "\u{20ac}10.50");
    }

    #[test]
    fn format_money_usd() {
        assert_eq!(format_money(9999, "USD"), "$99.99");
    }

    #[test]
    fn format_money_unknown_currency() {
        assert_eq!(format_money(100, "JPY"), "JPY1.00");
    }

    #[test]
    fn format_money_large_amount() {
        assert_eq!(format_money(1_000_000, "GBP"), "\u{00a3}10000.00");
    }

    #[test]
    fn format_money_negative_one_penny() {
        assert_eq!(format_money(-1, "GBP"), "-\u{00a3}0.01");
    }

    // ── Transaction methods ─────────────────────────────────────────────

    #[test]
    fn display_name_uses_merchant_when_present() {
        let tx = make_tx_with_merchant(-350, "Pret A Manger", "eating_out");
        assert_eq!(tx.display_name(), "Pret A Manger");
    }

    #[test]
    fn display_name_falls_back_to_description() {
        let tx = make_tx(-350, "Some Payment", "general");
        assert_eq!(tx.display_name(), "Some Payment");
    }

    #[test]
    fn display_name_falls_back_when_merchant_name_empty() {
        let mut tx = make_tx(-350, "Fallback Desc", "general");
        tx.merchant = Some(Merchant {
            id: "merch_x".to_string(),
            name: String::new(),
            logo: None,
            emoji: None,
            category: String::new(),
            address: None,
            online: false,
        });
        assert_eq!(tx.display_name(), "Fallback Desc");
    }

    #[test]
    fn is_expense_negative_non_load() {
        let tx = make_tx(-100, "Coffee", "eating_out");
        assert!(tx.is_expense());
    }

    #[test]
    fn is_expense_positive_is_not() {
        let tx = make_tx(500, "Refund", "general");
        assert!(!tx.is_expense());
    }

    #[test]
    fn is_expense_load_is_not() {
        let mut tx = make_tx(-100, "Top Up", "general");
        tx.is_load = true;
        assert!(!tx.is_expense());
    }

    #[test]
    fn is_expense_zero_is_not() {
        let tx = make_tx(0, "Zero", "general");
        assert!(!tx.is_expense());
    }

    #[test]
    fn is_declined_with_reason() {
        let mut tx = make_tx(-100, "Declined", "general");
        tx.decline_reason = Some("INSUFFICIENT_FUNDS".to_string());
        assert!(tx.is_declined());
    }

    #[test]
    fn is_declined_without_reason() {
        let tx = make_tx(-100, "OK", "general");
        assert!(!tx.is_declined());
    }

    // ── JSON deserialization ────────────────────────────────────────────

    #[test]
    fn deserialize_balance_from_api_json() {
        let json = r#"{
            "balance": 5000,
            "total_balance": 15000,
            "currency": "GBP",
            "spend_today": -2500
        }"#;
        let balance: Balance = serde_json::from_str(json).unwrap();
        assert_eq!(balance.balance, 5000);
        assert_eq!(balance.total_balance, 15000);
        assert_eq!(balance.currency, "GBP");
        assert_eq!(balance.spend_today, -2500);
        assert_eq!(balance.balance_including_flexible_savings, 0);
    }

    #[test]
    fn deserialize_balance_with_savings() {
        let json = r#"{
            "balance": 5000,
            "total_balance": 15000,
            "currency": "GBP",
            "spend_today": -2500,
            "balance_including_flexible_savings": 50000
        }"#;
        let balance: Balance = serde_json::from_str(json).unwrap();
        assert_eq!(balance.balance_including_flexible_savings, 50000);
    }

    #[test]
    fn deserialize_account_from_api_json() {
        let json = r#"{
            "id": "acc_123",
            "description": "user_123",
            "type": "uk_retail",
            "currency": "GBP"
        }"#;
        let account: Account = serde_json::from_str(json).unwrap();
        assert_eq!(account.id, "acc_123");
        assert_eq!(account.account_type, "uk_retail");
        assert!(!account.closed);
    }

    #[test]
    fn deserialize_transaction_minimal() {
        let json = r#"{
            "id": "tx_abc",
            "amount": -550,
            "currency": "GBP",
            "created": "2025-01-15T10:30:00Z",
            "description": "TESCO"
        }"#;
        let tx: Transaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.id, "tx_abc");
        assert_eq!(tx.amount, -550);
        assert_eq!(tx.description, "TESCO");
        assert!(tx.merchant.is_none());
        assert!(tx.notes.is_empty());
    }

    #[test]
    fn deserialize_transaction_with_merchant() {
        let json = r#"{
            "id": "tx_abc",
            "amount": -350,
            "currency": "GBP",
            "created": "2025-01-15T10:30:00Z",
            "description": "PRET",
            "merchant": {
                "id": "merch_xyz",
                "name": "Pret A Manger",
                "category": "eating_out",
                "online": false
            }
        }"#;
        let tx: Transaction = serde_json::from_str(json).unwrap();
        let m = tx.merchant.as_ref().unwrap();
        assert_eq!(m.name, "Pret A Manger");
        assert_eq!(m.category, "eating_out");
        assert!(!m.online);
    }

    #[test]
    fn deserialize_pot() {
        let json = r#"{
            "id": "pot_abc",
            "name": "Holiday Fund",
            "balance": 50000,
            "currency": "GBP",
            "deleted": false,
            "goal_amount": 100000,
            "round_up": true
        }"#;
        let pot: Pot = serde_json::from_str(json).unwrap();
        assert_eq!(pot.name, "Holiday Fund");
        assert_eq!(pot.balance, 50000);
        assert_eq!(pot.goal_amount, Some(100000));
        assert!(pot.round_up);
        assert!(!pot.deleted);
    }

    #[test]
    fn deserialize_accounts_response() {
        let json = r#"{
            "accounts": [
                {"id": "acc_1", "description": "Personal", "currency": "GBP"},
                {"id": "acc_2", "description": "Joint", "currency": "GBP", "closed": true}
            ]
        }"#;
        let resp: AccountsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.accounts.len(), 2);
        assert!(!resp.accounts[0].closed);
        assert!(resp.accounts[1].closed);
    }

    #[test]
    fn deserialize_webhook() {
        let json = r#"{
            "id": "whk_123",
            "account_id": "acc_456",
            "url": "https://example.com/hook"
        }"#;
        let wh: Webhook = serde_json::from_str(json).unwrap();
        assert_eq!(wh.id, "whk_123");
        assert_eq!(wh.url, "https://example.com/hook");
    }

    #[test]
    fn deserialize_token_response() {
        let json = r#"{
            "access_token": "tok_abc",
            "refresh_token": "ref_xyz",
            "expires_in": 21600,
            "token_type": "Bearer",
            "user_id": "user_123"
        }"#;
        let tok: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(tok.access_token, "tok_abc");
        assert_eq!(tok.refresh_token, Some("ref_xyz".to_string()));
        assert_eq!(tok.expires_in, Some(21600));
        assert_eq!(tok.user_id, "user_123");
    }

    #[test]
    fn deserialize_token_response_without_refresh() {
        let json = r#"{
            "access_token": "tok_abc",
            "token_type": "Bearer",
            "user_id": "user_123"
        }"#;
        let tok: TokenResponse = serde_json::from_str(json).unwrap();
        assert!(tok.refresh_token.is_none());
        assert!(tok.expires_in.is_none());
    }

    // ── Serialization round-trip ────────────────────────────────────────

    #[test]
    fn transaction_serialize_roundtrip() {
        let tx = make_tx_with_merchant(-1500, "Nando's", "eating_out");
        let json = serde_json::to_string(&tx).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["amount"], -1500);
        assert_eq!(parsed["merchant"]["name"], "Nando's");
    }

    #[test]
    fn pot_serialize_roundtrip() {
        let pot = make_pot("Savings", 25000);
        let json = serde_json::to_string(&pot).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "Savings");
        assert_eq!(parsed["balance"], 25000);
    }
}
