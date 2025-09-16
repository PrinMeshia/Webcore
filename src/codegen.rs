use std::fs;
use std::path::Path;
use crate::ast::*;

pub fn generate(components: &Vec<Component>, out_dir: &str, config: &crate::config::Config) -> Result<(), String> {
    if components.is_empty() {
        return Err("No components to generate".into());
    }

    let comp = &components[0];
    let name = &comp.name;
    
    // Pour l'instant, on génère un code simple qui fonctionne
    // TODO: Améliorer pour utiliser la nouvelle AST
    
    // Génération du state.js
    let js_module = r#"export class StateModel {
    #name;
    constructor(initial) { this.#name = initial; }
    get name() { return this.#name; }
    set name(v) { this.#name = v; this.render?.(); }
    render() {
        const el = document.getElementById('ws_name');
        if (el) el.textContent = this.#name;
    }
}

export const app = new StateModel("World");
"#;

    // Génération du main.js
    let main_module = r#"import { app } from './state.js';

document.addEventListener('DOMContentLoaded', () => {
  const btn = document.querySelector('button');
  if (btn) btn.addEventListener('click', () => { app.name = 'WebCore'; });
  app.render();
});
"#;

    // Génération du CSS
    let css = r#".counter {
    padding: 1rem;
    border: 1px solid #ccc;
    border-radius: 8px;
    max-width: 300px;
    margin: 0 auto;
}

h1 {
    color: #333;
    text-align: center;
}

button {
    padding: 0.5rem 1rem;
    margin: 0.25rem;
    border: none;
    border-radius: 4px;
    background: #007bff;
    color: white;
    cursor: pointer;
}

button:hover {
    background: #0056b3;
}
"#;

    // Génération du HTML
    let lang = config.app.lang.clone().unwrap_or_else(|| "en".into());
    let title = config.app.title.clone().unwrap_or_else(|| name.clone());
    let head_extras = if config.scripts.preload.is_empty() { String::new() } else {
        config.scripts.preload.iter().map(|s| format!("<script src=\"{}\" defer></script>\n", s)).collect::<String>()
    };

    let html = format!(r#"<!DOCTYPE html>
<html lang="{}">
<head>
  <meta charset="{}">
  <meta name="viewport" content="{}">
  <title>{}</title>
  {}
  <link rel="stylesheet" href="out.css">
  <script type="module" src="main.js" defer></script>
</head>
<body>
  <div class="counter">
    <h1>Hello <span id="ws_name">World</span>!</h1>
    <button>Click me</button>
  </div>
</body>
</html>"#,
        lang,
        config.meta.charset,
        config.meta.viewport,
        title,
        head_extras
    );

    // Création du répertoire de sortie
    let out_path = Path::new(out_dir);
    if let Err(e) = fs::create_dir_all(out_path) {
        return Err(format!("Failed to create out dir: {}", e));
    }

    // Écriture des fichiers
    fs::write(out_path.join("state.js"), js_module).map_err(|e| format!("Failed write state.js: {}", e))?;
    fs::write(out_path.join("main.js"), main_module).map_err(|e| format!("Failed write main.js: {}", e))?;
    fs::write(out_path.join("out.tmp.css"), css).map_err(|e| format!("Failed write out.tmp.css: {}", e))?;
    fs::write(out_path.join("index.html"), html).map_err(|e| format!("Failed write index.html: {}", e))?;

    println!("Generated HTML/JS/CSS in {}", out_path.display());
    Ok(())
}
