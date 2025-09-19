//! CSS Code Generator

use crate::theme::Theme;

pub fn generate_css() -> String {
    "/* CSS output placeholder */".to_string()
}

pub fn generate_theme_css(theme: &Theme) -> String {
    let mut css = String::new();
    css.push_str(":root {\n");
    
    // Generate color variables
    for (key, value) in &theme.colors {
        css.push_str(&format!("  --color-{}: {};\n", key.replace("-", "-"), value));
    }
    
    // Generate font variables
    for (key, value) in &theme.fonts {
        css.push_str(&format!("  --font-{}: {};\n", key, value));
    }
    
    // Generate radius variables
    for (key, value) in &theme.radius {
        css.push_str(&format!("  --radius-{}: {};\n", key, value));
    }
    
    // Generate breakpoint variables
    for (key, value) in &theme.breakpoints {
        css.push_str(&format!("  --breakpoint-{}: {};\n", key, value));
    }
    
    css.push_str("}\n");
    css
}
