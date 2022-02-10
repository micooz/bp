use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct Rule {
    pub raw: String,
    pub group: RuleGroup,
    pub prefix: RulePrefix,
    pub value: RuleValue,
}

impl Rule {
    pub fn is_allow(&self) -> bool {
        self.group == RuleGroup::Allow
    }
    pub fn is_deny(&self) -> bool {
        self.group == RuleGroup::Deny
    }
}

#[derive(Debug, Clone)]
pub struct RuleValue {
    pub host: String,
    pub port: String,
}

impl RuleValue {
    pub fn is_match(&self, host: &str, port: Option<u16>) -> bool {
        if self.host != "*" && self.host != host {
            return false;
        }
        self.match_port(port)
    }

    pub fn is_fuzzy_match(&self, host: &str, port: Option<u16>) -> bool {
        if self.host != "*" && !host.contains(&self.host) {
            return false;
        }
        self.match_port(port)
    }

    fn match_port(&self, port: Option<u16>) -> bool {
        if self.port == "*" {
            return true;
        }
        if port.is_some() && self.port == port.unwrap().to_string() {
            return true;
        }
        false
    }
}

impl Display for RuleValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}",
            if self.host.is_empty() { "*" } else { &self.host },
            if self.port.is_empty() { "*" } else { &self.port },
        )
    }
}

impl From<&str> for RuleValue {
    fn from(s: &str) -> Self {
        let mut split = s.split(':');
        let host = split.next().unwrap_or("*");
        let port = split.next().unwrap_or("*");

        RuleValue {
            host: host.to_string(),
            port: port.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RulePrefix {
    Exact,
    Fuzzy,
    Ignore,
}

impl Display for RulePrefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Self::Exact => "",
            Self::Fuzzy => "~",
            Self::Ignore => "#",
        };
        write!(f, "{}", v)
    }
}

impl Default for RulePrefix {
    fn default() -> Self {
        Self::Ignore
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuleGroup {
    Allow,
    Deny,
}

impl Display for RuleGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Self::Allow => "[Allow]",
            Self::Deny => "[Deny]",
        };
        write!(f, "{}", v)
    }
}
