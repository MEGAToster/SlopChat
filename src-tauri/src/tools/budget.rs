use rusqlite::params;
use tauri::{AppHandle, Manager};

use crate::config::Limit;
use crate::store::Store;

const HEADROOM: f64 = 0.02;

pub fn record(app: &AppHandle, provider: &str, units: u32, cost: f64) {
    let store = app.state::<Store>();
    let conn = store.conn();
    let _ = conn.execute(
        "INSERT INTO search_usage (month, provider, requests, units, cost)
         VALUES (strftime('%Y-%m', 'now', 'localtime'), ?1, 1, ?2, ?3)
         ON CONFLICT(month, provider) DO UPDATE SET
           requests = requests + 1, units = units + ?2, cost = cost + ?3",
        params![provider, units, cost],
    );
}

pub fn monthly(app: &AppHandle, provider: &str) -> (i64, i64, f64) {
    let store = app.state::<Store>();
    let conn = store.conn();
    conn.query_row(
        "SELECT COALESCE(requests, 0), COALESCE(units, 0), COALESCE(cost, 0)
         FROM search_usage
         WHERE month = strftime('%Y-%m', 'now', 'localtime') AND provider = ?1",
        params![provider],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .unwrap_or((0, 0, 0.0))
}

pub fn total_units(app: &AppHandle, provider: &str) -> i64 {
    let store = app.state::<Store>();
    let conn = store.conn();
    conn.query_row(
        "SELECT COALESCE(SUM(units), 0) FROM search_usage WHERE provider = ?1",
        params![provider],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

pub fn allowed(app: &AppHandle, provider: &str, limit: &Limit) -> bool {
    let (_, units, cost) = monthly(app, provider);
    within(limit, units, cost, total_units(app, provider))
}

fn within(limit: &Limit, units: i64, cost: f64, total: i64) -> bool {
    if let Some(max) = limit.monthly {
        if units >= max as i64 {
            return false;
        }
    }
    if let Some(usd) = limit.monthly_usd {
        if cost >= usd - HEADROOM {
            return false;
        }
    }
    if let Some(max) = limit.total {
        if total >= max as i64 {
            return false;
        }
    }
    true
}

pub fn mark_exhausted(app: &AppHandle, provider: &str) {
    let store = app.state::<Store>();
    let conn = store.conn();
    let _ = conn.execute(
        "INSERT INTO search_exhausted (provider, month)
         VALUES (?1, strftime('%Y-%m', 'now', 'localtime'))
         ON CONFLICT(provider) DO UPDATE SET month = excluded.month",
        params![provider],
    );
}

pub fn is_exhausted(app: &AppHandle, provider: &str) -> bool {
    let store = app.state::<Store>();
    let conn = store.conn();
    conn.query_row(
        "SELECT 1 FROM search_exhausted
         WHERE provider = ?1 AND month = strftime('%Y-%m', 'now', 'localtime')",
        params![provider],
        |_| Ok(()),
    )
    .is_ok()
}

// A new key means a fresh account, so drop the exhausted mark and its usage.
pub fn reset(app: &AppHandle, provider: &str) {
    let store = app.state::<Store>();
    let conn = store.conn();
    let _ = conn.execute(
        "DELETE FROM search_exhausted WHERE provider = ?1",
        params![provider],
    );
    let _ = conn.execute("DELETE FROM search_usage WHERE provider = ?1", params![provider]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limit(monthly: Option<u32>, monthly_usd: Option<f64>, total: Option<u32>) -> Limit {
        Limit {
            monthly,
            monthly_usd,
            total,
        }
    }

    #[test]
    fn allowed_respects_every_limit_field() {
        assert!(within(&limit(Some(100), None, None), 99, 0.0, 0));
        assert!(!within(&limit(Some(100), None, None), 100, 0.0, 0));

        assert!(within(&limit(None, Some(10.0), None), 0, 9.97, 0));
        assert!(!within(&limit(None, Some(10.0), None), 0, 9.99, 0));

        assert!(within(&limit(None, None, Some(2500)), 0, 0.0, 2499));
        assert!(!within(&limit(None, None, Some(2500)), 0, 0.0, 2500));
    }
}
