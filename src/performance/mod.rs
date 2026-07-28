// src/performance/mod.rs

//! Performance Layer
//!
//! Per Architecture: Monitors and optimizes system performance, including
//! caching, resource management, and execution optimization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU usage percentage
    pub cpu_usage: f32,
    
    /// Memory usage in bytes
    pub memory_usage: u64,
    
    /// Active threads
    pub active_threads: usize,
    
    /// Request latency in milliseconds
    pub avg_latency_ms: f64,
    
    /// Requests per second
    pub requests_per_second: f64,
}

/// Cache entry
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    access_count: u64,
    last_accessed: Instant,
}

/// LRU Cache implementation
pub struct LruCache<K, V> {
    capacity: usize,
    entries: HashMap<K, CacheEntry<V>>,
    access_order: Vec<K>,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: HashMap::new(),
            access_order: Vec::new(),
        }
    }
    
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.access_count += 1;
            entry.last_accessed = Instant::now();
            
            // Move to end of access order (most recently used)
            self.access_order.retain(|k| k != key);
            self.access_order.push(key.clone());
            
            Some(entry.value.clone())
        } else {
            None
        }
    }
    
    pub fn put(&mut self, key: K, value: V) {
        // If key exists, update it
        if self.entries.contains_key(&key) {
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.value = value;
                entry.last_accessed = Instant::now();
            }
            self.access_order.retain(|k| k != &key);
            self.access_order.push(key);
            return;
        }
        
        // Evict if at capacity
        if self.entries.len() >= self.capacity {
            if let Some(oldest) = self.access_order.first().cloned() {
                self.entries.remove(&oldest);
                self.access_order.remove(0);
            }
        }
        
        // Insert new entry
        self.entries.insert(key.clone(), CacheEntry {
            value,
            created_at: Instant::now(),
            access_count: 1,
            last_accessed: Instant::now(),
        });
        self.access_order.push(key);
    }
    
    pub fn remove(&mut self, key: &K) -> bool {
        self.access_order.retain(|k| k != key);
        self.entries.remove(key).is_some()
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
    }
    
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    pub fn stats(&self) -> CacheStats {
        let total_accesses: u64 = self.entries.values().map(|e| e.access_count).sum();
        CacheStats {
            entries: self.entries.len(),
            capacity: self.capacity,
            total_accesses,
            hit_rate: if total_accesses > 0 {
                self.entries.values().filter(|e| e.access_count > 1).count() as f64 / total_accesses as f64
            } else {
                0.0
            },
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub capacity: usize,
    pub total_accesses: u64,
    pub hit_rate: f64,
}

/// Performance optimizer
pub struct PerformanceLayer {
    /// Response cache
    response_cache: LruCache<String, CachedResponse>,
    
    /// Query cache
    query_cache: LruCache<String, QueryResult>,
    
    /// Request statistics
    request_stats: HashMap<String, RequestStats>,
    
    /// Timing statistics
    operation_timings: HashMap<String, Vec<Duration>>,
}

impl PerformanceLayer {
    pub fn new() -> Self {
        Self {
            response_cache: LruCache::new(1000),
            query_cache: LruCache::new(5000),
            request_stats: HashMap::new(),
            operation_timings: HashMap::new(),
        }
    }
    
    // ========================================================================
    // Caching
    // ========================================================================
    
    /// Get cached response
    pub fn get_cached_response(&mut self, key: &str) -> Option<CachedResponse> {
        self.response_cache.get(&key.to_string())
    }
    
    /// Cache a response
    pub fn cache_response(&mut self, key: String, response: CachedResponse) {
        self.response_cache.put(key, response);
    }
    
    /// Get cached query result
    pub fn get_cached_query(&mut self, query: &str) -> Option<QueryResult> {
        self.query_cache.get(&query.to_string())
    }
    
    /// Cache a query result
    pub fn cache_query(&mut self, query: String, result: QueryResult) {
        self.query_cache.put(query, result);
    }
    
    /// Invalidate cache entries matching a pattern
    pub fn invalidate_cache(&mut self, pattern: &str) {
        // For simplicity, clear all caches when invalidation is requested
        // A more sophisticated implementation would support selective invalidation
        self.response_cache.clear();
        self.query_cache.clear();
        tracing::info!("Cache invalidated for pattern: {}", pattern);
    }
    
    // ========================================================================
    // Request Tracking
    // ========================================================================
    
    /// Record a request
    pub fn record_request(&mut self, endpoint: &str) {
        let stats = self.request_stats.entry(endpoint.to_string()).or_insert_with(|| RequestStats::default());
        stats.total_requests += 1;
        stats.last_request = Instant::now();
    }
    
    /// Record request duration
    pub fn record_duration(&mut self, operation: &str, duration: Duration) {
        let timings = self.operation_timings.entry(operation.to_string()).or_insert_with(Vec::new);
        timings.push(duration);
        
        // Keep only last 1000 timings
        if timings.len() > 1000 {
            timings.remove(0);
        }
    }
    
    /// Get average duration for an operation
    pub fn get_avg_duration(&self, operation: &str) -> Option<Duration> {
        self.operation_timings.get(operation).and_then(|timings| {
            if timings.is_empty() {
                None
            } else {
                let sum: Duration = timings.iter().sum();
                Some(sum / timings.len() as u32)
            }
        })
    }
    
    /// Get operation statistics
    pub fn get_operation_stats(&self, operation: &str) -> Option<OperationStats> {
        self.operation_timings.get(operation).map(|timings| {
            if timings.is_empty() {
                return OperationStats::default();
            }
            
            let sum: Duration = timings.iter().sum();
            let avg = sum / timings.len() as u32;
            let min = *timings.iter().min().unwrap();
            let max = *timings.iter().max().unwrap();
            
            OperationStats {
                count: timings.len(),
                avg_duration_ms: avg.as_millis() as f64,
                min_duration_ms: min.as_millis() as f64,
                max_duration_ms: max.as_millis() as f64,
            }
        })
    }
    
    // ========================================================================
    // Performance Analysis
    // ========================================================================
    
    /// Get overall performance report
    pub fn get_performance_report(&self) -> PerformanceReport {
        let mut operation_stats = HashMap::new();
        
        for (op, timings) in &self.operation_timings {
            if !timings.is_empty() {
                let sum: Duration = timings.iter().sum();
                let avg = sum / timings.len() as u32;
                
                operation_stats.insert(op.clone(), OperationStats {
                    count: timings.len(),
                    avg_duration_ms: avg.as_millis() as f64,
                    min_duration_ms: timings.iter().min().map(|d| d.as_millis() as f64).unwrap_or(0.0),
                    max_duration_ms: timings.iter().max().map(|d| d.as_millis() as f64).unwrap_or(0.0),
                });
            }
        }
        
        PerformanceReport {
            response_cache_stats: self.response_cache.stats(),
            query_cache_stats: self.query_cache.stats(),
            endpoint_stats: self.request_stats.len(),
            operation_stats,
        }
    }
    
    /// Suggest optimizations based on current patterns
    pub fn suggest_optimizations(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Check cache hit rates
        let query_hit_rate = self.query_cache.stats().hit_rate;
        if query_hit_rate < 0.3 {
            suggestions.push("Consider increasing query cache size or improving cache key strategy".to_string());
        }
        
        // Check slow operations
        for (op, stats) in self.get_performance_report().operation_stats {
            if stats.avg_duration_ms > 1000.0 {
                suggestions.push(format!("Operation '{}' is slow (avg: {:.1}ms). Consider optimization.", op, stats.avg_duration_ms));
            }
        }
        
        suggestions
    }
    
    /// Reset all statistics
    pub fn reset_stats(&mut self) {
        self.request_stats.clear();
        self.operation_timings.clear();
        self.response_cache.clear();
        self.query_cache.clear();
    }
}

/// Cached response
#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub data: Vec<u8>,
    pub content_type: String,
    pub cached_at: Instant,
    pub expires_at: Option<Instant>,
}

impl CachedResponse {
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            Instant::now() > expires
        } else {
            false
        }
    }
}

/// Query result cache entry
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub result: String,
    pub score: f32,
    pub cached_at: Instant,
}

/// Request statistics
#[derive(Debug, Clone)]
pub struct RequestStats {
    pub total_requests: u64,
    pub last_request: Instant,
}

impl Default for RequestStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            last_request: Instant::now(),
        }
    }
}

/// Operation statistics
#[derive(Debug, Clone)]
pub struct OperationStats {
    pub count: usize,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
}

impl Default for OperationStats {
    fn default() -> Self {
        Self {
            count: 0,
            avg_duration_ms: 0.0,
            min_duration_ms: 0.0,
            max_duration_ms: 0.0,
        }
    }
}

/// Full performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub response_cache_stats: CacheStats,
    pub query_cache_stats: CacheStats,
    pub endpoint_stats: usize,
    pub operation_stats: HashMap<String, OperationStats>,
}

impl Default for PerformanceLayer {
    fn default() -> Self {
        Self::new()
    }
}
