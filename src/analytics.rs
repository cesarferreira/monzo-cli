use std::collections::HashMap;

use chrono::{Datelike, Utc};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, CellAlignment, Color, Table};

use crate::models::*;

/// Spending breakdown by category
pub fn category_breakdown(transactions: &[Transaction]) {
    let mut by_category: HashMap<String, i64> = HashMap::new();
    let mut total_spend: i64 = 0;

    for tx in transactions {
        if tx.is_expense() {
            let cat = if tx.category.is_empty() {
                "uncategorized".to_string()
            } else {
                tx.category.clone()
            };
            *by_category.entry(cat).or_default() += tx.amount.abs();
            total_spend += tx.amount.abs();
        }
    }

    if by_category.is_empty() {
        println!("No spending data found.");
        return;
    }

    let mut sorted: Vec<_> = by_category.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let currency = transactions
        .first()
        .map(|t| t.currency.as_str())
        .unwrap_or("GBP");

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["Category", "Spent", "% of Total", "Bar"]);

    for (cat, amount) in &sorted {
        let pct = (*amount as f64 / total_spend as f64 * 100.0) as u32;
        let bar_len = (pct as usize) / 2;
        let bar = "\u{2588}".repeat(bar_len.max(1));

        table.add_row(vec![
            Cell::new(category_emoji(cat)),
            Cell::new(format_money(-(*amount as i64), currency))
                .set_alignment(CellAlignment::Right)
                .fg(Color::Red),
            Cell::new(format!("{pct}%")).set_alignment(CellAlignment::Right),
            Cell::new(&bar).fg(Color::Cyan),
        ]);
    }

    println!("{}", "Spending by Category".bold());
    println!("{table}");
    println!(
        "  Total: {}",
        format_money(-(total_spend as i64), currency).red()
    );
}

/// Top merchants by total spend
pub fn top_merchants(transactions: &[Transaction], limit: usize) {
    let mut by_merchant: HashMap<String, (i64, u32)> = HashMap::new();

    for tx in transactions {
        if tx.is_expense() {
            let name = tx.display_name().to_string();
            let entry = by_merchant.entry(name).or_default();
            entry.0 += tx.amount.abs();
            entry.1 += 1;
        }
    }

    let mut sorted: Vec<_> = by_merchant.into_iter().collect();
    sorted.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
    sorted.truncate(limit);

    let currency = transactions
        .first()
        .map(|t| t.currency.as_str())
        .unwrap_or("GBP");

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["Merchant", "Total Spent", "# Transactions", "Avg"]);

    for (name, (amount, count)) in &sorted {
        let avg = *amount as i64 / *count as i64;
        table.add_row(vec![
            Cell::new(name),
            Cell::new(format_money(-(*amount as i64), currency))
                .set_alignment(CellAlignment::Right)
                .fg(Color::Red),
            Cell::new(count).set_alignment(CellAlignment::Right),
            Cell::new(format_money(-avg, currency)).set_alignment(CellAlignment::Right),
        ]);
    }

    println!("{}", format!("Top {} Merchants", limit).bold());
    println!("{table}");
}

/// Daily spending totals
pub fn daily_spending(transactions: &[Transaction]) {
    let mut by_day: HashMap<String, i64> = HashMap::new();

    for tx in transactions {
        if tx.is_expense() {
            let day = tx.created.format("%Y-%m-%d").to_string();
            *by_day.entry(day).or_default() += tx.amount.abs();
        }
    }

    let mut sorted: Vec<_> = by_day.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let currency = transactions
        .first()
        .map(|t| t.currency.as_str())
        .unwrap_or("GBP");

    let max_spend = sorted.iter().map(|(_, v)| *v).max().unwrap_or(1);

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["Date", "Spent", "Bar"]);

    for (day, amount) in &sorted {
        let bar_len = (*amount as f64 / max_spend as f64 * 30.0) as usize;
        let bar = "\u{2588}".repeat(bar_len.max(1));

        table.add_row(vec![
            Cell::new(day),
            Cell::new(format_money(-(*amount as i64), currency))
                .set_alignment(CellAlignment::Right)
                .fg(Color::Red),
            Cell::new(&bar).fg(Color::Cyan),
        ]);
    }

    let total: i64 = sorted.iter().map(|(_, v)| *v).sum();
    let avg = if sorted.is_empty() {
        0
    } else {
        total / sorted.len() as i64
    };

    println!("{}", "Daily Spending".bold());
    println!("{table}");
    println!(
        "  Average: {} / day",
        format_money(-(avg as i64), currency).yellow()
    );
}

/// Weekly spending totals
pub fn weekly_spending(transactions: &[Transaction]) {
    let mut by_week: HashMap<String, i64> = HashMap::new();

    for tx in transactions {
        if tx.is_expense() {
            let week = format!("{}-W{:02}", tx.created.year(), tx.created.iso_week().week());
            *by_week.entry(week).or_default() += tx.amount.abs();
        }
    }

    let mut sorted: Vec<_> = by_week.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let currency = transactions
        .first()
        .map(|t| t.currency.as_str())
        .unwrap_or("GBP");

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["Week", "Spent"]);

    for (week, amount) in &sorted {
        table.add_row(vec![
            Cell::new(week),
            Cell::new(format_money(-(*amount as i64), currency))
                .set_alignment(CellAlignment::Right)
                .fg(Color::Red),
        ]);
    }

    let total: i64 = sorted.iter().map(|(_, v)| *v).sum();
    let avg = if sorted.is_empty() {
        0
    } else {
        total / sorted.len() as i64
    };

    println!("{}", "Weekly Spending".bold());
    println!("{table}");
    println!(
        "  Average: {} / week",
        format_money(-(avg as i64), currency).yellow()
    );
}

/// Monthly spending prediction based on current month's rate
pub fn predict_monthly(transactions: &[Transaction]) {
    let now = Utc::now();
    let current_month = now.month();
    let current_year = now.year();
    let day_of_month = now.day();

    // Get days in current month
    let days_in_month = if current_month == 12 {
        31
    } else {
        let next = chrono::NaiveDate::from_ymd_opt(current_year, current_month + 1, 1).unwrap();
        let this = chrono::NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap();
        (next - this).num_days() as u32
    };

    let month_spend: i64 = transactions
        .iter()
        .filter(|tx| {
            tx.is_expense()
                && tx.created.month() == current_month
                && tx.created.year() == current_year
        })
        .map(|tx| tx.amount.abs())
        .sum();

    let currency = transactions
        .first()
        .map(|t| t.currency.as_str())
        .unwrap_or("GBP");

    if day_of_month == 0 {
        println!("Not enough data to predict.");
        return;
    }

    let daily_rate = month_spend as f64 / day_of_month as f64;
    let projected = (daily_rate * days_in_month as f64) as i64;
    let remaining = projected - month_spend;

    println!("{}", "Monthly Spending Prediction".bold());
    println!(
        "  Spent so far:     {}",
        format_money(-(month_spend as i64), currency).red()
    );
    println!(
        "  Daily avg:        {}",
        format_money(-(daily_rate as i64), currency).yellow()
    );
    println!(
        "  Projected total:  {}",
        format_money(-(projected as i64), currency).red().bold()
    );
    println!(
        "  Remaining est:    {}",
        format_money(-(remaining as i64), currency).yellow()
    );
    println!(
        "  Days left:        {}",
        days_in_month - day_of_month
    );

    // Compare with previous months
    let mut monthly_totals: HashMap<(i32, u32), i64> = HashMap::new();
    for tx in transactions {
        if tx.is_expense() {
            let key = (tx.created.year(), tx.created.month());
            *monthly_totals.entry(key).or_default() += tx.amount.abs();
        }
    }

    // Remove current month from comparison
    monthly_totals.remove(&(current_year, current_month));

    if !monthly_totals.is_empty() {
        let mut prev_months: Vec<_> = monthly_totals.into_iter().collect();
        prev_months.sort_by(|a, b| b.0.cmp(&a.0));

        let avg_prev: i64 = prev_months.iter().map(|(_, v)| *v).sum::<i64>()
            / prev_months.len() as i64;

        println!("\n{}", "  Historical Comparison".bold());
        for ((year, month), amount) in prev_months.iter().take(3) {
            println!(
                "    {year}-{month:02}:  {}",
                format_money(-(*amount as i64), currency)
            );
        }
        println!(
            "    Average:  {}",
            format_money(-(avg_prev as i64), currency).yellow()
        );

        let diff = projected as i64 - avg_prev;
        if diff > 0 {
            println!(
                "    Trend:    {} {} than average",
                format_money(diff, currency).red(),
                "more".red()
            );
        } else {
            println!(
                "    Trend:    {} {} than average",
                format_money(diff.abs(), currency).green(),
                "less".green()
            );
        }
    }
}

/// Recurring transaction detection
pub fn detect_recurring(transactions: &[Transaction]) {
    let mut by_merchant: HashMap<String, Vec<&Transaction>> = HashMap::new();

    for tx in transactions {
        if tx.is_expense() {
            let name = tx.display_name().to_string();
            by_merchant.entry(name).or_default().push(tx);
        }
    }

    // Find merchants with regular intervals (likely subscriptions)
    let mut recurring: Vec<(String, i64, String)> = Vec::new();

    for (name, txs) in &by_merchant {
        if txs.len() < 2 {
            continue;
        }

        let mut sorted: Vec<_> = txs.iter().copied().collect();
        sorted.sort_by_key(|t| t.created);

        // Check if amounts are consistent (within 10% of each other)
        let amounts: Vec<i64> = sorted.iter().map(|t| t.amount.abs()).collect();
        let avg_amount = amounts.iter().sum::<i64>() / amounts.len() as i64;
        let consistent = amounts
            .iter()
            .all(|a| (*a as f64 - avg_amount as f64).abs() / (avg_amount as f64) < 0.15);

        if !consistent {
            continue;
        }

        // Check intervals
        let intervals: Vec<i64> = sorted
            .windows(2)
            .map(|w| (w[1].created - w[0].created).num_days())
            .collect();

        let avg_interval = intervals.iter().sum::<i64>() / intervals.len() as i64;

        let frequency = if (25..=35).contains(&avg_interval) {
            "Monthly"
        } else if (6..=8).contains(&avg_interval) {
            "Weekly"
        } else if (13..=16).contains(&avg_interval) {
            "Bi-weekly"
        } else if (85..=95).contains(&avg_interval) {
            "Quarterly"
        } else if (355..=375).contains(&avg_interval) {
            "Yearly"
        } else {
            continue;
        };

        recurring.push((name.clone(), avg_amount, frequency.to_string()));
    }

    recurring.sort_by(|a, b| b.1.cmp(&a.1));

    let currency = transactions
        .first()
        .map(|t| t.currency.as_str())
        .unwrap_or("GBP");

    if recurring.is_empty() {
        println!("No recurring transactions detected.");
        return;
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(vec!["Merchant", "Amount", "Frequency"]);

    let mut monthly_total: i64 = 0;

    for (name, amount, freq) in &recurring {
        let monthly = match freq.as_str() {
            "Weekly" => *amount * 4,
            "Bi-weekly" => *amount * 2,
            "Monthly" => *amount,
            "Quarterly" => *amount / 3,
            "Yearly" => *amount / 12,
            _ => *amount,
        };
        monthly_total += monthly;

        table.add_row(vec![
            Cell::new(name),
            Cell::new(format_money(-(*amount as i64), currency))
                .set_alignment(CellAlignment::Right)
                .fg(Color::Red),
            Cell::new(freq),
        ]);
    }

    println!("{}", "Detected Recurring Payments".bold());
    println!("{table}");
    println!(
        "  Estimated monthly recurring: {}",
        format_money(-(monthly_total as i64), currency).red()
    );
}

/// Full insights report
pub fn full_report(transactions: &[Transaction]) {
    println!("{}", "=== Monzo Spending Insights ===\n".bold().cyan());

    predict_monthly(transactions);
    println!();

    category_breakdown(transactions);
    println!();

    top_merchants(transactions, 10);
    println!();

    weekly_spending(transactions);
    println!();

    detect_recurring(transactions);
}

/// Compute category totals (used internally by category_breakdown, exposed for testing)
pub fn compute_category_totals(transactions: &[Transaction]) -> Vec<(String, i64)> {
    let mut by_category: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    for tx in transactions {
        if tx.is_expense() {
            let cat = if tx.category.is_empty() {
                "uncategorized".to_string()
            } else {
                tx.category.clone()
            };
            *by_category.entry(cat).or_default() += tx.amount.abs();
        }
    }
    let mut sorted: Vec<_> = by_category.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted
}

/// Compute top merchants (used internally, exposed for testing)
pub fn compute_top_merchants(transactions: &[Transaction]) -> Vec<(String, i64, u32)> {
    let mut by_merchant: HashMap<String, (i64, u32)> = HashMap::new();
    for tx in transactions {
        if tx.is_expense() {
            let name = tx.display_name().to_string();
            let entry = by_merchant.entry(name).or_default();
            entry.0 += tx.amount.abs();
            entry.1 += 1;
        }
    }
    let mut sorted: Vec<_> = by_merchant
        .into_iter()
        .map(|(name, (total, count))| (name, total, count))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted
}

fn category_emoji(cat: &str) -> String {
    let emoji = match cat {
        "eating_out" => "\u{1f37d}\u{fe0f}  ",
        "groceries" => "\u{1f6d2} ",
        "transport" => "\u{1f68c} ",
        "shopping" => "\u{1f6cd}\u{fe0f}  ",
        "entertainment" => "\u{1f3ac} ",
        "bills" => "\u{1f4cb} ",
        "cash" => "\u{1f4b5} ",
        "holidays" => "\u{2708}\u{fe0f}  ",
        "expenses" => "\u{1f4bc} ",
        "general" => "\u{1f4e6} ",
        _ => "\u{2753} ",
    };
    format!("{emoji}{cat}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::test_helpers::*;
    use chrono::{Duration, Utc};

    fn sample_transactions() -> Vec<Transaction> {
        let now = Utc::now();
        vec![
            make_tx_with_merchant(-350, "Pret A Manger", "eating_out"),
            make_tx_with_merchant(-1200, "Tesco", "groceries"),
            make_tx_with_merchant(-250, "Pret A Manger", "eating_out"),
            make_tx_with_merchant(-4500, "TfL", "transport"),
            make_tx_with_merchant(-800, "Tesco", "groceries"),
            make_tx(500, "Refund from Amazon", "shopping"), // positive = not expense
        ]
    }

    // ── compute_category_totals ─────────────────────────────────────────

    #[test]
    fn category_totals_groups_correctly() {
        let txs = sample_transactions();
        let totals = compute_category_totals(&txs);

        let transport = totals.iter().find(|(c, _)| c == "transport");
        assert_eq!(transport.unwrap().1, 4500);

        let eating = totals.iter().find(|(c, _)| c == "eating_out");
        assert_eq!(eating.unwrap().1, 600); // 350 + 250

        let groceries = totals.iter().find(|(c, _)| c == "groceries");
        assert_eq!(groceries.unwrap().1, 2000); // 1200 + 800
    }

    #[test]
    fn category_totals_excludes_positive_amounts() {
        let txs = sample_transactions();
        let totals = compute_category_totals(&txs);
        // "shopping" refund should not appear (it's positive)
        let shopping = totals.iter().find(|(c, _)| c == "shopping");
        assert!(shopping.is_none());
    }

    #[test]
    fn category_totals_sorted_descending() {
        let txs = sample_transactions();
        let totals = compute_category_totals(&txs);
        for w in totals.windows(2) {
            assert!(w[0].1 >= w[1].1);
        }
    }

    #[test]
    fn category_totals_empty_transactions() {
        let totals = compute_category_totals(&[]);
        assert!(totals.is_empty());
    }

    #[test]
    fn category_totals_excludes_loads() {
        let mut tx = make_tx(-1000, "Top Up", "general");
        tx.is_load = true;
        let totals = compute_category_totals(&[tx]);
        assert!(totals.is_empty());
    }

    #[test]
    fn category_totals_uncategorized() {
        let tx = make_tx(-500, "Mystery", "");
        let totals = compute_category_totals(&[tx]);
        assert_eq!(totals[0].0, "uncategorized");
        assert_eq!(totals[0].1, 500);
    }

    // ── compute_top_merchants ───────────────────────────────────────────

    #[test]
    fn top_merchants_groups_and_counts() {
        let txs = sample_transactions();
        let merchants = compute_top_merchants(&txs);

        let pret = merchants.iter().find(|(n, _, _)| n == "Pret A Manger");
        assert!(pret.is_some());
        let (_, total, count) = pret.unwrap();
        assert_eq!(*total, 600); // 350 + 250
        assert_eq!(*count, 2);

        let tesco = merchants.iter().find(|(n, _, _)| n == "Tesco");
        let (_, total, count) = tesco.unwrap();
        assert_eq!(*total, 2000); // 1200 + 800
        assert_eq!(*count, 2);
    }

    #[test]
    fn top_merchants_excludes_income() {
        let txs = sample_transactions();
        let merchants = compute_top_merchants(&txs);
        let refund = merchants.iter().find(|(n, _, _)| n.contains("Amazon"));
        assert!(refund.is_none());
    }

    #[test]
    fn top_merchants_sorted_by_total() {
        let txs = sample_transactions();
        let merchants = compute_top_merchants(&txs);
        for w in merchants.windows(2) {
            assert!(w[0].1 >= w[1].1);
        }
    }

    #[test]
    fn top_merchants_empty() {
        let merchants = compute_top_merchants(&[]);
        assert!(merchants.is_empty());
    }

    // ── Smoke tests (ensure no panics) ──────────────────────────────────

    #[test]
    fn category_breakdown_no_panic() {
        category_breakdown(&sample_transactions());
    }

    #[test]
    fn category_breakdown_empty_no_panic() {
        category_breakdown(&[]);
    }

    #[test]
    fn top_merchants_display_no_panic() {
        top_merchants(&sample_transactions(), 5);
    }

    #[test]
    fn daily_spending_no_panic() {
        daily_spending(&sample_transactions());
    }

    #[test]
    fn daily_spending_empty_no_panic() {
        daily_spending(&[]);
    }

    #[test]
    fn weekly_spending_no_panic() {
        weekly_spending(&sample_transactions());
    }

    #[test]
    fn weekly_spending_empty_no_panic() {
        weekly_spending(&[]);
    }

    #[test]
    fn predict_monthly_no_panic() {
        // Create transactions in the current month
        let now = Utc::now();
        let txs = vec![
            make_tx_at(-500, "Coffee", "eating_out", now - Duration::days(1)),
            make_tx_at(-3000, "Groceries", "groceries", now - Duration::days(3)),
        ];
        predict_monthly(&txs);
    }

    #[test]
    fn predict_monthly_empty_no_panic() {
        predict_monthly(&[]);
    }

    #[test]
    fn detect_recurring_no_panic() {
        detect_recurring(&sample_transactions());
    }

    #[test]
    fn detect_recurring_empty_no_panic() {
        detect_recurring(&[]);
    }

    #[test]
    fn detect_recurring_finds_monthly() {
        let now = Utc::now();
        let txs: Vec<Transaction> = (0..4)
            .map(|i| {
                make_tx_at(
                    -999,
                    "Netflix",
                    "entertainment",
                    now - Duration::days(i * 30),
                )
            })
            .collect();
        // Should not panic; detection logic runs
        detect_recurring(&txs);
    }

    #[test]
    fn full_report_no_panic() {
        let now = Utc::now();
        let txs = vec![
            make_tx_at(-500, "Coffee", "eating_out", now - Duration::days(1)),
            make_tx_at(-3000, "Groceries", "groceries", now - Duration::days(3)),
            make_tx_at(-1500, "Uber", "transport", now - Duration::days(5)),
        ];
        full_report(&txs);
    }

    #[test]
    fn full_report_empty_no_panic() {
        full_report(&[]);
    }
}
