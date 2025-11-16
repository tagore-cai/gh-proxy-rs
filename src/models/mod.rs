use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// Cache entry structure
#[derive(Clone)]
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub size: usize,
}

// Cache structure with memory limit
#[derive(Clone)]
pub struct AppCache {
    pub cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    pub enabled: bool,
    pub max_capacity: usize,        // Maximum number of entries
    pub max_memory: usize,          // Maximum memory in bytes
    pub time_to_live: u64,
    pub current_memory: Arc<RwLock<usize>>,  // Current memory usage
}

impl AppCache {
    pub fn new(enabled: bool, max_capacity: usize, time_to_live: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            enabled,
            max_capacity,
            max_memory: 100 * 1024 * 1024, // 100MB default limit
            time_to_live,
            current_memory: Arc::new(RwLock::new(0)),
        }
    }

    pub fn with_memory_limit(enabled: bool, max_capacity: usize, max_memory: usize, time_to_live: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            enabled,
            max_capacity,
            max_memory,
            time_to_live,
            current_memory: Arc::new(RwLock::new(0)),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        if !self.enabled {
            return None;
        }

        let cache = self.cache.read().ok()?;
        if let Some(entry) = cache.get(key) {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()?
                .as_secs();
            
            if current_time - entry.timestamp < self.time_to_live {
                Some(entry.data.clone())
            } else {
                // Entry expired
                drop(cache);
                self.remove(key);
                None
            }
        } else {
            None
        }
    }

    pub fn set(&self, key: String, data: Vec<u8>) -> bool {
        if !self.enabled {
            return false;
        }

        let data_size = data.len();
        
        // Check if data is too large
        if data_size > self.max_memory {
            tracing::warn!("Data size {} exceeds maximum memory limit {}, not caching", data_size, self.max_memory);
            return false;
        }

        let mut cache = match self.cache.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        
        // Check memory limit and evict entries if needed
        let mut current_memory = match self.current_memory.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        
        // Evict entries until we have enough memory
        while *current_memory + data_size > self.max_memory && !cache.is_empty() {
            if let Some(key_to_remove) = cache.keys().next().cloned() {
                if let Some(removed_entry) = cache.remove(&key_to_remove) {
                    *current_memory = current_memory.saturating_sub(removed_entry.size);
                    tracing::debug!("Evicted cache entry: {} (size: {})", key_to_remove, removed_entry.size);
                }
            }
        }
        
        // Check capacity limit
        if cache.len() >= self.max_capacity && !cache.contains_key(&key) {
            // Remove oldest entry to make space
            if let Some(key_to_remove) = cache.keys().next().cloned() {
                if let Some(removed_entry) = cache.remove(&key_to_remove) {
                    *current_memory = current_memory.saturating_sub(removed_entry.size);
                    tracing::debug!("Evicted cache entry due to capacity: {} (size: {})", key_to_remove, removed_entry.size);
                }
            }
        }
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();
        
        // Update memory usage
        if let Some(old_entry) = cache.get(&key) {
            *current_memory = current_memory.saturating_sub(old_entry.size);
        }
        *current_memory = current_memory.saturating_add(data_size);
        
        cache.insert(key, CacheEntry { data, timestamp, size: data_size });
        true
    }

    pub fn remove(&self, key: &str) -> bool {
        let mut cache = match self.cache.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        
        if let Some(removed_entry) = cache.remove(key) {
            let mut current_memory = match self.current_memory.write() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            *current_memory = current_memory.saturating_sub(removed_entry.size);
            true
        } else {
            false
        }
    }
    
    pub fn get_memory_usage(&self) -> usize {
        match self.current_memory.read() {
            Ok(guard) => *guard,
            Err(_) => 0,
        }
    }
    
    pub fn get_entry_count(&self) -> usize {
        match self.cache.read() {
            Ok(guard) => guard.len(),
            Err(_) => 0,
        }
    }
}