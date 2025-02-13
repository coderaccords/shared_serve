#![allow(dead_code)]
use std::collections::LinkedList;
use std::sync::{RwLock, Arc};
use std::fmt;

#[repr(C)]
pub struct Header {
    pub read_index: RwLock<usize>,
    pub write_index: RwLock<usize>,
}

impl Header {
    pub fn new() -> Self {
        Header {
            read_index: RwLock::new(0),
            write_index: RwLock::new(0),
        }
    }
}

pub const CAPACITY: usize = 10;
pub const SHARED_MEMORY_SIZE: usize = std::mem::size_of::<Header>() + std::mem::size_of::<Request>() * CAPACITY;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    GET = 0,
    INSERT = 1,
    DELETE = 2,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Request {
    pub operation: Operation,
    pub key: [u8; 64],     // Fixed buffer for key
    pub value: [u8; 256], 
}

impl Request {
    pub fn new(operation: Operation, key: &str, value: &str) -> Self {
        let mut key_buffer = [0u8; 64];
        let mut value_buffer = [0u8; 256];
        
        key_buffer[..key.len().min(64)].copy_from_slice(&key.as_bytes()[..key.len().min(64)]);
        value_buffer[..value.len().min(256)].copy_from_slice(&value.as_bytes()[..value.len().min(256)]);
        
        Request {
            operation,
            key: key_buffer,
            value: value_buffer,
        }
    }

    /// Returns the key as a &str, excluding any trailing null bytes.
    pub fn key_str(&self) -> &str {
        Self::bytes_to_str(&self.key)
    }

    /// Returns the value as a &str, excluding any trailing null bytes.
    pub fn value_str(&self) -> &str {
        Self::bytes_to_str(&self.value)
    }

    /// Helper function to convert &[u8; N] to &str by finding the first \0
    fn bytes_to_str<const N: usize>(bytes: &[u8; N]) -> &str {
        if let Some(pos) = bytes.iter().position(|&c| c == 0) {
            // Safe to unwrap because we're slicing at a valid UTF-8 boundary
            std::str::from_utf8(&bytes[..pos]).unwrap_or("<invalid utf8>")
        } else {
            // No null bytes found, attempt to convert the entire array
            std::str::from_utf8(bytes).unwrap_or("<invalid utf8>")
        }
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Operation: {:?}, Key: {}, Value: {}",
            self.operation,
            self.key_str(),
            self.value_str()
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct HashCell {
    key: String,
    value: String,
}

pub struct HashTable {
    buckets: Vec<Arc<RwLock<LinkedList<HashCell>>>>,
    size: usize,
}

impl HashTable {
    pub fn new(size: usize) -> Self {
        let mut buckets = Vec::with_capacity(size);
        for _ in 0..size {
            buckets.push(Arc::new(RwLock::new(LinkedList::new())));
        }
        HashTable {
            buckets,
            size,
        }
    }

    fn hash(&self, key: &str) -> usize {
        let mut hash = 0;
        for c in key.chars() {
            hash = hash * 31 + c as usize;
        }
        hash % self.size
    }

    pub fn get_bucket(&self, key: &str) -> usize {
        self.hash(key)
    }

    pub fn insert(&self, key: &str, value: &str) {
        let index = self.get_bucket(&key);
        let mut bucket = self.buckets[index].write().unwrap();
        
        for cell in bucket.iter_mut() {
            if cell.key == key {
                cell.value = value.to_string();
                return;
            }
        }
        bucket.push_back(HashCell { key: key.to_string(), value: value.to_string() });
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let index = self.get_bucket(&key);
        // Get a read lock on the bucket
        let bucket = self.buckets[index].read().unwrap();
        for cell in bucket.iter() {
            if cell.key == key {
                let value = cell.value.clone();
                return Some(value);
            }
        }

        None
    }

    pub fn delete(&self, key: &str)-> bool {
        let index = self.get_bucket(&key);
        // get position of the cell
        let mut bucket = self.buckets[index].write().unwrap();
        for (position, cell) in bucket.iter().enumerate() {
            println!("cell: {:?}", cell);
            if cell.key == key {
                // As remove is not stable and is O(n), 
                // instead the list is split at the position 
                // and the first element of the tail is popped.
                // This results in identical complexity.

                let mut tail = bucket.split_off(position);
                tail.pop_front();
                bucket.append(&mut tail);
                return true;
            }
        }
        false
    }

}

// Unit tests for the hash table
#[test]
fn test_hash_table() {
    let mut hash_table = HashTable::new(10);
    hash_table.insert("key1", "value1");
    hash_table.insert("key2", "value2");
    hash_table.insert("key1", "value3");
    assert_eq!(hash_table.get("key1").unwrap().as_str(), "value3");
    assert_eq!(hash_table.get("key2").unwrap().as_str(), "value2");
}

#[test]
fn test_hash_table_delete() {
    let mut hash_table = HashTable::new(10);
    hash_table.insert("key1", "value1");
    hash_table.delete("key1");
    assert_eq!(hash_table.get("key1"), None);
}

#[test]
fn test_hash_table_insert() {
    let mut hash_table = HashTable::new(10);
    hash_table.insert("key1", "value1");
    assert_eq!(hash_table.get("key1").unwrap().as_str(), "value1");
}

