use std::env;

pub struct Config {
    pub endpoint: String,
    pub token: String,
}

pub fn get() -> Option<Config> {
    if let (Ok(x), Ok(y)) = (env::var("RDFS_ENDPOINT"), env::var("RDFS_TOKEN")) {
        return Some(Config {
            endpoint: x,
            token: y,
        });
    }
    None
}
