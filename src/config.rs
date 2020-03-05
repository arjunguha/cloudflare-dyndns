use serde::Deserialize;
use std::convert::AsRef;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub cloudflare_auth_token: String,
    pub zone_identifier: String,
    pub domain_name: String,
    pub ip_query_addess: String,
}

impl Config {
    pub fn from_file<P>(filename: P) -> Config
    where
        P: AsRef<Path>,
    {
        let config_str = fs::read_to_string(filename).expect("opening configuration file");
        return serde_json::from_str(&config_str).expect("parsing configuration file");
    }
}
