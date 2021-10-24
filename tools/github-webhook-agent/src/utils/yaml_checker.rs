use serde_json::Value;
use yaml_rust::Yaml;

pub struct YamlChecker {}

impl YamlChecker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn check_json(&self, rule: &Yaml, json: &Value) -> bool {
        // empty value match
        if rule.is_null() && !json.is_null() {
            return false;
        }

        // array match
        if rule.is_array() {
            if !json.is_array() {
                return false;
            }

            let arr = rule.as_vec().unwrap();

            for (index, item) in arr.iter().enumerate() {
                if !self.check_json(item, &json[index]) {
                    return false;
                }
            }

            return true;
        }

        // hash/object match
        if let Some(obj) = rule.as_hash() {
            if !json.is_object() {
                return false;
            }

            for (key, val) in obj {
                let key_str = key.as_str().unwrap();

                if !self.check_json(val, &json[key_str]) {
                    return false;
                }
            }
        }

        // primitive types match
        if rule.as_bool().is_some() && rule.as_bool() != json.as_bool() {
            return false;
        }
        if rule.as_f64().is_some() && rule.as_f64() != json.as_f64() {
            return false;
        }
        if rule.as_i64().is_some() && rule.as_i64() != json.as_i64() {
            return false;
        }
        if rule.as_str().is_some() && rule.as_str() != json.as_str() {
            return false;
        }

        true
    }
}

#[test]
fn test_is_match() {
    use std::str::FromStr;
    use yaml_rust::YamlLoader;

    let checker = YamlChecker::new();

    let json_str = r#"
        {
          "foo": {
            "bar": "value"
          },
          "will-not-match": "value",
          "baz": [
            { "key": "string" },
            { "key": 100 },
            { "key": false },
            { "key": "extra" }
          ]
        }
    "#;

    let json = Value::from_str(json_str).unwrap();

    let yaml_str = r#"
        foo:
          bar: value

        # don't match this key:
        # will-not-match: value

        baz:
          - key: string
          - key: 100
          - key: false
    "#;

    let yaml = YamlLoader::load_from_str(yaml_str).unwrap();

    assert!(checker.check_json(&yaml[0], &json));
}
