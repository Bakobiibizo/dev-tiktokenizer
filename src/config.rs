use std::env;

#[derive(Clone)]
pub struct Config {
    pub api_host: String,
    pub api_port: u16,
    pub default_tokenizer: String,
    pub default_embedding_model: String,
    pub preload: bool,
}

impl Config {
    pub fn load() -> Self {
        let api_host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let api_port = env::var("API_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(7105);
        let default_tokenizer =
            env::var("DEFAULT_TOKENIZER_MODEL").unwrap_or_else(|_| "cl100k_base".to_string());
        let default_embedding_model =
            env::var("DEFAULT_EMBEDDING_MODEL").unwrap_or_else(|_| "text-embedding-ada-002".to_string());
        let preload = env::var("PRELOAD")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(true);

        Self {
            api_host,
            api_port,
            default_tokenizer,
            default_embedding_model,
            preload,
        }
    }
}
