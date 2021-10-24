use regex::{Captures, Regex};

pub struct Options {
    pub match_prefix: String,
    pub match_postfix: String,
}

pub struct Template {
    regex: Regex,
}

impl Template {
    fn escape_regex_chars(s: &str) -> String {
        s.replace("$", "\\$").replace("{", "\\{").replace("}", "\\}")
    }

    pub fn new() -> Self {
        Self::from_options(Options {
            match_prefix: "{{".to_string(),
            match_postfix: "}}".to_string(),
        })
    }

    pub fn from_options(opts: Options) -> Self {
        let match_prefix = Self::escape_regex_chars(&opts.match_prefix);
        let match_postfix = Self::escape_regex_chars(&opts.match_postfix);

        let exp = format!("{}(.*?){}", match_prefix, match_postfix);
        let regex = Regex::new(&exp).unwrap();

        Self { regex }
    }

    pub fn render<T>(&self, tpl: &str, ctx: &T) -> String
    where
        T: Getter,
    {
        let cooked = self.regex.replace_all(tpl, |caps: &Captures| {
            if caps.len() == 2 {
                let path = caps.get(1).unwrap().as_str().trim();
                ctx.get_by_path(path).unwrap_or("".to_string())
            } else {
                "".to_string()
            }
        });

        cooked.to_string()
    }
}

pub trait Getter {
    fn get_by_path(&self, path: &str) -> Option<String>;
}

#[test]
fn test_template() {
    use crate::context::Context;
    use serde_json::Value;
    use std::str::FromStr;

    let template = Template::new();

    let data = r#"{
      "bar": "bar",
      "foo": [{
        "qux": "foo.0_qux"
      }]
    }"#;

    let data = Value::from_str(data).unwrap();

    let ctx = Context {
        data: Some(&data),
        secrets: None,
    };

    assert_eq!(
        template.render("foo {{ bar }} baz {{ foo.0.qux }}", &ctx),
        "foo bar baz foo.0_qux"
    );
}
