use std::env;
use std::fs::Permissions;
use std::path::Path;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub api_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Credentials {
    pub token: String,
    pub refresh_token: String,
}

fn get_default_config() -> Config {
    Config{
        api_url: "https://auth.whitedog.cloud".to_string(),
    }
}

fn get_config_dir() -> String {
    env::var("MA_PROXY_CONFIG_DIR").unwrap_or_else(|_| {
        let base_dirs = BaseDirs::new().expect("Couldn't get base dirs");
        let config_dir = base_dirs.config_dir().join("whitedog").join("middleauth-ssh-proxy");

        if !config_dir.exists() {
            std::fs::create_dir_all(config_dir.clone()).expect("Couldn't create config directory");
        }

        config_dir.into_string().unwrap()
    })
}

fn get_config_file_path() -> String {
    let config_dir = get_config_dir();
    let config_file_path = Path::new(&config_dir).join("config.toml");

    config_file_path.into_string().unwrap()
}

fn get_credentials_file_path() -> String {
    let config_dir = get_config_dir();
    let credentials_file_path = Path::new(&config_dir).join("credentials.toml");

    credentials_file_path.into_string().unwrap()
}

pub fn get_config() -> Config {
    let config_path = get_config_file_path();

    if Path::exists(config_path.as_ref()) {
        let config_content = std::fs::read_to_string(config_path).expect("Couldn't read config file");
        toml::from_str(&config_content).expect("Couldn't parse config file")
    } else {
        get_default_config()
    }
}

pub fn write_config(config: &Config) -> std::io::Result<()> {
    let config_path = get_config_file_path();
    std::fs::write(&config_path, toml::to_string(config).unwrap())
}

pub fn get_credentials() -> Option<Credentials> {
    let credentials_file_path = get_credentials_file_path();
    
    if Path::exists(credentials_file_path.as_ref()) {
        let credentials_content = std::fs::read_to_string(credentials_file_path).expect("Couldn't read credentials file");
        Some(toml::from_str(&credentials_content).expect("Couldn't parse credentials file"))
    } else { None }
}

pub fn write_credentials(credentials: &Credentials) -> std::io::Result<()> {
    let credentials_path = get_credentials_file_path();
    std::fs::write(&credentials_path, toml::to_string(credentials).unwrap())
}