use crate::utils::template::Getter;
use json_dotpath::DotPaths;
use serde_json::Value;

const SECRETS_PREFIX: &str = "secrets.";

pub struct Context<'a> {
    pub data: Option<&'a Value>,
    pub secrets: Option<&'a Value>,
}

impl<'a> Getter for Context<'a> {
    fn get_by_path(&self, path: &str) -> Option<String> {
        if path.is_empty() {
            return None;
        }

        if self.secrets.is_some() && path.starts_with(SECRETS_PREFIX) {
            let dot_path = path.strip_prefix(SECRETS_PREFIX).unwrap();
            return self.secrets.unwrap().dot_get(dot_path).ok().unwrap();
        }

        if self.data.is_some() {
            return self.data.unwrap().dot_get(path).ok().unwrap();
        }

        None
    }
}
