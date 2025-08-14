use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use super::{GCounter, NodeContext, ValueStore, SharedStore};
use tokio::sync::mpsc;

fn create_test_value_store() -> ValueStore {
    let store: SharedStore = Arc::new(RwLock::new(HashSet::new()));
    let (tx, _rx) = mpsc::channel(100);
    ValueStore::new(store, tx)
}

fn create_test_g_counter() -> GCounter {
    let node_id = Arc::new(RwLock::new(Some("n1".to_string())));
    let context = NodeContext {
        broadcast: Box::new(|_| {}),
        gossip: Box::new(|_| ()),
        next_id: Box::new(|| 1),
        online_neighbors: Box::new(|| vec![]),
        offline_neighbors: Box::new(|| vec![]),
        get_node_id: Box::new(move || node_id.read().unwrap().as_ref().unwrap().clone()),
    };
    GCounter::new(context)
}

#[test]
fn test_update_store_data_new_value() {
    let mut value_store = create_test_value_store();
    let result = value_store.update_store_data(123);
    assert!(result);
    assert!(value_store.contains_value(123));
}

#[test]
fn test_update_store_data_duplicate_value() {
    let mut value_store = create_test_value_store();
    value_store.update_store_data(123);
    let result = value_store.update_store_data(123);
    assert!(!result);
}

#[test]
fn test_values() {
    let mut value_store = create_test_value_store();
    value_store.update_store_data(1);
    value_store.update_store_data(2);
    value_store.update_store_data(3);
    let mut values = value_store.values();
    values.sort();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn test_contains_value() {
    let mut value_store = create_test_value_store();
    value_store.update_store_data(456);
    assert!(value_store.contains_value(456));
    assert!(!value_store.contains_value(789));
}

#[test]
fn test_g_counter_local_sum() {
    let mut g_counter = create_test_g_counter();
    let mut values = HashMap::new();
    values.insert("n1".to_string(), 10);
    values.insert("n2".to_string(), 20);
    g_counter.update_counter(values);
    assert_eq!(g_counter.local_sum(), 30);
}

#[test]
fn test_g_counter_local_value() {
    let mut g_counter = create_test_g_counter();
    assert_eq!(g_counter.local_value(), 0);
    let mut values = HashMap::new();
    values.insert("n1".to_string(), 10);
    g_counter.update_counter(values);
    assert_eq!(g_counter.local_value(), 10);
}

#[test]
fn test_g_counter_update_counter() {
    let mut g_counter = create_test_g_counter();
    let mut values = HashMap::new();
    values.insert("n1".to_string(), 10);
    values.insert("n2".to_string(), 20);
    g_counter.update_counter(values);
    assert_eq!(g_counter.local_sum(), 30);
}

#[test]
fn test_g_counter_update_counter_with_existing_values() {
    let mut g_counter = create_test_g_counter();
    let mut values = HashMap::new();
    values.insert("n1".to_string(), 10);
    g_counter.update_counter(values);
    let mut new_values = HashMap::new();
    new_values.insert("n1".to_string(), 5); // This should be ignored
    new_values.insert("n2".to_string(), 20);
    g_counter.update_counter(new_values);
    assert_eq!(g_counter.local_sum(), 30);
}
