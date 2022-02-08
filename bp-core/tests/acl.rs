use std::fs::read_to_string;

use bp_core::acl::{AccessControlList, DomainRule};

#[test]
fn test_load_from_file() {
    let acl = AccessControlList::default();
    assert!(acl.load_from_file("tests/fixtures/acl_simple.txt").is_ok());
    assert_eq!(acl.count(), 6);
}

#[test]
fn test_to_pac() {
    let acl = AccessControlList::default();
    assert!(acl.load_from_file("tests/fixtures/acl_simple.txt").is_ok());
    insta::assert_debug_snapshot!(acl.to_pac("127.0.0.1:1080"));
}

#[test]
fn test_save_to_file() {
    let acl = AccessControlList::default();
    acl.push("foo.com", DomainRule::ExactMatch);
    acl.push("bar.com", DomainRule::NotExtractMatch);
    acl.push("baz.com", DomainRule::FuzzyMatch);
    acl.push("bad.com", DomainRule::Ignore);

    let tmp_path = "tests/tmp/acl.txt";

    assert!(acl.save_to_file(tmp_path.into()).is_ok());
    insta::assert_debug_snapshot!(read_to_string(tmp_path).unwrap());
}

#[test]
fn test_is_host_hit() {
    let acl = AccessControlList::default();
    acl.push("foo.com", DomainRule::FuzzyMatch);
    acl.push("foo.com", DomainRule::Ignore);
    acl.push("bar.com", DomainRule::ExactMatch);
    acl.push("baz.com", DomainRule::NotExtractMatch);

    assert!(acl.is_host_hit("foo.com"));
    assert!(acl.is_host_hit("www.foo.com"));
    assert!(!acl.is_host_hit("www.bar.com"));
    assert!(!acl.is_host_hit("baz.com"));
    assert!(!acl.is_host_hit("www.baz.com"));
}
