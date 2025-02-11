use std::collections::LinkedList;

#[derive(Debug, PartialEq, Eq, Clone)]
struct HashCell {
    key: String,
    value: String,
}

struct HashTable {
    buckets: Vec<LinkedList<HashCell>>,
    size: usize,
}

impl HashTable {
    fn new(size: usize) -> Self {
        HashTable {
            buckets: vec![LinkedList::new(); size],
            size: size,
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
        self.buckets[index].push_back(HashCell { key: key.to_string(), value: value.to_string() });
    }

    fn get(&self, key: &str) -> Option<&String> {
        let index = self.hash(&key);
        for cell in self.buckets[index].iter() {
            if cell.key == key {
                return Some(&cell.value);
            }
        }
        None
    }

    fn delete(&mut self, key: &str) {
        let index = self.hash(&key);
        // get position of the cell
        let mut bucket = &mut self.buckets[index];
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
