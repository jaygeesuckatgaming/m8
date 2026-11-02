//! What the mate remembers between games: the Gem system's Cognee server (its HTTP API on
//! :8011), the same memory the stream's Gem has. Entries carry the config's `source`
//! ("halo"), so they read "[halo] Gem: ..." beside the chat's in recall.
//!
//! Writes go out on threads of their own (the server can be slow), recalls block and are
//! called from threads too. With no url (or `enabled = false`) every call does nothing.

use crate::config::MemoryConfig;
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Clone)]
pub struct RemoteMemory {
    agent: ureq::Agent,
    url: Option<String>,
    source: String,
}

impl RemoteMemory {
    pub fn new(cfg: &MemoryConfig) -> Self {
        let url = if cfg.enabled { cfg.url.clone() } else { None };
        let url = url
            .map(|u| u.trim().trim_end_matches('/').to_string())
            .filter(|u| !u.is_empty());
        RemoteMemory {
            agent: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(120))
                .build(),
            url,
            source: cfg.source.clone(),
        }
    }

    pub fn enabled(&self) -> bool {
        self.url.is_some()
    }

    /// Adds a line to the memory, on a thread of its own: the server's writes are serialised
    /// and can take a while, and the game must not wait for them.
    pub fn remember(&self, speaker: &str, text: &str) {
        let Some(url) = self.url.clone() else { return };
        let (agent, source) = (self.agent.clone(), self.source.clone());
        let (speaker, text) = (speaker.to_string(), text.to_string());
        std::thread::spawn(move || {
            let body = json!({"speaker": speaker, "text": text, "source": source});
            if let Err(e) = agent.post(&format!("{url}/remember")).send_json(body) {
                println!("[memory] remember failed: {e}");
            }
        });
    }

    /// What the memory has about `query` (nothing if the server isn't there).
    pub fn recall(&self, query: &str, top_k: usize) -> Vec<String> {
        let Some(url) = self.url.clone() else {
            return Vec::new();
        };
        let body = json!({"query": query, "top_k": top_k});
        match self.agent.post(&format!("{url}/recall")).send_json(body) {
            Ok(response) => response
                .into_json::<Value>()
                .ok()
                .and_then(|value| value["results"].as_array().cloned())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            Err(e) => {
                println!("[memory] recall failed: {e}");
                Vec::new()
            }
        }
    }
}
