//! Minimal in-memory TTL cache (mirror the Python backend's cachetools.TTLCache
//! usage: stats 600s, tags 300s, CivitAI 24h, Gelbooru 5min). Values are cloned
//! out; keep them cheap-to-clone or wrap in Arc if they grow.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct TtlCache<V> {
    default_ttl: Duration,
    max_entries: usize,
    /// key → (expiry, value)
    map: Mutex<HashMap<String, (Instant, V)>>,
}

impl<V: Clone> TtlCache<V> {
    pub fn new(default_ttl: Duration, max_entries: usize) -> Self {
        Self {
            default_ttl,
            max_entries,
            map: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<V> {
        let mut map = self.map.lock().unwrap();
        match map.get(key) {
            Some((expiry, v)) if *expiry > Instant::now() => Some(v.clone()),
            Some(_) => {
                map.remove(key);
                None
            }
            None => None,
        }
    }

    pub fn insert(&self, key: impl Into<String>, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl);
    }

    /// Insert with a per-entry TTL (used to cache negative lookups briefly).
    pub fn insert_with_ttl(&self, key: impl Into<String>, value: V, ttl: Duration) {
        let mut map = self.map.lock().unwrap();
        let now = Instant::now();
        if map.len() >= self.max_entries {
            map.retain(|_, (expiry, _)| *expiry > now);
        }
        if map.len() >= self.max_entries {
            // Still full: evict the entry closest to expiry (approximates
            // cachetools' TTL-ordered eviction without an ordered structure).
            if let Some(k) = map
                .iter()
                .min_by_key(|(_, (expiry, _))| *expiry)
                .map(|(k, _)| k.clone())
            {
                map.remove(&k);
            }
        }
        map.insert(key.into(), (now + ttl, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_inserted_until_expiry() {
        let cache = TtlCache::new(Duration::from_secs(60), 10);
        cache.insert("a", 1);
        assert_eq!(cache.get("a"), Some(1));
        assert_eq!(cache.get("missing"), None);
    }

    #[test]
    fn expired_entries_are_dropped() {
        let cache = TtlCache::new(Duration::from_secs(60), 10);
        cache.insert_with_ttl("a", 1, Duration::ZERO);
        assert_eq!(cache.get("a"), None);
    }

    #[test]
    fn eviction_keeps_capacity_bounded() {
        let cache = TtlCache::new(Duration::from_secs(60), 2);
        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.insert("c", 3);
        let present = ["a", "b", "c"]
            .iter()
            .filter(|k| cache.get(k).is_some())
            .count();
        assert_eq!(present, 2);
        assert_eq!(cache.get("c"), Some(3));
    }
}
