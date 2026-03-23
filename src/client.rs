use anyhow::Result;
use reqwest::Client;

use crate::config::Config;
use crate::models::*;

const BASE_URL: &str = "https://api.monzo.com";

pub struct MonzoClient {
    http: Client,
    token: String,
}

impl MonzoClient {
    pub fn new(config: &Config) -> Result<Self> {
        let token = config.access_token()?.to_string();
        let http = Client::builder()
            .user_agent("monzo/1.0")
            .build()?;
        Ok(Self { http, token })
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    // ── Ping / WhoAmI ───────────────────────────────────────────────────────

    pub async fn whoami(&self) -> Result<WhoAmI> {
        let resp = self
            .http
            .get(format!("{BASE_URL}/ping/whoami"))
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        Self::check_response(&resp)?;
        Ok(resp.json().await?)
    }

    // ── Accounts ────────────────────────────────────────────────────────────

    pub async fn accounts(&self, account_type: Option<&str>) -> Result<Vec<Account>> {
        let mut req = self
            .http
            .get(format!("{BASE_URL}/accounts"))
            .header("Authorization", self.auth_header());
        if let Some(at) = account_type {
            req = req.query(&[("account_type", at)]);
        }
        let resp = req.send().await?;
        Self::check_response(&resp)?;
        let data: AccountsResponse = resp.json().await?;
        Ok(data.accounts.into_iter().filter(|a| !a.closed).collect())
    }

    // ── Balance ─────────────────────────────────────────────────────────────

    pub async fn balance(&self, account_id: &str) -> Result<Balance> {
        let resp = self
            .http
            .get(format!("{BASE_URL}/balance"))
            .header("Authorization", self.auth_header())
            .query(&[("account_id", account_id)])
            .send()
            .await?;
        Self::check_response(&resp)?;
        Ok(resp.json().await?)
    }

    // ── Transactions ────────────────────────────────────────────────────────

    pub async fn transactions(
        &self,
        account_id: &str,
        since: Option<&str>,
        before: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<Transaction>> {
        let mut params: Vec<(&str, String)> = vec![
            ("account_id", account_id.to_string()),
            ("expand[]", "merchant".to_string()),
        ];
        if let Some(s) = since {
            params.push(("since", s.to_string()));
        }
        if let Some(b) = before {
            params.push(("before", b.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }
        let resp = self
            .http
            .get(format!("{BASE_URL}/transactions"))
            .header("Authorization", self.auth_header())
            .query(&params)
            .send()
            .await?;
        Self::check_response(&resp)?;
        let data: TransactionsResponse = resp.json().await?;
        Ok(data.transactions)
    }

    /// Fetch all transactions using pagination (max 100 per page)
    pub async fn all_transactions(
        &self,
        account_id: &str,
        since: Option<&str>,
        before: Option<&str>,
    ) -> Result<Vec<Transaction>> {
        let mut all = Vec::new();
        let mut current_since = since.map(|s| s.to_string());

        loop {
            let batch = self
                .transactions(
                    account_id,
                    current_since.as_deref(),
                    before,
                    Some(100),
                )
                .await?;

            if batch.is_empty() {
                break;
            }

            let last_id = batch.last().unwrap().id.clone();
            all.extend(batch);

            // Use the last transaction ID as `since` for next page
            current_since = Some(last_id);

            // Safety: if we got fewer than 100, we've reached the end
            if all.len() % 100 != 0 {
                break;
            }
        }

        Ok(all)
    }

    pub async fn transaction(&self, tx_id: &str) -> Result<Transaction> {
        let resp = self
            .http
            .get(format!("{BASE_URL}/transactions/{tx_id}"))
            .header("Authorization", self.auth_header())
            .query(&[("expand[]", "merchant")])
            .send()
            .await?;
        Self::check_response(&resp)?;
        let data: serde_json::Value = resp.json().await?;
        let tx: Transaction = serde_json::from_value(data["transaction"].clone())?;
        Ok(tx)
    }

    pub async fn annotate_transaction(
        &self,
        tx_id: &str,
        key: &str,
        value: &str,
    ) -> Result<Transaction> {
        let field = format!("metadata[{key}]");
        let resp = self
            .http
            .patch(format!("{BASE_URL}/transactions/{tx_id}"))
            .header("Authorization", self.auth_header())
            .form(&[(field.as_str(), value)])
            .send()
            .await?;
        Self::check_response(&resp)?;
        let data: serde_json::Value = resp.json().await?;
        let tx: Transaction = serde_json::from_value(data["transaction"].clone())?;
        Ok(tx)
    }

    // ── Pots ────────────────────────────────────────────────────────────────

    pub async fn pots(&self, account_id: &str) -> Result<Vec<Pot>> {
        let resp = self
            .http
            .get(format!("{BASE_URL}/pots"))
            .header("Authorization", self.auth_header())
            .query(&[("current_account_id", account_id)])
            .send()
            .await?;
        Self::check_response(&resp)?;
        let data: PotsResponse = resp.json().await?;
        Ok(data.pots.into_iter().filter(|p| !p.deleted).collect())
    }

    pub async fn deposit_into_pot(
        &self,
        pot_id: &str,
        source_account_id: &str,
        amount: i64,
    ) -> Result<Pot> {
        let dedupe = uuid::Uuid::new_v4().to_string();
        let resp = self
            .http
            .put(format!("{BASE_URL}/pots/{pot_id}/deposit"))
            .header("Authorization", self.auth_header())
            .form(&[
                ("source_account_id", source_account_id),
                ("amount", &amount.to_string()),
                ("dedupe_id", &dedupe),
            ])
            .send()
            .await?;
        Self::check_response(&resp)?;
        Ok(resp.json().await?)
    }

    pub async fn withdraw_from_pot(
        &self,
        pot_id: &str,
        destination_account_id: &str,
        amount: i64,
    ) -> Result<Pot> {
        let dedupe = uuid::Uuid::new_v4().to_string();
        let resp = self
            .http
            .put(format!("{BASE_URL}/pots/{pot_id}/withdraw"))
            .header("Authorization", self.auth_header())
            .form(&[
                ("destination_account_id", destination_account_id),
                ("amount", &amount.to_string()),
                ("dedupe_id", &dedupe),
            ])
            .send()
            .await?;
        Self::check_response(&resp)?;
        Ok(resp.json().await?)
    }

    // ── Feed Items ──────────────────────────────────────────────────────────

    pub async fn create_feed_item(
        &self,
        account_id: &str,
        title: &str,
        body: &str,
        image_url: Option<&str>,
    ) -> Result<()> {
        let img = image_url.unwrap_or("https://monzo.com/static/images/favicon.png");
        let resp = self
            .http
            .post(format!("{BASE_URL}/feed"))
            .header("Authorization", self.auth_header())
            .form(&[
                ("account_id", account_id),
                ("type", "basic"),
                ("params[title]", title),
                ("params[body]", body),
                ("params[image_url]", img),
            ])
            .send()
            .await?;
        Self::check_response(&resp)?;
        Ok(())
    }

    // ── Webhooks ────────────────────────────────────────────────────────────

    pub async fn webhooks(&self, account_id: &str) -> Result<Vec<Webhook>> {
        let resp = self
            .http
            .get(format!("{BASE_URL}/webhooks"))
            .header("Authorization", self.auth_header())
            .query(&[("account_id", account_id)])
            .send()
            .await?;
        Self::check_response(&resp)?;
        let data: WebhooksResponse = resp.json().await?;
        Ok(data.webhooks)
    }

    pub async fn register_webhook(&self, account_id: &str, url: &str) -> Result<Webhook> {
        let resp = self
            .http
            .post(format!("{BASE_URL}/webhooks"))
            .header("Authorization", self.auth_header())
            .form(&[("account_id", account_id), ("url", url)])
            .send()
            .await?;
        Self::check_response(&resp)?;
        let data: serde_json::Value = resp.json().await?;
        let wh: Webhook = serde_json::from_value(data["webhook"].clone())?;
        Ok(wh)
    }

    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        let resp = self
            .http
            .delete(format!("{BASE_URL}/webhooks/{webhook_id}"))
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        Self::check_response(&resp)?;
        Ok(())
    }

    // ── Token Refresh ───────────────────────────────────────────────────────

    pub async fn refresh_token(
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<TokenResponse> {
        let http = Client::new();
        let resp = http
            .post(format!("{BASE_URL}/oauth2/token"))
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Token refresh failed ({status}): {body}");
        }
        Ok(resp.json().await?)
    }

    // ── Error Handling ──────────────────────────────────────────────────────

    fn check_response(resp: &reqwest::Response) -> Result<()> {
        if resp.status() == 401 {
            anyhow::bail!(
                "Authentication failed (401). Your token may have expired.\n\
                 Run `monzo auth refresh` to refresh your token, or\n\
                 Run `monzo auth login` to re-authenticate."
            );
        }
        if resp.status() == 429 {
            anyhow::bail!("Rate limited (429). Please wait a moment and try again.");
        }
        if !resp.status().is_success() {
            anyhow::bail!("API error: {}", resp.status());
        }
        Ok(())
    }
}
