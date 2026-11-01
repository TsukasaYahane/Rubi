//! A small bounded map that forgets its oldest entries, for caches that must not grow for the
//! whole life of the process.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::Mutex;

pub(crate) struct Recent<K, V> {
    cap: usize,
    inner: Mutex<Inner<K, V>>,
}

struct Inner<K, V> {
    map: HashMap<K, V>,
    /// Keys oldest first; an update keeps the key's original place.
    order: VecDeque<K>,
}

impl<K: Eq + Hash + Clone, V: Clone> Recent<K, V> {
    pub(crate) fn new(cap: usize) -> Self {
        Self { cap: cap.max(1), inner: Mutex::new(Inner { map: HashMap::new(), order: VecDeque::new() }) }
    }

    pub(crate) fn put(&self, key: K, value: V) {
        let mut g = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if g.map.insert(key.clone(), value).is_none() {
            g.order.push_back(key);
        }
        while g.map.len() > self.cap {
            let Some(old) = g.order.pop_front() else { break };
            g.map.remove(&old);
        }
    }

    /// Forgets everything.
    pub(crate) fn clear(&self) {
        let mut g = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        g.map = HashMap::new();
        g.order = VecDeque::new();
    }

    pub(crate) fn get(&self, key: &K) -> Option<V> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).map.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forgets_the_oldest_beyond_capacity() {
        let r = Recent::new(2);
        r.put(1, "a");
        r.put(2, "b");
        r.put(1, "a2");
        r.put(3, "c");
        assert_eq!(r.get(&1), None, "the oldest key goes first, even after an update");
        assert_eq!(r.get(&2), Some("b"));
        assert_eq!(r.get(&3), Some("c"));
        r.clear();
        assert_eq!(r.get(&3), None);
    }
}
