use colored::Colorize;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, CellAlignment, Color, Table};

use crate::models::*;

pub fn print_accounts(accounts: &[Account]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["ID", "Description", "Type", "Currency", "Created"]);

    for acc in accounts {
        let created = acc
            .created
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        table.add_row(vec![
            &acc.id,
            &acc.description,
            &acc.account_type,
            &acc.currency,
            &created,
        ]);
    }

    println!("{table}");
}

pub fn print_balance(balance: &Balance) {
    let cur = &balance.currency;
    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["", "Amount"]);

    table.add_row(vec![
        Cell::new("Balance"),
        Cell::new(format_money(balance.balance, cur))
            .set_alignment(CellAlignment::Right)
            .fg(Color::Green),
    ]);
    table.add_row(vec![
        Cell::new("Total Balance"),
        Cell::new(format_money(balance.total_balance, cur))
            .set_alignment(CellAlignment::Right)
            .fg(Color::Green),
    ]);
    table.add_row(vec![
        Cell::new("Spent Today"),
        Cell::new(format_money(balance.spend_today, cur))
            .set_alignment(CellAlignment::Right)
            .fg(Color::Red),
    ]);
    if balance.balance_including_flexible_savings != 0 {
        table.add_row(vec![
            Cell::new("Including Savings"),
            Cell::new(format_money(balance.balance_including_flexible_savings, cur))
                .set_alignment(CellAlignment::Right)
                .fg(Color::Cyan),
        ]);
    }

    println!("{table}");
}

pub fn print_transactions(transactions: &[Transaction]) {
    if transactions.is_empty() {
        println!("No transactions found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["Date", "Description", "Category", "Amount", "Balance"]);

    for tx in transactions {
        let amount_str = format_money(tx.amount, &tx.currency);
        let balance_str = tx
            .account_balance
            .map(|b| format_money(b, &tx.currency))
            .unwrap_or_default();

        let amount_color = if tx.amount < 0 {
            Color::Red
        } else {
            Color::Green
        };

        let declined = if tx.is_declined() { " [DECLINED]" } else { "" };

        table.add_row(vec![
            Cell::new(tx.created.format("%Y-%m-%d %H:%M")),
            Cell::new(format!("{}{}", tx.display_name(), declined)),
            Cell::new(&tx.category),
            Cell::new(&amount_str)
                .set_alignment(CellAlignment::Right)
                .fg(amount_color),
            Cell::new(&balance_str).set_alignment(CellAlignment::Right),
        ]);
    }

    println!("{table}");
    println!("  {} transactions", transactions.len());
}

pub fn print_transaction_detail(tx: &Transaction) {
    let cur = &tx.currency;
    println!("{}", "Transaction Detail".bold());
    println!("  ID:          {}", tx.id);
    println!("  Description: {}", tx.display_name());
    println!(
        "  Amount:      {}",
        if tx.amount < 0 {
            format_money(tx.amount, cur).red().to_string()
        } else {
            format_money(tx.amount, cur).green().to_string()
        }
    );
    println!("  Category:    {}", tx.category);
    println!("  Created:     {}", tx.created.format("%Y-%m-%d %H:%M:%S"));
    if !tx.notes.is_empty() {
        println!("  Notes:       {}", tx.notes);
    }
    if let Some(ref m) = tx.merchant {
        println!("  Merchant:    {} {}", m.emoji.as_deref().unwrap_or(""), m.name);
        if let Some(ref addr) = m.address {
            if !addr.short_formatted.is_empty() {
                println!("  Location:    {}", addr.short_formatted);
            }
        }
        println!("  Online:      {}", if m.online { "Yes" } else { "No" });
    }
    if let Some(local) = tx.local_amount {
        if let Some(ref local_cur) = tx.local_currency {
            if local_cur != cur {
                println!(
                    "  Local:       {}",
                    format_money(local, local_cur)
                );
            }
        }
    }
    if let Some(bal) = tx.account_balance {
        println!("  Balance:     {}", format_money(bal, cur));
    }
    if !tx.metadata.is_empty() {
        println!("  Metadata:");
        for (k, v) in &tx.metadata {
            println!("    {k}: {v}");
        }
    }
}

pub fn print_pots(pots: &[Pot], currency: &str) {
    if pots.is_empty() {
        println!("No pots found.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["Name", "Balance", "Goal", "Type", "Round-up"]);

    let mut total: i64 = 0;

    for pot in pots {
        total += pot.balance;
        let goal = pot
            .goal_amount
            .map(|g| {
                let pct = if g > 0 {
                    (pot.balance as f64 / g as f64 * 100.0) as u32
                } else {
                    0
                };
                format!("{} ({}%)", format_money(g, currency), pct)
            })
            .unwrap_or_else(|| "-".to_string());

        let pot_type = pot.pot_type.as_deref().unwrap_or("-");
        let roundup = if pot.round_up { "Yes" } else { "-" };

        table.add_row(vec![
            Cell::new(&pot.name),
            Cell::new(format_money(pot.balance, currency))
                .set_alignment(CellAlignment::Right)
                .fg(Color::Green),
            Cell::new(&goal).set_alignment(CellAlignment::Right),
            Cell::new(pot_type),
            Cell::new(roundup),
        ]);
    }

    println!("{table}");
    println!(
        "  {} pots | Total: {}",
        pots.len(),
        format_money(total, currency).green()
    );
}

pub fn print_webhooks(webhooks: &[Webhook]) {
    if webhooks.is_empty() {
        println!("No webhooks registered.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["ID", "URL"]);

    for wh in webhooks {
        table.add_row(vec![&wh.id, &wh.url]);
    }

    println!("{table}");
}
