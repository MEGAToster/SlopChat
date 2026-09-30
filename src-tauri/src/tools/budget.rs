use rusqlite::params;
use tauri::{AppHandle, Manager};

use crate::store::Store;

pub fn remaining(app: &AppHandle, budget: f64) -> f64 {
    budget - spent(app)
}

pub fn spent(app: &AppHandle) -> f64 {
    let store = app.state::<Store>();
    let conn = store.conn();
    conn.query_row(
        "SELECT COALESCE(cost, 0) FROM search_usage WHERE month = strftime('%Y-%m', 'now', 'localtime')",
        [],
        |row| row.get(0),
    )
    .unwrap_or(0.0)
}

pub fn usage(app: &AppHandle) -> (i64, f64) {
    let store = app.state::<Store>();
    let conn = store.conn();
    conn.query_row(
        "SELECT COALESCE(requests, 0), COALESCE(cost, 0) FROM search_usage WHERE month = strftime('%Y-%m', 'now', 'localtime')",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .unwrap_or((0, 0.0))
}

pub fn record(app: &AppHandle, cost: f64) {
    let store = app.state::<Store>();
    let conn = store.conn();
    let _ = conn.execute(
        "INSERT INTO search_usage (month, requests, cost)
         VALUES (strftime('%Y-%m', 'now', 'localtime'), 1, ?1)
         ON CONFLICT(month) DO UPDATE SET requests = requests + 1, cost = cost + ?1",
        params![cost],
    );
}
