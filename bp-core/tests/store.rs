use bp_core::utils::store::*;
use bytes::Bytes;

#[test]
fn test_cache() {
    let mut cache = Store::default();
    cache.push_back(Bytes::from_static(b"foo"));

    assert_eq!(cache.pull(0), Bytes::from_static(b""));
    assert_eq!(cache.pull(2), Bytes::from_static(b"fo"));
    assert_eq!(cache.pull(2), Bytes::from_static(b"o"));
    assert_eq!(cache.len(), 0);

    cache.clear();
    assert_eq!(cache.len(), 0);

    cache.push_back(Bytes::from_static(b"foo"));
    assert_eq!(cache.pull_all(), Bytes::from_static(b"foo"));
    assert!(cache.is_empty());
}
