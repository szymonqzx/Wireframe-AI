//! Optimized JSON serialization utilities with zero-copy operations.
//!
//! Provides efficient serialization strategies for common message types
//! to reduce CPU overhead and improve performance with buffer reuse and
//! zero-copy deserialization where possible.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Serialize a value to JSON with optimized settings.
/// Uses compact serialization to reduce size and improve speed.
#[inline]
pub fn to_compact_json<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(value)
}

/// Serialize a value to JSON with pretty printing for debugging.
#[inline]
pub fn to_pretty_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}

/// Deserialize JSON bytes with efficient error handling.
/// Zero-copy when possible for borrowed data.
#[inline]
pub fn from_json_slice<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, serde_json::Error> {
    serde_json::from_slice(bytes)
}

/// Deserialize JSON from a string with efficient error handling.
#[inline]
pub fn from_json_str<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(s)
}

/// Serialize a value directly to a writer for streaming.
/// More memory-efficient for large payloads.
#[inline]
pub fn serialize_to_writer<W: Write, T: Serialize>(
    writer: &mut W,
    value: &T,
) -> Result<(), serde_json::Error> {
    serde_json::to_writer(writer, value)
}

/// Deserialize JSON from a reader for streaming.
/// More memory-efficient for large payloads.
#[inline]
pub fn deserialize_from_reader<R: std::io::Read, T: for<'de> Deserialize<'de>>(
    reader: R,
) -> Result<T, serde_json::Error> {
    serde_json::from_reader(reader)
}

/// Optimized JSON serializer that reuses buffers.
/// Reduces allocations for repeated serialization operations.
pub struct JsonSerializer {
    buffer: Vec<u8>,
}

impl JsonSerializer {
    /// Create a new JSON serializer with reusable buffer.
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
        }
    }

    /// Create a new JSON serializer with custom initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    /// Serialize a value and return the bytes, reusing the internal buffer.
    #[inline]
    pub fn serialize<T: Serialize>(&mut self, value: &T) -> Result<&[u8], serde_json::Error> {
        self.buffer.clear();
        serde_json::to_writer(&mut self.buffer, value)?;
        Ok(&self.buffer)
    }

    /// Serialize a value and return the bytes as a Vec (owned).
    /// Useful when the buffer needs to be kept.
    #[inline]
    pub fn serialize_to_vec<T: Serialize>(
        &mut self,
        value: &T,
    ) -> Result<Vec<u8>, serde_json::Error> {
        self.buffer.clear();
        serde_json::to_writer(&mut self.buffer, value)?;
        Ok(self.buffer.clone())
    }

    /// Get the current buffer capacity.
    #[inline]
    pub fn buffer_capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Get the current buffer length.
    #[inline]
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Clear the buffer and reduce capacity if needed.
    #[inline]
    pub fn shrink(&mut self) {
        if self.buffer.capacity() > 4096 {
            self.buffer = Vec::with_capacity(4096);
        } else {
            self.buffer.clear();
        }
    }

    /// Reserve additional capacity in the buffer.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional);
    }
}

impl Default for JsonSerializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimized JSON deserializer that reuses scratch space.
/// Reduces allocations for repeated deserialization operations.
pub struct JsonDeserializer {
    scratch: Vec<u8>,
}

impl JsonDeserializer {
    /// Create a new JSON deserializer.
    #[inline]
    pub fn new() -> Self {
        Self {
            scratch: Vec::with_capacity(4096),
        }
    }

    /// Create a new JSON deserializer with custom scratch capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            scratch: Vec::with_capacity(capacity),
        }
    }

    /// Deserialize from bytes with zero-copy when possible.
    #[inline]
    pub fn deserialize<'a, T: Deserialize<'a>>(
        &self,
        bytes: &'a [u8],
    ) -> Result<T, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Deserialize from string.
    #[inline]
    pub fn deserialize_str<'a, T: Deserialize<'a>>(
        &self,
        s: &'a str,
    ) -> Result<T, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Deserialize from bytes using scratch space for temporary storage.
    /// Useful for deserializing into types that require owned data.
    pub fn deserialize_owned<T: for<'de> Deserialize<'de>>(
        &mut self,
        bytes: &[u8],
    ) -> Result<T, serde_json::Error> {
        self.scratch.clear();
        self.scratch.extend_from_slice(bytes);
        serde_json::from_slice(&self.scratch)
    }

    /// Get the scratch buffer capacity.
    #[inline]
    pub fn scratch_capacity(&self) -> usize {
        self.scratch.capacity()
    }

    /// Clear the scratch buffer.
    #[inline]
    pub fn clear_scratch(&mut self) {
        self.scratch.clear();
    }
}

impl Default for JsonDeserializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Zero-copy view into JSON data for efficient deserialization.
pub struct JsonView<'a> {
    data: &'a [u8],
}

impl<'a> JsonView<'a> {
    /// Create a new JSON view from bytes.
    #[inline]
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Deserialize the JSON data with zero-copy.
    #[inline]
    pub fn deserialize<T: Deserialize<'a>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(self.data)
    }

    /// Get the underlying data bytes.
    #[inline]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.data
    }

    /// Get the data as a string slice (if valid UTF-8).
    #[inline]
    pub fn as_str(&self) -> Result<&'a str, std::str::Utf8Error> {
        std::str::from_utf8(self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_compact_json() {
        let value = json!({"key": "value"});
        let bytes = to_compact_json(&value).unwrap();
        assert!(bytes.len() > 0);
    }

    #[test]
    fn test_pretty_json() {
        let value = json!({"key": "value"});
        let pretty = to_pretty_json(&value).unwrap();
        assert!(pretty.contains('\n'));
    }

    #[test]
    fn test_from_json_slice() {
        let value = json!({"key": "value"});
        let bytes = to_compact_json(&value).unwrap();
        let parsed: serde_json::Value = from_json_slice(&bytes).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_from_json_str() {
        let json_str = r#"{"key": "value"}"#;
        let parsed: serde_json::Value = from_json_str(json_str).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_json_serializer_reuse() {
        let mut serializer = JsonSerializer::new();

        let value1 = json!({"key1": "value1"});
        let len1 = serializer.serialize(&value1).unwrap().len();

        let value2 = json!({"key2": "value2"});
        let len2 = serializer.serialize(&value2).unwrap().len();

        assert!(len1 > 0);
        assert!(len2 > 0);
        // Buffer is reused
        assert!(serializer.buffer_capacity() >= len1.max(len2));
    }

    #[test]
    fn test_json_serializer_with_capacity() {
        let serializer = JsonSerializer::with_capacity(8192);
        assert!(serializer.buffer_capacity() >= 8192);
    }

    #[test]
    fn test_json_serializer_to_vec() {
        let mut serializer = JsonSerializer::new();
        let value = json!({"key": "value"});
        let vec = serializer.serialize_to_vec(&value).unwrap();
        assert!(vec.len() > 0);
    }

    #[test]
    fn test_json_deserializer() {
        let deserializer = JsonDeserializer::new();

        let bytes = br#"{"key": "value"}"#;
        let parsed: serde_json::Value = deserializer.deserialize(bytes).unwrap();
        assert_eq!(parsed["key"], "value");

        let json_str = r#"{"key": "value"}"#;
        let parsed2: serde_json::Value = deserializer.deserialize_str(json_str).unwrap();
        assert_eq!(parsed2["key"], "value");
    }

    #[test]
    fn test_json_deserializer_owned() {
        let mut deserializer = JsonDeserializer::new();
        let bytes = br#"{"key": "value"}"#;
        let parsed: serde_json::Value = deserializer.deserialize_owned(bytes).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn test_json_view() {
        let bytes = br#"{"key": "value"}"#;
        let view = JsonView::new(bytes);
        let parsed: serde_json::Value = view.deserialize().unwrap();
        assert_eq!(parsed["key"], "value");
        assert_eq!(view.as_bytes(), bytes);
    }

    #[test]
    fn test_json_view_str() {
        let bytes = br#"{"key": "value"}"#;
        let view = JsonView::new(bytes);
        let str = view.as_str().unwrap();
        assert_eq!(str, r#"{"key": "value"}"#);
    }
}

/// Serialization cache for frequently serialized data.
/// Reduces CPU overhead for repeated serialization of the same data.
pub struct SerializationCache<T>
where
    T: Serialize + Eq + Hash,
{
    cache: HashMap<String, (Vec<u8>, Instant)>,
    max_size: usize,
    ttl: Duration,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> SerializationCache<T>
where
    T: Serialize + Eq + Hash,
{
    /// Create a new serialization cache.
    #[inline]
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: HashMap::with_capacity(max_size),
            max_size,
            ttl,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get cached serialization or serialize and cache.
    #[inline]
    pub fn get_or_serialize(
        &mut self,
        key: String,
        value: &T,
    ) -> Result<Vec<u8>, serde_json::Error> {
        // Check cache
        if let Some((cached, timestamp)) = self.cache.get(&key) {
            if timestamp.elapsed() < self.ttl {
                return Ok(cached.clone());
            }
        }

        // Serialize and cache
        let serialized = serde_json::to_vec(value)?;

        // Evict if at capacity
        if self.cache.len() >= self.max_size {
            self.evict_expired();
            if self.cache.len() >= self.max_size {
                // Remove oldest entry
                if let Some(oldest_key) = self
                    .cache
                    .iter()
                    .min_by_key(|(_, (_, time))| time)
                    .map(|(k, _)| k.clone())
                {
                    self.cache.remove(&oldest_key);
                }
            }
        }

        self.cache.insert(key, (serialized.clone(), Instant::now()));
        Ok(serialized)
    }

    /// Evict expired entries from the cache.
    #[inline]
    fn evict_expired(&mut self) {
        let now = Instant::now();
        self.cache
            .retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.ttl);
    }

    /// Clear the cache.
    #[inline]
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics.
    #[inline]
    pub fn stats(&self) -> CacheStats {
        let now = Instant::now();
        let expired_count = self
            .cache
            .values()
            .filter(|(_, timestamp)| now.duration_since(*timestamp) >= self.ttl)
            .count();

        CacheStats {
            size: self.cache.len(),
            max_size: self.max_size,
            expired_count,
            hit_rate: 0.0, // Would need to track hits/misses
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub expired_count: usize,
    pub hit_rate: f64,
}

/// Batch serialization for efficient processing of multiple values.
pub struct BatchSerializer {
    serializer: JsonSerializer,
}

impl BatchSerializer {
    /// Create a new batch serializer.
    #[inline]
    pub fn new() -> Self {
        Self {
            serializer: JsonSerializer::new(),
        }
    }

    /// Create a new batch serializer with custom capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            serializer: JsonSerializer::with_capacity(capacity),
        }
    }

    /// Serialize multiple values efficiently.
    #[inline]
    pub fn serialize_batch<T: Serialize>(
        &mut self,
        values: &[T],
    ) -> Result<Vec<Vec<u8>>, serde_json::Error> {
        let mut results = Vec::with_capacity(values.len());
        for value in values {
            let bytes = self.serializer.serialize_to_vec(value)?;
            results.push(bytes);
        }
        Ok(results)
    }

    /// Serialize multiple values with keys (for caching).
    #[inline]
    pub fn serialize_batch_with_keys<I, T>(
        &mut self,
        items: I,
    ) -> Result<Vec<(String, Vec<u8>)>, serde_json::Error>
    where
        I: IntoIterator<Item = (String, T)>,
        T: Serialize,
    {
        let iterator = items.into_iter();
        let mut results = Vec::with_capacity(iterator.size_hint().0);
        for (key, value) in iterator {
            let bytes = self.serializer.serialize_to_vec(&value)?;
            results.push((key, bytes));
        }
        Ok(results)
    }

    /// Get the underlying serializer.
    #[inline]
    pub fn serializer(&mut self) -> &mut JsonSerializer {
        &mut self.serializer
    }
}

impl Default for BatchSerializer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Optimized string interning for reducing memory allocations.
pub struct StringInterner {
    strings: HashMap<String, Arc<String>>,
    max_size: usize,
}

impl StringInterner {
    /// Create a new string interner.
    #[inline]
    pub fn new(max_size: usize) -> Self {
        Self {
            strings: HashMap::with_capacity(max_size),
            max_size,
        }
    }

    /// Intern a string, returning an Arc to the shared string.
    #[inline]
    pub fn intern(&mut self, s: &str) -> Arc<String> {
        if let Some(interned) = self.strings.get(s) {
            return Arc::clone(interned);
        }

        // Evict if at capacity
        if self.strings.len() >= self.max_size {
            // Remove a random entry (simple eviction strategy)
            if let Some(key) = self.strings.keys().next().cloned() {
                self.strings.remove(&key);
            }
        }

        let arc_string = Arc::new(s.to_string());
        self.strings.insert(s.to_string(), Arc::clone(&arc_string));
        arc_string
    }

    /// Get the number of interned strings.
    #[inline]
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Clear the interner.
    #[inline]
    pub fn clear(&mut self) {
        self.strings.clear();
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_serialization_cache() {
        let mut cache = SerializationCache::new(10, Duration::from_secs(60));
        let value = "test_value";

        let result1 = cache.get_or_serialize("key1".to_string(), &value).unwrap();
        let result2 = cache.get_or_serialize("key1".to_string(), &value).unwrap();

        assert_eq!(result1, result2);
        assert!(cache.stats().size > 0);
    }

    #[test]
    fn test_batch_serializer() {
        let mut serializer = BatchSerializer::new();
        let values = vec![1, 2, 3, 4, 5];

        let results = serializer.serialize_batch(&values).unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_string_interner() {
        let mut interner = StringInterner::new(100);
        let s1 = interner.intern("hello");
        let s2 = interner.intern("hello");
        let s3 = interner.intern("world");

        // Same string should return same Arc
        assert!(Arc::ptr_eq(&s1, &s2));
        // Different string should return different Arc
        assert!(!Arc::ptr_eq(&s1, &s3));
        assert_eq!(interner.len(), 2);
    }
}
