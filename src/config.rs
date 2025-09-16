use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub title: Option<String>,
    pub lang: Option<String>,
    pub sso: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetaConfig {
    pub charset: String,
    pub viewport: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ScriptsConfig {
    pub preload: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub meta: MetaConfig,
    #[serde(default)]
    pub scripts: ScriptsConfig,
}
