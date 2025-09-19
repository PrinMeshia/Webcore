//! JavaScript Code Generator for WebCore Runtime

use crate::ast::*;
use crate::codegen::codegen_html::HandlerMapping;

fn compile_expression(expr: &str) -> String {
    let mut compiled = expr.to_string();
    
    // Handle += operator (e.g., count += 1)
    if compiled.contains("+=") {
        let parts: Vec<&str> = compiled.split("+=").collect();
        if parts.len() == 2 {
            let var_name = parts[0].trim();
            let value = parts[1].trim();
            compiled = format!("window.__webcore_state__.set('{}', (window.__webcore_state__.get('{}') || 0) + {})", 
                             var_name, var_name, value);
        }
    }
    // Handle -= operator (e.g., count -= 1)
    else if compiled.contains("-=") {
        let parts: Vec<&str> = compiled.split("-=").collect();
        if parts.len() == 2 {
            let var_name = parts[0].trim();
            let value = parts[1].trim();
            compiled = format!("window.__webcore_state__.set('{}', (window.__webcore_state__.get('{}') || 0) - {})", 
                             var_name, var_name, value);
        }
    }
    // Handle = operator (e.g., count = max(0, count - 1))
    else if compiled.contains("=") && !compiled.contains("==") && !compiled.contains("!=") {
        let parts: Vec<&str> = compiled.split("=").collect();
        if parts.len() == 2 {
            let var_name = parts[0].trim();
            let mut value_expr = parts[1].trim().to_string();
            
            // Replace max() calls
            value_expr = value_expr.replace("max(", "window.__webcore_utils__.max(");
            value_expr = value_expr.replace("min(", "window.__webcore_utils__.min(");
            
            // Replace variable references with state getters
            value_expr = value_expr.replace("count", "window.__webcore_state__.get('count')");
            
            compiled = format!("window.__webcore_state__.set('{}', {})", var_name, value_expr);
        }
    }
    // Handle other expressions
    else {
        // Replace max() calls
        compiled = compiled.replace("max(", "window.__webcore_utils__.max(");
        compiled = compiled.replace("min(", "window.__webcore_utils__.min(");
        
        // Replace variable references with state getters
        compiled = compiled.replace("count", "window.__webcore_state__.get('count')");
    }
    
    compiled
}

pub fn generate_js() -> String {
    "// JS output placeholder".to_string()
}

pub fn generate_runtime_js(handlers: &[HandlerMapping]) -> String {
    let mut js = String::new();
    
    // WebCore Runtime
    js.push_str("// WebCore Runtime\n");
    js.push_str("(function() {\n");
    js.push_str("  'use strict';\n\n");
    
    // Generate compiled handlers
    js.push_str("  // Compiled Event Handlers\n");
    js.push_str("  window.__webcore_handlers__ = {\n");
    for handler in handlers {
        js.push_str(&format!("    '{}': function() {{\n", handler.id));
        js.push_str("      try {\n");
        
        // Compile the expression to use state management
        let compiled_expr = compile_expression(&handler.expression);
        js.push_str(&format!("        {}\n", compiled_expr));
        
        js.push_str("      } catch (error) {\n");
        js.push_str("        console.error('Error executing handler:', error);\n");
        js.push_str("      }\n");
        js.push_str("    },\n");
    }
    js.push_str("  };\n\n");
    
    // State management
    js.push_str("  // State Management\n");
    js.push_str("  class WebCoreState {\n");
    js.push_str("    constructor() {\n");
    js.push_str("      this.data = new Map();\n");
    js.push_str("      this.listeners = new Map();\n");
    js.push_str("    }\n\n");
    
    js.push_str("    set(key, value) {\n");
    js.push_str("      this.data.set(key, value);\n");
    js.push_str("      this.notify(key, value);\n");
    js.push_str("    }\n\n");
    
    js.push_str("    get(key) {\n");
    js.push_str("      return this.data.get(key);\n");
    js.push_str("    }\n\n");
    
    js.push_str("    subscribe(key, callback) {\n");
    js.push_str("      if (!this.listeners.has(key)) {\n");
    js.push_str("        this.listeners.set(key, []);\n");
    js.push_str("      }\n");
    js.push_str("      this.listeners.get(key).push(callback);\n");
    js.push_str("    }\n\n");
    
    js.push_str("    notify(key, value) {\n");
    js.push_str("      const callbacks = this.listeners.get(key) || [];\n");
    js.push_str("      callbacks.forEach(callback => callback(value));\n");
    js.push_str("    }\n");
    js.push_str("  }\n\n");
    
    // Global state instance
    js.push_str("  window.__webcore_state__ = new WebCoreState();\n");
    js.push_str("  \n");
    js.push_str("  // Initialize default state\n");
    js.push_str("  window.__webcore_state__.set('count', 0);\n\n");
    
    // Event handlers
    js.push_str("  // Event Handlers\n");
    js.push_str("  function handleEvent(event, handler) {\n");
    js.push_str("    event.preventDefault();\n");
    js.push_str("    try {\n");
    js.push_str("      handler();\n");
    js.push_str("    } catch (error) {\n");
    js.push_str("      console.error('WebCore event handler error:', error);\n");
    js.push_str("    }\n");
    js.push_str("  }\n\n");
    
    // Utility functions
    js.push_str("  // Utility Functions\n");
    js.push_str("  window.__webcore_utils__ = {\n");
    js.push_str("    max: Math.max,\n");
    js.push_str("    min: Math.min,\n");
    js.push_str("    abs: Math.abs,\n");
    js.push_str("    round: Math.round,\n");
    js.push_str("    floor: Math.floor,\n");
    js.push_str("    ceil: Math.ceil\n");
    js.push_str("  };\n\n");
    
    // Initialize
    js.push_str("  // Initialize WebCore\n");
    js.push_str("  document.addEventListener('DOMContentLoaded', function() {\n");
    js.push_str("    console.log('WebCore Runtime initialized');\n");
    js.push_str("    \n");
    js.push_str("    // Initialize interpolations\n");
    js.push_str("    const interpolations = document.querySelectorAll('[data-webcore-interpolation]');\n");
    js.push_str("    interpolations.forEach(function(element) {\n");
    js.push_str("      const varName = element.getAttribute('data-webcore-interpolation');\n");
    js.push_str("      const updateText = function() {\n");
    js.push_str("        const value = window.__webcore_state__.get(varName);\n");
    js.push_str("        element.textContent = value !== undefined ? value : '';\n");
    js.push_str("      };\n");
    js.push_str("      updateText();\n");
    js.push_str("      window.__webcore_state__.subscribe(varName, updateText);\n");
    js.push_str("    });\n");
    js.push_str("    \n");
    js.push_str("    // Global HTML5 event handlers\n");
    js.push_str("    window.webcore_handle_click = function(handlerId) {\n");
    js.push_str("      if (window.__webcore_handlers__[handlerId]) {\n");
    js.push_str("        window.__webcore_handlers__[handlerId]();\n");
    js.push_str("      }\n");
    js.push_str("    };\n");
    js.push_str("    \n");
    js.push_str("    window.webcore_handle_submit = function(handlerId) {\n");
    js.push_str("      if (window.__webcore_handlers__[handlerId]) {\n");
    js.push_str("        window.__webcore_handlers__[handlerId]();\n");
    js.push_str("      }\n");
    js.push_str("    };\n");
    js.push_str("    \n");
    js.push_str("    window.webcore_handle_change = function(handlerId) {\n");
    js.push_str("      if (window.__webcore_handlers__[handlerId]) {\n");
    js.push_str("        window.__webcore_handlers__[handlerId]();\n");
    js.push_str("      }\n");
    js.push_str("    };\n");
    js.push_str("    \n");
    js.push_str("    window.webcore_handle_input = function(handlerId) {\n");
    js.push_str("      if (window.__webcore_handlers__[handlerId]) {\n");
    js.push_str("        window.__webcore_handlers__[handlerId]();\n");
    js.push_str("      }\n");
    js.push_str("    };\n");
    js.push_str("    \n");
    js.push_str("    window.webcore_handle_event = function(eventType, handlerId) {\n");
    js.push_str("      if (window.__webcore_handlers__[handlerId]) {\n");
    js.push_str("        window.__webcore_handlers__[handlerId]();\n");
    js.push_str("      }\n");
    js.push_str("    };\n");
    js.push_str("  });\n");
    
    js.push_str("})();\n");
    
    js
}

pub fn generate_component_js(component: &Component) -> String {
    let mut js = String::new();
    
    js.push_str(&format!("// Component: {}\n", component.name));
    
    // Generate state initialization
    for state_var in &component.state {
        js.push_str(&format!(
            "window.__webcore_state__.set('{}', {});\n",
            state_var.name,
            state_var.default_value.as_ref().unwrap_or(&"null".to_string())
        ));
    }
    
    js
}
