use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub store: Store,
    #[serde(flatten)]
    pub credentials: Credentials,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Store {
    Apple,
    Google,
}

impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Store::Apple => write!(f, "apple"),
            Store::Google => write!(f, "google"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Credentials {
    Apple {
        key_id: String,
        issuer_id: String,
        key_path: String,
    },
    Google {
        service_account_path: String,
    },
}
