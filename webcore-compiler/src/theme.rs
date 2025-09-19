//! Theme system for WebCore

use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: HashMap<String, String>,
    pub fonts: HashMap<String, String>,
    pub radius: HashMap<String, String>,
    pub breakpoints: HashMap<String, String>,
}

#[derive(Debug, serde::Deserialize)]
struct ThemeConfig {
    theme: ThemeData,
}

#[derive(Debug, serde::Deserialize)]
struct ThemeData {
    name: String,
    colors: HashMap<String, String>,
    fonts: HashMap<String, String>,
    radius: HashMap<String, String>,
    breakpoints: HashMap<String, String>,
}

pub fn load_theme(theme_path: &str) -> Result<Theme, String> {
    let content = fs::read_to_string(theme_path)
        .map_err(|e| format!("Failed to read theme file {}: {}", theme_path, e))?;
    
    let config: ThemeConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse theme file {}: {}", theme_path, e))?;
    
    Ok(Theme {
        name: config.theme.name,
        colors: config.theme.colors,
        fonts: config.theme.fonts,
        radius: config.theme.radius,
        breakpoints: config.theme.breakpoints,
    })
}

