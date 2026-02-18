// use serde::Serialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use once_cell::sync::OnceCell;
use std::env;
// use std::collections::HashMap;

const MAX_LOGS: usize = 100;

pub struct Telemetry {
    // Mutex<VecDeque> is safe for async access across threads.
    logs: Arc<Mutex<VecDeque<Value>>>,
}

impl Telemetry {
    pub fn new() -> Self {
        println!("✅ [Telemetry] Internal Audit Module Active (Capacity: {}).", MAX_LOGS);
        Telemetry {
            logs: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOGS))),
        }
    }

    pub fn ship(&self, log_entry: Value) {
        if let Ok(mut logs) = self.logs.lock() {
            if logs.len() >= MAX_LOGS {
                logs.pop_front(); // Remove oldest
            }
            logs.push_back(log_entry.clone());
        }
    }

    pub fn get_recent_logs(&self) -> Vec<Value> {
        if let Ok(logs) = self.logs.lock() {
            // Return a clone of the vector (Snapshot)
            return logs.iter().cloned().collect();
        }
        vec![]
    }
}

// Global Singleton
pub static TELEMETRY: OnceCell<Telemetry> = OnceCell::new();

pub fn init() {
    TELEMETRY.set(Telemetry::new()).ok();
}

pub fn ship(log: Value) {
    if let Some(t) = TELEMETRY.get() {
        t.ship(log);
    }
}

pub fn get_logs() -> Vec<Value> {
    if let Some(t) = TELEMETRY.get() {
        return t.get_recent_logs();
    }
    vec![]
}
