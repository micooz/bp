use bp_core::acl::*;

#[test]
fn test_load_from_file() {
    let acl = AccessControlList::default();
    assert!(acl.load_from_file("tests/fixtures/acl.txt".into()).is_ok());
}

#[test]
fn test_save_to_file() {
    let acl = AccessControlList::default();
    acl.insert("example.com", DomainRule::NotExtractMatch);

    assert!(acl.save_to_file("/tmp/acl.txt".into()).is_ok());
}

#[test]
fn test_is_host_hit() {
    let acl = AccessControlList::default();
    acl.load_from_file("tests/fixtures/acl.txt".into()).unwrap();

    assert!(acl.is_host_hit("www.example.com"));
}

#[test]
fn test_insert() {
    let acl = AccessControlList::default();
    acl.insert("example.com", DomainRule::NotExtractMatch);

    assert!(!acl.is_host_hit("example.com"));
}

// #[test]
// fn test_watch() {
//     let acl = AccessControlList::default();
//     let path = "tests/fixtures/acl.txt".to_string();

//     acl.load_from_file(path.clone()).unwrap();

//     tokio::runtime::Builder::new_multi_thread()
//         .build()
//         .unwrap()
//         .block_on(async {
//             let handle = tokio::spawn(async move {
//                 acl.watch(path).unwrap();
//             });
//             handle.await.unwrap();
//         });
// }
