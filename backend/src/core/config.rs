use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub server_host: String,
    pub server_port: u16,
    pub ws_path: String,
    pub plugin_dir: String,
}

pub fn load_config() -> Result<Config, anyhow::Error> {
    dotenv().ok();

    let config = Config {
        database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        jwt_expires_in: env::var("JWT_EXPIRES_IN").unwrap_or_else(|_| "24h".to_string()),
        server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        server_port: env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()?,
        ws_path: env::var("WS_PATH").unwrap_or_else(|_| "/ws".to_string()),
        plugin_dir: env::var("PLUGIN_DIR").unwrap_or_else(|_| "plugins".to_string()),
    };

    Ok(config)
}
