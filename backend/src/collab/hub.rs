use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::broadcast;

#[derive(Clone)]
pub struct CollabHub {
    inner: Arc<Mutex<HashMap<String, broadcast::Sender<HubMessage>>>>,
}

#[derive(Clone, Debug)]
pub struct HubMessage {
    pub json: String,
    pub except_user_id: Option<String>,
}

impl CollabHub {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn sender(&self, room_id: &str) -> broadcast::Sender<HubMessage> {
        let mut guard = self.inner.lock().unwrap();
        guard
            .entry(room_id.to_string())
            .or_insert_with(|| broadcast::channel(256).0)
            .clone()
    }

    pub fn subscribe(&self, room_id: &str) -> broadcast::Receiver<HubMessage> {
        self.sender(room_id).subscribe()
    }

    pub fn broadcast(&self, room_id: &str, json: String, except_user_id: Option<String>) {
        let _ = self.sender(room_id).send(HubMessage {
            json,
            except_user_id,
        });
    }
}

impl Default for CollabHub {
    fn default() -> Self {
        Self::new()
    }
}
