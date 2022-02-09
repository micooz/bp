use std::fs::read_to_string;

use bp_core::acl::{AccessControlList, RuleGroup, RulePrefix};

#[test]
fn test_load_from_file() {
    let acl = AccessControlList::default();
    assert!(acl.load_from_file("tests/fixtures/acl_simple.txt").is_ok());
    assert_eq!(acl.count(), 7);
}

#[test]
fn test_to_pac() {
    let acl = AccessControlList::default();
    assert!(acl.load_from_file("tests/fixtures/acl_simple.txt").is_ok());
    insta::assert_snapshot!(acl.to_pac("127.0.0.1:1080").unwrap());
}

#[test]
fn test_save_to_file() {
    let acl = AccessControlList::default();
    acl.push(RuleGroup::Allow, RulePrefix::Exact, "");
    acl.push(RuleGroup::Allow, RulePrefix::Exact, "*");
    acl.push(RuleGroup::Allow, RulePrefix::Exact, "*:");
    acl.push(RuleGroup::Allow, RulePrefix::Exact, "*:*");
    acl.push(RuleGroup::Allow, RulePrefix::Fuzzy, "");
    acl.push(RuleGroup::Allow, RulePrefix::Fuzzy, "*");

    acl.push(RuleGroup::Deny, RulePrefix::Exact, "bar1.com");
    acl.push(RuleGroup::Deny, RulePrefix::Fuzzy, "bar2.com:80");
    acl.push(RuleGroup::Deny, RulePrefix::Ignore, "bar3.com:443");

    acl.push(RuleGroup::Allow, RulePrefix::Exact, "foo1.com");
    acl.push(RuleGroup::Allow, RulePrefix::Fuzzy, "foo2.com:80");
    acl.push(RuleGroup::Allow, RulePrefix::Ignore, "foo3.com:443");

    let tmp_path = "tests/tmp/acl.txt";

    assert!(acl.save_to_file(tmp_path.into()).is_ok());
    insta::assert_snapshot!(read_to_string(tmp_path).unwrap());
}

#[test]
fn test_try_match() {
    let acl = AccessControlList::default();

    acl.load_from_file("tests/fixtures/acl_simple.txt").unwrap();
    assert!(acl.try_match("xxx.com", None).is_none());
    assert!(acl.try_match("example.net", None).unwrap().is_deny());
    assert!(acl.try_match("example.net", Some(80)).unwrap().is_deny());
    assert!(acl.try_match("foo_baz.com", None).unwrap().is_deny());

    acl.load_from_file("tests/fixtures/acl_whitelist.txt").unwrap();
    assert!(acl.try_match("example.com", None).unwrap().is_allow());
    assert!(acl.try_match("example.com", Some(53)).unwrap().is_allow());
    assert!(acl.try_match("_example.com", None).unwrap().is_deny());
    assert!(acl.try_match("example.net", None).unwrap().is_deny());

    acl.load_from_file("tests/fixtures/acl_blacklist.txt").unwrap();
    assert!(acl.try_match("example.com", None).unwrap().is_deny());
    assert!(acl.try_match("example.com", Some(53)).unwrap().is_deny());
    assert!(acl.try_match("_example.com", None).unwrap().is_allow());
    assert!(acl.try_match("example.net", None).unwrap().is_allow());

    acl.load_from_file("tests/fixtures/acl_mixed.txt").unwrap();
    assert!(acl.try_match("baz.com", Some(443)).unwrap().is_deny());
    assert!(acl.try_match("example.com", Some(443)).unwrap().is_allow());
    assert!(acl.try_match("bar.com", None).unwrap().is_allow());
    assert!(acl.try_match("deny.com", None).unwrap().is_deny());
}
