//! HTML Code Generator

use crate::ast::*;

// Options passed from the build to influence the page shell
#[derive(Debug, Clone)]
pub struct HtmlPageOptions {
    pub lang: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct HandlerMapping {
    pub id: String,
    pub event_type: String,
    pub expression: String,
}

pub struct HtmlGenerationResult {
    pub html: String,
    pub handlers: Vec<HandlerMapping>,
}

pub fn generate_html(document: &WebCoreDocument, page_name: &str, options: &HtmlPageOptions) -> Result<HtmlGenerationResult, String> {
    // Find the page
    let page = document.pages.get(page_name)
        .ok_or_else(|| format!("Page '{}' not found", page_name))?;
    
    // Find the layout (try MainLayout first, then default)
    let layout = document.layouts.get("MainLayout")
        .or_else(|| document.layouts.get("default"))
        .ok_or_else(|| "No layout found (tried MainLayout and default)".to_string())?;
    
    // Generate HTML by combining layout and page content
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n");
    html.push_str(&format!("<html lang=\"{}\">\n<head>\n", html_escape(&options.lang)));
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str(&format!("  <title>{}</title>\n", html_escape(&options.title)));
    html.push_str("  <link rel=\"stylesheet\" href=\"theme.css\">\n");
    html.push_str("</head>\n<body>\n");
    
    // Generate layout content, replacing slots with page content
    let (layout_content, handlers) = generate_layout_with_page_and_components(layout, page, document)?;
    html.push_str(&layout_content);
    
    html.push_str("  <script src=\"webcore.js\"></script>\n");
    html.push_str("</body>\n</html>");
    
    Ok(HtmlGenerationResult { html, handlers })
}

fn generate_layout_with_page(layout: &Layout, page: &Page) -> Result<String, String> {
    generate_elements_with_slot_replacement(&layout.content, &page.content)
}

fn generate_layout_with_page_and_components(layout: &Layout, page: &Page, document: &WebCoreDocument) -> Result<(String, Vec<HandlerMapping>), String> {
    generate_elements_with_slot_replacement_and_components(&layout.content, &page.content, document)
}

fn generate_elements(elements: &[Element]) -> Result<String, String> {
    let mut result = String::new();
    for element in elements {
        result.push_str(&generate_element(element)?);
    }
    Ok(result)
}

fn generate_element(element: &Element) -> Result<String, String> {
    match element {
        Element::Text(text) => Ok(html_escape(text)),
        Element::Tag { name, attributes, content } => {
            // Map custom tags
            if name == "text" {
                // Render only the inner content without wrapping tag
                return generate_elements(content);
            }

            let mapped_name = if name == "link" { "a" } else { name.as_str() };
            let is_link = mapped_name == "a";
            let mut resolved_href: Option<String> = None;

            let mut result = String::new();
            result.push_str(&format!("<{}", mapped_name));
            
            // Generate attributes
            for attr in attributes {
                match &attr.value {
                    AttributeValue::String(value) => {
                        if is_link && attr.name == "to" {
                            resolved_href = Some(value.clone());
                        } else {
                            result.push_str(&format!(" {}=\"{}\"", attr.name, html_escape(value)));
                        }
                    }
                    AttributeValue::Boolean(true) => {
                        result.push_str(&format!(" {}", attr.name));
                    }
                    AttributeValue::Boolean(false) => {}
                    AttributeValue::Expression(expr) => {
                        if attr.name.starts_with("on:") {
                            // Event handler: on:click={ count += 1 }
                            let event_type = attr.name.strip_prefix("on:").unwrap_or("click");
                            result.push_str(&format!(" data-webcore-event=\"{}\" data-webcore-handler=\"{}\"", event_type, expr));
                        } else {
                            result.push_str(&format!(" {}=\"{{}}\"", attr.name));
                        }
                    }
                }
            }

            // Default href for link→a if missing
            if is_link {
                if let Some(h) = resolved_href {
                    result.push_str(&format!(" href=\"{}\"", html_escape(&h)));
                } else if !attributes.iter().any(|a| a.name == "href") {
                    result.push_str(" href=\"#\"");
                }
            }
            
            result.push('>');
            result.push_str(&generate_elements(content)?);
            result.push_str(&format!("</{}>", mapped_name));
            Ok(result)
        }
        Element::Slot(name) => Ok(format!("<!-- Slot: {} -->", name)),
        Element::Component { name, attributes, content } => {
            let mut result = String::new();
            result.push_str(&format!("<{}", name));
            
            // Generate attributes
            for attr in attributes {
                match &attr.value {
                    AttributeValue::String(value) => {
                        result.push_str(&format!(" {}=\"{}\"", attr.name, html_escape(value)));
                    }
                    AttributeValue::Boolean(true) => {
                        result.push_str(&format!(" {}", attr.name));
                    }
                    AttributeValue::Boolean(false) => {}
                    AttributeValue::Expression(expr) => {
                        if attr.name.starts_with("on:") {
                            // Event handler: on:click={ count += 1 }
                            let event_type = attr.name.strip_prefix("on:").unwrap_or("click");
                            result.push_str(&format!(" data-webcore-event=\"{}\" data-webcore-handler=\"{}\"", event_type, expr));
                        } else {
                            result.push_str(&format!(" {}=\"{{}}\"", attr.name));
                        }
                    }
                }
            }
            
            result.push('>');
            result.push_str(&generate_elements(content)?);
            result.push_str(&format!("</{}>", name));
            Ok(result)
        }
        Element::Interpolation(expr) => {
            // Support mixed text like "prefix {var} suffix"
            if let (Some(start), Some(end)) = (expr.find('{'), expr.find('}')) {
                let prefix = html_escape(&expr[..start]);
                let var_name = expr[start + 1..end].to_string();
                let suffix = html_escape(&expr[end + 1..]);
                Ok(format!("{}<span data-webcore-interpolation=\"{}\">0</span>{}", prefix, var_name, suffix))
            } else {
                Ok(format!("<span data-webcore-interpolation=\"{}\">0</span>", html_escape(expr)))
            }
        }
    }
}

fn generate_elements_with_slot_replacement(elements: &[Element], page_content: &[Element]) -> Result<String, String> {
    let mut result = String::new();
    for element in elements {
        match element {
            Element::Slot(slot_name) => {
                if slot_name == "content" {
                    // Replace content slot with page content
                    result.push_str(&generate_elements(page_content)?);
                } else {
                    result.push_str(&format!("<!-- Slot: {} -->", slot_name));
                }
            }
            Element::Tag { name, attributes, content: _content } => {
                let mut tag_result = String::new();
                tag_result.push_str(&format!("<{}", name));
                
                // Generate attributes
                for attr in attributes {
                    match &attr.value {
                        AttributeValue::String(value) => {
                            tag_result.push_str(&format!(" {}=\"{}\"", attr.name, html_escape(value)));
                        }
                        AttributeValue::Boolean(true) => {
                            tag_result.push_str(&format!(" {}", attr.name));
                        }
                        AttributeValue::Boolean(false) => {}
                        AttributeValue::Expression(_expr) => {
                            tag_result.push_str(&format!(" {}=\"{{}}\"", attr.name));
                        }
                    }
                }
                
                tag_result.push('>');
                // This function shouldn't be called in the new system - it uses old logic
                tag_result.push_str("<!-- OLD LOGIC CALLED -->");
                tag_result.push_str(&format!("</{}>", name));
                result.push_str(&tag_result);
            }
            _ => {
                // This function shouldn't be called in the new system - it uses old logic
                result.push_str("<!-- OLD LOGIC CALLED -->");
            }
        }
    }
    Ok(result)
}

fn generate_elements_with_slot_replacement_and_components(elements: &[Element], page_content: &[Element], document: &WebCoreDocument) -> Result<(String, Vec<HandlerMapping>), String> {
    let mut result = String::new();
    let mut all_handlers = Vec::new();
    
    for element in elements {
        match element {
            Element::Slot(slot_name) => {
                if slot_name == "content" {
                    // Replace content slot with page content
                    let (content_html, content_handlers) = generate_elements_with_components(page_content, document)?;
                    result.push_str(&content_html);
                    all_handlers.extend(content_handlers);
                } else {
                    result.push_str(&format!("<!-- Slot: {} -->", slot_name));
                }
            }
            Element::Tag { name, attributes, content } => {
                let mut tag_result = String::new();
                if name == "text" {
                    // Render only inner content
                    let (content_html, content_handlers) = generate_elements_with_components(content, document)?;
                    result.push_str(&content_html);
                    all_handlers.extend(content_handlers);
                    continue;
                }

                let mapped_name = if name == "link" { "a" } else { name.as_str() };
                let is_link = mapped_name == "a";
                let mut resolved_href: Option<String> = None;
                tag_result.push_str(&format!("<{}", mapped_name));
                
                // Generate attributes
                for attr in attributes {
                    match &attr.value {
                        AttributeValue::String(value) => {
                            if is_link && attr.name == "to" {
                                resolved_href = Some(value.clone());
                            } else {
                                tag_result.push_str(&format!(" {}=\"{}\"", attr.name, html_escape(value)));
                            }
                        }
                        AttributeValue::Boolean(true) => {
                            tag_result.push_str(&format!(" {}", attr.name));
                        }
                        AttributeValue::Boolean(false) => {}
                        AttributeValue::Expression(_expr) => {
                            tag_result.push_str(&format!(" {}=\"{{}}\"", attr.name));
                        }
                    }
                }

                if is_link {
                    if let Some(h) = resolved_href {
                        tag_result.push_str(&format!(" href=\"{}\"", html_escape(&h)));
                    } else if !attributes.iter().any(|a| a.name == "href") {
                        tag_result.push_str(" href=\"#\"");
                    }
                }
                
                tag_result.push('>');
                // Inside regular tags within the layout, do not perform slot replacement again
                let (content_html, content_handlers) = generate_elements_with_components(content, document)?;
                tag_result.push_str(&content_html);
                tag_result.push_str(&format!("</{}>", mapped_name));
                result.push_str(&tag_result);
                all_handlers.extend(content_handlers);
            }
            Element::Component { name, attributes, content } => {
                // Find the component definition
                if let Some(component) = document.components.get(name) {
                    // Replace component with its view content
                    let (component_html, component_handlers) = generate_elements_with_components(&component.view, document)?;
                    result.push_str(&component_html);
                    all_handlers.extend(component_handlers);
                } else {
                    // Component not found, generate as HTML element
                    let mut comp_result = String::new();
                    comp_result.push_str(&format!("<{}", name));
                    
                    // Generate attributes
                    for attr in attributes {
                        match &attr.value {
                            AttributeValue::String(value) => {
                                comp_result.push_str(&format!(" {}=\"{}\"", attr.name, html_escape(value)));
                            }
                            AttributeValue::Boolean(true) => {
                                comp_result.push_str(&format!(" {}", attr.name));
                            }
                            AttributeValue::Boolean(false) => {}
                            AttributeValue::Expression(_expr) => {
                                comp_result.push_str(&format!(" {}=\"{{}}\"", attr.name));
                            }
                        }
                    }
                    
                    comp_result.push('>');
                    let (content_html, content_handlers) = generate_elements_with_components(content, document)?;
                    comp_result.push_str(&content_html);
                    comp_result.push_str(&format!("</{}>", name));
                    result.push_str(&comp_result);
                    all_handlers.extend(content_handlers);
                }
            }
            _ => {
                let (element_html, element_handlers) = generate_element_with_components(element, document)?;
                result.push_str(&element_html);
                all_handlers.extend(element_handlers);
            }
        }
    }
    Ok((result, all_handlers))
}

fn generate_elements_with_components(elements: &[Element], document: &WebCoreDocument) -> Result<(String, Vec<HandlerMapping>), String> {
    let mut result = String::new();
    let mut all_handlers = Vec::new();
    let mut counter = 0;
    
    for element in elements {
        let (element_html, handlers) = generate_element_with_components_counter(element, document, &mut counter)?;
        result.push_str(&element_html);
        all_handlers.extend(handlers);
    }
    Ok((result, all_handlers))
}

fn generate_element_with_components(element: &Element, document: &WebCoreDocument) -> Result<(String, Vec<HandlerMapping>), String> {
    generate_element_with_components_counter(element, document, &mut 0)
}

fn generate_element_with_components_counter(element: &Element, document: &WebCoreDocument, counter: &mut usize) -> Result<(String, Vec<HandlerMapping>), String> {
    match element {
        Element::Text(text) => Ok((html_escape(text), Vec::new())),
        Element::Tag { name, attributes, content } => {
            let mut result = String::new();
            let mut handlers = Vec::new();
            if name == "text" {
                let (content_html, content_handlers) = generate_elements_with_components(content, document)?;
                return Ok((content_html, content_handlers));
            }

            let mapped_name = if name == "link" { "a" } else { name.as_str() };
            let is_link = mapped_name == "a";
            let mut resolved_href: Option<String> = None;
            result.push_str(&format!("<{}", mapped_name));
            
            // Generate attributes
            for attr in attributes {
                match &attr.value {
                    AttributeValue::String(value) => {
                        if is_link && attr.name == "to" {
                            resolved_href = Some(value.clone());
                        } else {
                            result.push_str(&format!(" {}=\"{}\"", attr.name, html_escape(value)));
                        }
                    }
                    AttributeValue::Boolean(true) => {
                        result.push_str(&format!(" {}", attr.name));
                    }
                    AttributeValue::Boolean(false) => {}
                    AttributeValue::Expression(expr) => {
                        if attr.name.starts_with("on:") {
                            // Event handler: on:click={ count += 1 }
                            let event_type = attr.name.strip_prefix("on:").unwrap_or("click");
                            *counter += 1;
                            let handler_id = format!("btn{}", counter);
                            
                            // Add handler to our collection
                            handlers.push(HandlerMapping {
                                id: handler_id.clone(),
                                event_type: event_type.to_string(),
                                expression: expr.clone(),
                            });
                            
                            // Use native HTML5 event attributes with simple IDs
                            match event_type {
                                "click" => result.push_str(&format!(" id=\"{}\" onclick=\"webcore_handle_click('{}')\"", handler_id, handler_id)),
                                "submit" => result.push_str(&format!(" id=\"{}\" onsubmit=\"webcore_handle_submit('{}')\"", handler_id, handler_id)),
                                "change" => result.push_str(&format!(" id=\"{}\" onchange=\"webcore_handle_change('{}')\"", handler_id, handler_id)),
                                "input" => result.push_str(&format!(" id=\"{}\" oninput=\"webcore_handle_input('{}')\"", handler_id, handler_id)),
                                _ => result.push_str(&format!(" id=\"{}\" on{}=\"webcore_handle_event('{}', '{}')\"", handler_id, event_type, event_type, handler_id)),
                            }
                        } else {
                            result.push_str(&format!(" {}=\"{{}}\"", attr.name));
                        }
                    }
                }
            }

            if is_link {
                if let Some(h) = resolved_href {
                    result.push_str(&format!(" href=\"{}\"", html_escape(&h)));
                } else if !attributes.iter().any(|a| a.name == "href") {
                    result.push_str(" href=\"#\"");
                }
            }
            
            result.push('>');
            let (content_html, content_handlers) = generate_elements_with_components(content, document)?;
            result.push_str(&content_html);
            result.push_str(&format!("</{}>", mapped_name));
            handlers.extend(content_handlers);
            Ok((result, handlers))
        }
        Element::Slot(name) => Ok((format!("<!-- Slot: {} -->", name), Vec::new())),
        Element::Component { name, attributes, content } => {
            // Find the component definition
            if let Some(component) = document.components.get(name) {
                // Replace component with its view content
                generate_elements_with_components(&component.view, document)
            } else {
                // Component not found, generate as HTML element
                let mut result = String::new();
                result.push_str(&format!("<{}", name));
                
                // Generate attributes
                for attr in attributes {
                    match &attr.value {
                        AttributeValue::String(value) => {
                            result.push_str(&format!(" {}=\"{}\"", attr.name, html_escape(value)));
                        }
                        AttributeValue::Boolean(true) => {
                            result.push_str(&format!(" {}", attr.name));
                        }
                        AttributeValue::Boolean(false) => {}
                        AttributeValue::Expression(_expr) => {
                            result.push_str(&format!(" {}=\"{{}}\"", attr.name));
                        }
                    }
                }
                
                result.push('>');
                let (content_html, content_handlers) = generate_elements_with_components(content, document)?;
                result.push_str(&content_html);
                result.push_str(&format!("</{}>", name));
                Ok((result, content_handlers))
            }
        }
        Element::Interpolation(expr) => {
            // Support mixed text like "prefix {var} suffix"
            if let (Some(start), Some(end)) = (expr.find('{'), expr.find('}')) {
                let prefix = html_escape(&expr[..start]);
                let var_name = expr[start + 1..end].to_string();
                let suffix = html_escape(&expr[end + 1..]);
                Ok((format!("{}<span data-webcore-interpolation=\"{}\">0</span>{}", prefix, var_name, suffix), Vec::new()))
            } else {
                Ok((format!("<span data-webcore-interpolation=\"{}\">0</span>", html_escape(expr)), Vec::new()))
            }
        }
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_fallback_uses_on_event_attribute() {
        // Build minimal doc with a button using an unknown event
        let mut doc = WebCoreDocument {
            app: None,
            layouts: std::collections::HashMap::new(),
            pages: std::collections::HashMap::new(),
            components: std::collections::HashMap::new(),
        };
        doc.layouts.insert("MainLayout".to_string(), Layout { name: "MainLayout".to_string(), content: vec![
            Element::Slot("content".to_string())
        ]});
        doc.pages.insert("test".to_string(), Page { name: "test".to_string(), content: vec![
            Element::Tag { name: "button".to_string(), attributes: vec![
                Attribute { name: "on:foo".to_string(), value: AttributeValue::Expression("count += 1".to_string()) }
            ], content: vec![] }
        ]});

        let opts = HtmlPageOptions { lang: "fr".to_string(), title: "t".to_string() };
        let res = generate_html(&doc, "test", &opts).expect("html ok");
        assert!(res.html.contains("onfoo=\"webcore_handle_event('foo',"));
    }
}
