use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Diff<K, T> {
    Add(K, T),
    Remove(K),
    Update(K, T),
}

pub fn hashmap_diff<K: Hash + Eq + Clone, T: PartialEq + Clone>(
    old: &HashMap<K, T>,
    new: &HashMap<K, T>,
) -> Vec<Diff<K, T>> {
    let mut diff = vec![];
    for (k, v) in old.iter() {
        if let Some(v2) = new.get(k) {
            if v != v2 {
                diff.push(Diff::Update(k.clone(), v2.clone()));
            }
        } else {
            diff.push(Diff::Remove(k.clone()));
        }
    }
    for (k, v) in new.iter() {
        if !old.contains_key(k) {
            diff.push(Diff::Add(k.clone(), v.clone()));
        }
    }
    diff
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::diffing::{hashmap_diff, Diff};

    #[test]
    fn test_hashmaps() {
        let mut a = HashMap::new();
        a.insert("a", 1);
        a.insert("b", 2);
        a.insert("c", 3);
        let mut b = a.clone();
        b.insert("b", 4);
        b.remove("c");
        b.insert("d", 5);
        let diff = hashmap_diff(&a, &b);
        assert_eq!(3, diff.len());
        assert!(diff
            .iter()
            .cloned()
            .find(|val| *val == Diff::Add(&"d", 5))
            .is_some());
        assert!(diff
            .iter()
            .cloned()
            .find(|val| *val == Diff::Update(&"b", 4))
            .is_some());
        assert!(diff
            .iter()
            .cloned()
            .find(|val| *val == Diff::Remove(&"c"))
            .is_some());
    }
}
