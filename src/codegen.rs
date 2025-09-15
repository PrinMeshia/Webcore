use std::fs;
use std::path::Path;
use crate::ast::Component;

fn extract_block_content(raw: &str) -> String {
    // Find first '{' and last '}' and return inner trimmed content
    if let Some(start) = raw.find('{') {
        if let Some(end) = raw.rfind('}') {
            let inner = &raw[start+1..end];
            return inner.trim().to_string();
        }
    }
    raw.to_string()
}

pub fn generate(components: &Vec<Component>, out_dir: &str) -> Result<(), String> {
    if components.is_empty() {
        return Err("No components to generate".into());
    }

    // We'll generate a simple index.html using the first component
    let comp = &components[0];
    let name = &comp.name;
    let state = comp.states.get(0);
    let state_name = state.map(|s| s.name.clone()).unwrap_or_else(|| "state".into());
    let state_value = state.map(|s| s.value.clone()).unwrap_or_else(|| "\"\"".into());

    let view_raw = comp.view.as_ref().map(|v| extract_block_content(v)).unwrap_or_default();
    let style_raw = comp.style.as_ref().map(|s| extract_block_content(s)).unwrap_or_default();

    // Simple heuristic: replace interpolation {name} with span with id
    // We'll create an element id for each state occurrence. For MVP handle the first state
    let placeholder = format!("{{{}}}", state_name);
    let rendered_view = view_raw.replace(&placeholder, &format!("<span id=\"ws_{}\">" , state_name) + "</span>");
    // The above produces <span id="ws_name"></span>, but we need to ensure text content is filled via JS.
    // To keep HTML simple, we'll also try to convert tags correctly if view contains full tags.

    let html = format!(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
      <meta charset="UTF-8">
      <meta name="viewport" content="width=device-width, initial-scale=1.0">
      <title>{name}</title>
      <style>
    {style}
      </style>
    </head>
    <body>
    {view}
    <script>
    // Simple state and render for MVP
    let state = {{ {state_name}: {state_value} }};

    function render() {{
      const el = document.getElementById('ws_{state_name}');
      if (el) {{
        el.textContent = state['{state_name}'];
      }}
    }}

    // Wire up first button with id heuristic: find first button and attach click that sets state to 'WebCore' if referenced
    const btn = document.querySelector('button');
    if (btn) {{
      btn.addEventListener('click', () => {{
        state['{state_name}'] = 'WebCore';
        render();
      }});
    }}

    // Initial render
    render();
    </script>
    </body>
    </html>
    "#,
        name = name,
        style = style_raw,
        view = view_raw,
        state_name = state_name,
        state_value = state_value
    );

    // create out_dir
    let out_path = Path::new(out_dir);
    if let Err(e) = fs::create_dir_all(out_path) {
        return Err(format!("Failed to create out dir: {}", e));
    }

    let index_path = out_path.join("index.html");
    if let Err(e) = fs::write(&index_path, html) {
        return Err(format!("Failed to write index.html: {}", e));
    }

    println!("Generated {}", index_path.display());
    Ok(())
}
