use std::collections::HashMap;

use mingling::{LazyInit, Program, macros::program_setup};

use crate::ThisProgram;

#[derive(Debug, Default, Clone)]
pub struct ResMlingConfig {
    kvp: HashMap<String, String>,
}

impl ResMlingConfig {
    /// Edit a KVP entry. Empty string removes the key.
    pub fn edit(&mut self, key: &str, value: &str) {
        if value.is_empty() {
            self.kvp.remove(key);
        } else {
            self.kvp.insert(key.to_string(), value.to_string());
        }
    }

    /// Get a value by key, returning "" if it doesn't exist.
    pub fn get(&self, key: &str) -> &str {
        self.kvp.get(key).map(|s| s.as_str()).unwrap_or("")
    }

    /// Get a value by key, returning `or` if it doesn't exist.
    pub fn get_or<'a>(&'a self, key: &'a str, or: &'a str) -> &'a str {
        if self.kvp.contains_key(key) {
            self.kvp.get(key).map(|s| s.as_str()).unwrap_or(or)
        } else {
            or
        }
    }

    /// Get a value by key, returning `or` if it doesn't exist and setting it.
    pub fn get_or_set<'a>(&'a mut self, key: &'a str, or: &'a str) -> &'a str {
        if !self.kvp.contains_key(key) {
            self.kvp.insert(key.to_string(), or.to_string());
        }
        self.kvp.get(key).map(|s| s.as_str()).unwrap_or(or)
    }

    /// Read the config from disk, defaulting to empty if not present.
    pub fn read() -> Self {
        let path = std::path::Path::new("");
        Self::read_from_path(path)
    }

    /// Read the config from a file at the given path.
    pub fn read_from_path(path: &std::path::Path) -> Self {
        let mut config = Self::default();
        if let Ok(content) = std::fs::read_to_string(path)
            && let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content)
            && let serde_json::Value::Object(map) = json_value
        {
            for (key, value) in map {
                if let serde_json::Value::String(s) = value {
                    config.kvp.insert(key, s);
                }
            }
        }
        config
    }

    /// Write the config to disk at the given path.
    pub fn write_to_path(&self, path: &std::path::Path) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json_value = serde_json::Value::Object(
            self.kvp
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        );
        if let Ok(json_string) = serde_json::to_string(&json_value) {
            let _ = std::fs::write(path, json_string);
        }
    }

    /// Write the config to the default data directory path.
    pub fn write(&self) {
        if let Some(data_dir) = dirs::data_dir() {
            let path = data_dir.join("mingling").join("mling-cfg.json");
            self.write_to_path(&path);
        }
    }
}

impl From<HashMap<String, String>> for ResMlingConfig {
    fn from(kvp: HashMap<String, String>) -> Self {
        Self { kvp }
    }
}

impl From<ResMlingConfig> for HashMap<String, String> {
    fn from(config: ResMlingConfig) -> Self {
        config.kvp
    }
}

#[program_setup]
pub fn mling_config_setup(p: &mut Program<ThisProgram>) {
    p.with_resource(
        ResMlingConfig::lazy_init(ResMlingConfig::read).with_on_drop(|config: ResMlingConfig| {
            config.write();
        }),
    );
}
