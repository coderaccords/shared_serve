use std::collections::LinkedList;
use std::sync::RwLock;

enum Operation {
    GET,
    INSERT,
    DELETE,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct HashCell {
    key: String,
    value: String,
}

struct HashTable {
    buckets: Vec<RwLock<LinkedList<HashCell>>>,
    size: usize,
}

impl HashTable {
    fn new(size: usize) -> Self {
        HashTable {
            buckets: (0..size).map(|_| RwLock::new(LinkedList::new())).collect(),
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

    fn insert(&mut self, key: &str, value: &str) {
        let index = self.hash(&key);
        let mut bucket = self.buckets[index].write().unwrap();
        bucket.push_back(HashCell { key: key.to_string(), value: value.to_string() });
    }

    fn get(&self, key: &str) -> Option<String> {
        let index = self.hash(&key);
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

    fn delete(&mut self, key: &str) {
        let index = self.hash(&key);
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
                return;
            }
        }
    }

}

// Unit tests for the hash table
#[test]
fn test_hash_table() {
    let mut hash_table = HashTable::new(10);
    hash_table.insert("key1", "value1");
    hash_table.insert("key2", "value2");
    assert_eq!(hash_table.get("key1").unwrap().as_str(), "value1");
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