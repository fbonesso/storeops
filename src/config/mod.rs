pub mod profiles;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub active_profile: Option<String>,
    #[serde(default)]
    pub profiles: HashMap<String, profiles::Profile>,
}

impl Config {
    pub fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("com", "storeops", "storeops").map(|d| d.config_dir().to_path_buf())
    }

    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path().ok_or("cannot determine config directory")?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Self::config_dir().ok_or("cannot determine config directory")?;
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("config.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn active_profile(&self) -> Option<&profiles::Profile> {
        self.active_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use profiles::{Credentials, Profile, Store};

    #[test]
    fn default_config_has_no_profiles() {
        let config = Config::default();
        assert!(config.profiles.is_empty());
        assert!(config.active_profile.is_none());
    }

    #[test]
    fn serialization_round_trip() {
        let mut config = Config::default();
        config.active_profile = Some("test".to_string());
        config.profiles.insert(
            "test".to_string(),
            Profile {
                store: Store::Apple,
                credentials: Credentials::Apple {
                    key_id: "K1".to_string(),
                    issuer_id: "I1".to_string(),
                    key_path: "/tmp/key.p8".to_string(),
                },
            },
        );

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.active_profile, Some("test".to_string()));
        assert!(deserialized.profiles.contains_key("test"));
    }

    #[test]
    fn profiles_can_be_added_and_retrieved() {
        let mut config = Config::default();
        config.profiles.insert(
            "myprofile".to_string(),
            Profile {
                store: Store::Google,
                credentials: Credentials::Google {
                    service_account_path: "/tmp/sa.json".to_string(),
                },
            },
        );
        config.active_profile = Some("myprofile".to_string());

        let active = config.active_profile().unwrap();
        assert!(matches!(active.store, Store::Google));
    }

    #[test]
    fn active_profile_returns_none_when_not_set() {
        let config = Config::default();
        assert!(config.active_profile().is_none());
    }

    #[test]
    fn active_profile_returns_none_for_missing_name() {
        let mut config = Config::default();
        config.active_profile = Some("nonexistent".to_string());
        assert!(config.active_profile().is_none());
    }
}
