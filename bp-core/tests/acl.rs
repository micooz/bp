use bp_core::acl::*;
use std::{fs::read_to_string, path::PathBuf};

#[test]
fn test_load_from_file() {
    let acl = AccessControlList::default();

    assert!(acl.load_from_file("tests/fixtures/acl.txt".into()).is_ok());
    assert_eq!(acl.count(), 1);
}

#[test]
fn test_save_to_file() {
    let acl = AccessControlList::default();
    acl.push("foo.com", DomainRule::ExactMatch);
    acl.push("bar.com", DomainRule::NotExtractMatch);
    acl.push("baz.com", DomainRule::FuzzyMatch);
    acl.push("bad.com", DomainRule::Ignore);

    let mut tmp_path = PathBuf::new();
    tmp_path.push("tests");
    tmp_path.push("tmp");
    tmp_path.push("acl.txt");

    assert!(acl.save_to_file(tmp_path.clone()).is_ok());
    assert_eq!(
        read_to_string(tmp_path).unwrap(),
        "foo.com\n!bar.com\n~baz.com\n#bad.com"
    );
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