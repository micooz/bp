use std::{
    fmt::Display,
    fs,
    io::{Read, Write},
    path::PathBuf,
    sync, time,
};

use anyhow::{Error, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

// AccessControlList

#[derive(Default, Debug)]
pub struct AccessControlList {
    domain_white_list: sync::Mutex<Vec<DomainItem>>,
}

impl AccessControlList {
    pub fn load_from_file(&self, path: String) -> Result<()> {
        if path.is_empty() {
            return Err(Error::msg("empty string specified"));
        }

        log::info!("loading white list from {}", path);

        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .unwrap();

        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        self.clear();
        self.parse(&content);

        log::info!("loaded {} valid rules", self.count());

        Ok(())
    }

    pub fn save_to_file(&self, path: PathBuf) -> Result<()> {
        let mut file = fs::OpenOptions::new().write(true).create(true).open(path)?;

        let buf = self.stringify();

        file.write_all(buf.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    pub fn push(&self, host: &str, rule: DomainRule) {
        self.domain_white_list.lock().unwrap().insert(
            0,
            DomainItem {
                raw: format!("{}{}", rule, host),
                rule,
                value: host.to_string(),
            },
        );
    }

    pub fn is_host_hit(&self, host: &str) -> bool {
        let domain_white_list = self.domain_white_list.lock().unwrap();

        for item in domain_white_list.iter() {
            match item.rule {
                DomainRule::ExactMatch => {
                    if item.value == host {
                        return true;
                    }
                }
                DomainRule::NotExtractMatch => {
                    if item.value == host {
                        return false;
                    }
                }
                DomainRule::FuzzyMatch => {
                    if host.contains(&item.value) {
                        return true;
                    }
                }
                DomainRule::Ignore => {}
            }
        }

        false
    }

    pub fn watch(&self, path: String) -> notify::Result<()> {
        // Create a channel to receive the events.
        let (tx, rx) = sync::mpsc::channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher: RecommendedWatcher = Watcher::new(tx, time::Duration::from_secs(2))?;

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        // This is a simple loop, but you may want to use more complex logic here,
        // for example to handle I/O.
        loop {
            if let Ok(notify::DebouncedEvent::Write(_)) = rx.recv() {
                if let Err(res) = self.load_from_file(path.clone()) {
                    log::warn!("reload failed due to: {}", res.to_string())
                }
            }
        }
    }

    pub fn count(&self) -> usize {
        self.domain_white_list
            .lock()
            .unwrap()
            .iter()
            .filter(|&x| x.rule != DomainRule::Ignore)
            .count()
    }

    fn clear(&self) {
        self.domain_white_list.lock().unwrap().clear();
    }

    fn parse(&self, content: &str) {
        let mut domain_white_list = self.domain_white_list.lock().unwrap();

        for line in content.lines() {
            let line = line.trim();

            let found = domain_white_list.iter().enumerate().find_map(
                |(index, v)| {
                    if v.raw == line {
                        Some(index)
                    } else {
                        None
                    }
                },
            );

            if let Some(index) = found {
                domain_white_list.remove(index);
            }

            let mut chars = line.trim().chars();
            let mut first_char = None;

            let rule = match chars.next() {
                Some(ch) => match ch {
                    '~' => DomainRule::FuzzyMatch,
                    '#' => DomainRule::Ignore,
                    '!' => DomainRule::NotExtractMatch,
                    _ => {
                        first_char = Some(ch);
                        DomainRule::ExactMatch
                    }
                },
                None => continue,
            };

            let mut value = chars.collect::<String>();

            if let Some(ch) = first_char {
                value.insert(0, ch);
            }

            domain_white_list.push(DomainItem {
                raw: line.to_string(),
                rule,
                value,
            });
        }

        domain_white_list.reverse();
    }

    fn stringify(&self) -> String {
        self.domain_white_list
            .lock()
            .unwrap()
            .iter()
            .rev()
            .map(|x| format!("{}{}", x.rule, x.value))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

// DomainItem

#[derive(Default, Debug, PartialEq, Eq)]
pub struct DomainItem {
    raw: String,
    rule: DomainRule,
    value: String,
}

// DomainRule

#[derive(Debug, PartialEq, Eq)]
pub enum DomainRule {
    ExactMatch,
    NotExtractMatch,
    FuzzyMatch,
    Ignore,
}

impl Display for DomainRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            DomainRule::ExactMatch => "",
            DomainRule::NotExtractMatch => "!",
            DomainRule::FuzzyMatch => "~",
            DomainRule::Ignore => "#",
        };
        write!(f, "{}", v)
    }
}

impl Default for DomainRule {
    fn default() -> Self {
        Self::Ignore
    }
}
