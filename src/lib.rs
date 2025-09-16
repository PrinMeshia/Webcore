pub mod ast;
pub mod parser;
pub mod codegen;
pub mod transformers;
pub mod config;

use anyhow::Result;
use std::fs;

pub fn build_project(input: &str, out_dir: &str) -> Result<()> {
    // read config
    let config_str = fs::read_to_string("webc.toml").unwrap_or_else(|_| String::from(r#"[app]
title = "MyApp"
lang = "en"

[meta]
charset = "UTF-8"
viewport = "width=device-width, initial-scale=1.0"

[scripts]
preload = []
"#));
    let config: config::Config = toml::from_str(&config_str)?;

    // read input
    let input_src = fs::read_to_string(input)?;
    let components = parser::parse_components(&input_src)
        .map_err(|e| anyhow::anyhow!(e))?;

    // codegen
    codegen::generate(&components, out_dir, &config)
        .map_err(|e| anyhow::anyhow!(e))?;

    // post-process (currently stubs)
    let state_js_path = format!("{}/state.js", out_dir);
    let js_tmp = fs::read_to_string(&state_js_path)?;
    let js_final = transformers::transform_js(&js_tmp, "es_latest")
        .map_err(|e| anyhow::anyhow!(e))?;
    fs::write(&state_js_path, js_final)?;

    let css_tmp_path = format!("{}/out.tmp.css", out_dir);
    let css_tmp = fs::read_to_string(&css_tmp_path)?;
    let css_final = transformers::transform_css(&css_tmp, vec!["last 2 versions"])
        .map_err(|e| anyhow::anyhow!(e))?;
    fs::write(format!("{}/out.css", out_dir), css_final)?;
    
    // Supprimer le fichier temporaire
    fs::remove_file(&css_tmp_path)?;

    println!("Build finished. Open {}/index.html", out_dir);
    Ok(())
}
