use std::{
    fmt::Display,
    fs,
    io::Write,
    path::PathBuf,
    sync::{mpsc, Arc},
    time,
};

use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::Mutex;
use serde::Serialize;
use tinytemplate::{format_unescaped, TinyTemplate};

use super::rule::{Rule, RuleGroup, RulePrefix, RuleValue};
use crate::{global, Shutdown};

pub fn get_acl() -> Arc<AccessControlList> {
    global::get_acl()
}

#[derive(Default, Debug)]
pub struct AccessControlList {
    rules: Mutex<Vec<Rule>>,
}

impl AccessControlList {
    pub fn load_from_file(&self, path: &str) -> Result<()> {
        log::info!("loading acl from {}", path);

        let content = fs::read_to_string(path)?;
        self.clear();
        self.deserialize(&content);

        log::info!("loaded {} valid rules", self.count());

        Ok(())
    }

    pub fn save_to_file(&self, path: PathBuf) -> Result<()> {
        let mut file = fs::OpenOptions::new().write(true).create(true).open(path)?;
        let content = self.serialize();

        file.write_all(content.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    pub fn push<V>(&self, group: RuleGroup, prefix: RulePrefix, value: V)
    where
        V: Into<RuleValue> + Display,
    {
        self.rules.lock().insert(
            0,
            Rule {
                raw: format!("{}{}", prefix, value),
                group,
                prefix,
                value: value.into(),
            },
        );
    }

    pub fn try_match(&self, host: &str, port: Option<u16>) -> Option<Rule> {
        let rules = self.rules.lock();

        for rule in rules.iter().rev() {
            match rule.prefix {
                RulePrefix::Exact => {
                    if rule.value.is_match(host, port) {
                        return Some(rule.clone());
                    }
                }
                RulePrefix::Fuzzy => {
                    if rule.value.is_fuzzy_match(host, port) {
                        return Some(rule.clone());
                    }
                }
                RulePrefix::Ignore => {}
            }
        }

        None
    }

    pub fn watch(&self, path: &str, shutdown: Shutdown) -> notify::Result<()> {
        // Create a channel to receive the events.
        let (tx, rx) = mpsc::channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher: RecommendedWatcher = Watcher::new(tx, time::Duration::from_secs(2))?;

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(path, RecursiveMode::NonRecursive)?;

        // unwatch when shutdown
        tokio::spawn(async move {
            shutdown.recv().await;
            drop(watcher);
        });

        // This is a simple loop, but you may want to use more complex logic here,
        // for example to handle I/O.
        loop {
            match rx.recv() {
                Ok(notify::DebouncedEvent::Write(_)) => {
                    if let Err(res) = self.load_from_file(path) {
                        log::warn!("reload failed due to: {}", res.to_string())
                    }
                }
                Err(_) => {
                    break;
                }
                _ => continue,
            }
        }

        Ok(())
    }

    pub fn count(&self) -> usize {
        self.rules
            .lock()
            .iter()
            .filter(|&x| x.prefix != RulePrefix::Ignore)
            .count()
    }

    pub fn to_pac(&self, proxy_addr: &str) -> Result<String> {
        let mut if_statements: Vec<String> = vec![];

        for rule in self.rules.lock().iter().rev() {
            let mut if_conditions: Vec<String> = vec![];

            let host_condition = match rule.value.host.as_str() {
                "*" => "".to_string(),
                v => match rule.prefix {
                    RulePrefix::Exact => format!(r#"host === "{}""#, v),
                    RulePrefix::Fuzzy => format!(r#"shExpMatch(host, "*{}*")"#, v),
                    RulePrefix::Ignore => "".to_string(),
                },
            };

            let port_condition = match rule.value.port.as_str() {
                "*" => "".to_string(),
                // port is a js variable defined in assets/pac.tpl
                v => format!("port === {}", v),
            };

            if !host_condition.is_empty() {
                if_conditions.push(host_condition);
            }
            if !port_condition.is_empty() {
                if_conditions.push(port_condition);
            }

            let if_return = match rule.group {
                RuleGroup::Allow => format!("PROXY {}; DIRECT", proxy_addr),
                RuleGroup::Deny => "DIRECT".to_string(),
            };

            let statement = match rule.prefix {
                RulePrefix::Exact | RulePrefix::Fuzzy => {
                    let if_condition = if if_conditions.is_empty() {
                        "true".to_string()
                    } else {
                        if_conditions.join(" && ")
                    };
                    format!(r#"if ({}) return "{}";"#, if_condition, if_return)
                }
                RulePrefix::Ignore => {
                    format!("// {}", rule.raw)
                }
            };

            // add two leading spaces for better readability
            if_statements.push(format!("  {}", statement));
        }

        let mut tt = TinyTemplate::new();
        tt.add_template("pac", include_str!("assets/pac.tpl"))?;
        tt.set_default_formatter(&format_unescaped);

        #[derive(Serialize)]
        struct Context {
            if_statements: String,
        }

        let rendered = tt.render(
            "pac",
            &Context {
                if_statements: if_statements.join("\n"),
            },
        )?;

        Ok(rendered)
    }

    fn clear(&self) {
        self.rules.lock().clear();
    }

    fn deserialize(&self, content: &str) {
        let mut rules = self.rules.lock();
        let mut group = RuleGroup::Deny;

        for line in content.lines() {
            let line = line.trim();

            // ignore empty line
            if line.is_empty() {
                continue;
            }

            // only keep the first part, for example: "keep_me ignore_me also_ignore" -> "keep_me"
            let mut split = line.split(' ');
            let item = split.next().unwrap();

            // determine to use which group
            match item.to_uppercase().as_str() {
                "[ALLOW]" => {
                    group = RuleGroup::Allow;
                    continue;
                }
                "[DENY]" => {
                    group = RuleGroup::Deny;
                    continue;
                }
                _ => {}
            }

            // obtain prefix
            let prefix = match item.chars().next().unwrap() {
                '~' => RulePrefix::Fuzzy,
                '#' => RulePrefix::Ignore,
                _ => RulePrefix::Exact,
            };

            // skip prefix
            let skip_n = usize::from(prefix != RulePrefix::Exact);
            let chars = item.chars().skip(skip_n);

            // to string
            let value = chars.collect::<String>();

            rules.push(Rule {
                raw: line.to_string(),
                group: group.clone(),
                prefix,
                value: value.as_str().into(),
            });
        }
    }

    fn serialize(&self) -> String {
        let rules = self.rules.lock();
        let groups = rules.group_by(|a, b| a.group == b.group);
        let mut lines: Vec<String> = vec![];

        for group in groups.rev() {
            let name = group[0].group.to_string();
            lines.push(name);
            lines.push("\n".to_string());

            for rule in group.iter().rev() {
                lines.push(format!("{}{}\n", rule.prefix, rule.value));
            }
            lines.push("\n".to_string());
        }

        lines.join("")
    }
}
