use std::fs::read;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub wallet: String,
}

impl Config {
    pub fn read_from_file() -> Self {
        let config = match read("config.toml") {
            Ok(bytes) => toml::from_str(&String::from_utf8_lossy(&bytes))
                .expect("config.toml must have valid toml format."),
            Err(_) => Config::default(),
        };

        // Validate the config
        if config.wallet.is_empty() {
            panic!("wallet address should not be empty")
        }

        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wallet: "0x388C818CA8B9251b393131C08a736A67ccB19297".to_string(),
        }
    }
}

#[test]
fn test_toml_config() {
    let toml_str = r#"
    wallet = "0x388C818CA8B9251b393131C08a736A67ccB19297"
    "#;

    let config = toml::from_str::<Config>(toml_str).unwrap();
    assert_eq!(config.wallet, "0x388C818CA8B9251b393131C08a736A67ccB19297");
}
