//! Parser for .webc files (MVP version)

use crate::ast::*;
use std::collections::HashMap;

pub fn parse_webc(source: &str) -> Result<WebCoreDocument, ParseError> {
    let mut parser = Parser::new(source);
    parser.parse_document()
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    ExpectedToken(String),
    InvalidSyntax(String),
}

pub struct Parser {
    source: String,
    pos: usize,
    tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub enum Token {
    Identifier(String),
    String(String),
    Number(String),
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Colon,
    Equals,
    Comma,
    Dot,
    Arrow, // =>
    Plus,
    Minus,
    Eof,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let tokens = Self::tokenize(source);
        Self {
            source: source.to_string(),
            pos: 0,
            tokens,
        }
    }

    fn tokenize(source: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = source.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '{' => tokens.push(Token::LeftBrace),
                '}' => tokens.push(Token::RightBrace),
                '(' => tokens.push(Token::LeftParen),
                ')' => tokens.push(Token::RightParen),
                ':' => tokens.push(Token::Colon),
                '=' => {
                    if chars.peek() == Some(&'>') {
                        chars.next();
                        tokens.push(Token::Arrow);
                    } else {
                        tokens.push(Token::Equals);
                    }
                }
                ',' => tokens.push(Token::Comma),
                '.' => tokens.push(Token::Dot),
                '+' => tokens.push(Token::Plus),
                '-' => tokens.push(Token::Minus),
                '"' => {
                    let mut string = String::new();
                    while let Some(c) = chars.next() {
                        if c == '"' {
                            break;
                        }
                        string.push(c);
                    }
                    tokens.push(Token::String(string));
                }
                c if c.is_alphabetic() || c == '_' => {
                    let mut ident = String::new();
                    ident.push(c);
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' || c == '-' || c == ':' {
                            ident.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Identifier(ident));
                }
                c if c.is_numeric() => {
                    let mut number = String::new();
                    number.push(c);
                    while let Some(&c) = chars.peek() {
                        if c.is_numeric() || c == '.' {
                            number.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(number));
                }
                c if c.is_whitespace() => continue,
                _ => {} // Ignore other characters for now
            }
        }
        tokens.push(Token::Eof);
        tokens
    }

    fn current_token(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if std::mem::discriminant(self.current_token()) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::ExpectedToken(format!("{:?}", expected)))
        }
    }

    pub fn parse_document(&mut self) -> Result<WebCoreDocument, ParseError> {
        let mut app = None;
        let mut layouts = HashMap::new();
        let mut pages = HashMap::new();
        let mut components = HashMap::new();

        while !matches!(self.current_token(), Token::Eof) {
            match self.current_token() {
                Token::Identifier(ref name) if name == "app" => {
                    app = Some(self.parse_app()?);
                }
                Token::Identifier(ref name) if name == "layout" => {
                    let layout = self.parse_layout()?;
                    layouts.insert(layout.name.clone(), layout);
                }
                Token::Identifier(ref name) if name == "page" => {
                    let page = self.parse_page()?;
                    pages.insert(page.name.clone(), page);
                }
                Token::Identifier(ref name) if name == "component" => {
                    let component = self.parse_component()?;
                    components.insert(component.name.clone(), component);
                }
                _ => {
                    // Try to parse as a simple element
                    let element = self.parse_element()?;
                    // For now, create a default page with this element
                    let page = Page {
                        name: "default".to_string(),
                        content: vec![element],
                    };
                    pages.insert("default".to_string(), page);
                }
            }
        }

        Ok(WebCoreDocument {
            app,
            layouts,
            pages,
            components,
        })
    }

    fn parse_app(&mut self) -> Result<App, ParseError> {
        self.expect(Token::Identifier("app".to_string()))?;
        
        let name = match self.current_token() {
            Token::Identifier(ref name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(ParseError::ExpectedToken("app name".to_string())),
        };

        self.expect(Token::LeftBrace)?;
        
        let mut theme = None;
        let mut layout = None;
        let mut routes = HashMap::new();
        
        while !matches!(self.current_token(), Token::RightBrace) {
            match self.current_token() {
                Token::Identifier(ref key) => {
                    let key = key.clone();
                    self.advance();
                    
                    if key == "theme" {
                        self.expect(Token::Colon)?;
                        theme = Some(match self.current_token() {
                            Token::String(ref value) => {
                                let value = value.clone();
                                self.advance();
                                value
                            }
                            _ => return Err(ParseError::ExpectedToken("theme name".to_string())),
                        });
                    } else if key == "layout" {
                        self.expect(Token::Colon)?;
                        layout = Some(match self.current_token() {
                            Token::Identifier(ref value) => {
                                let value = value.clone();
                                self.advance();
                                value
                            }
                            _ => return Err(ParseError::ExpectedToken("layout name".to_string())),
                        });
                    } else if key == "routes" {
                        self.expect(Token::LeftBrace)?;
                        while !matches!(self.current_token(), Token::RightBrace) {
                            let route_path = match self.current_token() {
                                Token::String(ref path) => {
                                    let path = path.clone();
                                    self.advance();
                                    path
                                }
                                _ => return Err(ParseError::ExpectedToken("route path".to_string())),
                            };
                            self.expect(Token::Colon)?;
                            let component_name = match self.current_token() {
                                Token::Identifier(ref name) => {
                                    let name = name.clone();
                                    self.advance();
                                    name
                                }
                                _ => return Err(ParseError::ExpectedToken("component name".to_string())),
                            };
                            routes.insert(route_path, component_name);
                        }
                        self.expect(Token::RightBrace)?;
                    }
                }
                _ => return Err(ParseError::UnexpectedToken(format!("{:?}", self.current_token()))),
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(App {
            name,
            theme,
            layout,
            routes,
        })
    }

    fn parse_layout(&mut self) -> Result<Layout, ParseError> {
        self.expect(Token::Identifier("layout".to_string()))?;
        
        let name = match self.current_token() {
            Token::Identifier(ref name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(ParseError::ExpectedToken("layout name".to_string())),
        };

        self.expect(Token::LeftBrace)?;
        
        let mut content = Vec::new();
        while !matches!(self.current_token(), Token::RightBrace) {
            content.push(self.parse_element()?);
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(Layout { name, content })
    }

    fn parse_page(&mut self) -> Result<Page, ParseError> {
        self.expect(Token::Identifier("page".to_string()))?;
        
        let name = match self.current_token() {
            Token::String(ref name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(ParseError::ExpectedToken("page name".to_string())),
        };

        self.expect(Token::LeftBrace)?;
        
        let mut content = Vec::new();
        while !matches!(self.current_token(), Token::RightBrace) {
            content.push(self.parse_element()?);
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(Page { name, content })
    }

    fn parse_component(&mut self) -> Result<Component, ParseError> {
        self.expect(Token::Identifier("component".to_string()))?;
        
        let name = match self.current_token() {
            Token::Identifier(ref name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(ParseError::ExpectedToken("component name".to_string())),
        };

        self.expect(Token::LeftBrace)?;
        
        let mut props = Vec::new();
        let mut state = Vec::new();
        let mut view = Vec::new();
        let mut style = Vec::new();
        
        while !matches!(self.current_token(), Token::RightBrace) {
            match self.current_token() {
                Token::Identifier(ref section) => {
                    let section = section.clone();
                    self.advance();
                    
                    if section == "props" {
                        self.expect(Token::LeftBrace)?;
                        while !matches!(self.current_token(), Token::RightBrace) {
                            let prop_name = match self.current_token() {
                                Token::Identifier(ref name) => {
                                    let name = name.clone();
                                    self.advance();
                                    name
                                }
                                _ => return Err(ParseError::ExpectedToken("prop name".to_string())),
                            };
                            let prop_type = if matches!(self.current_token(), Token::Colon) {
                                self.advance();
                                Some(match self.current_token() {
                                    Token::Identifier(ref type_) => {
                                        let type_ = type_.clone();
                                        self.advance();
                                        type_
                                    }
                                    _ => return Err(ParseError::ExpectedToken("prop type".to_string())),
                                })
                            } else {
                                None
                            };
                            props.push(Prop { name: prop_name, type_: prop_type });
                        }
                        self.expect(Token::RightBrace)?;
                    } else if section == "state" {
                        self.expect(Token::LeftBrace)?;
                        while !matches!(self.current_token(), Token::RightBrace) {
                            let state_name = match self.current_token() {
                                Token::Identifier(ref name) => {
                                    let name = name.clone();
                                    self.advance();
                                    name
                                }
                                _ => return Err(ParseError::ExpectedToken("state name".to_string())),
                            };
                            self.expect(Token::Colon)?;
                            let state_type = match self.current_token() {
                                Token::Identifier(ref type_) => {
                                    let type_ = type_.clone();
                                    self.advance();
                                    type_
                                }
                                _ => return Err(ParseError::ExpectedToken("state type".to_string())),
                            };
                            let default_value = if matches!(self.current_token(), Token::Equals) {
                                self.advance();
                                Some(match self.current_token() {
                                    Token::Number(ref value) => {
                                        let value = value.clone();
                                        self.advance();
                                        value
                                    }
                                    Token::String(ref value) => {
                                        let value = value.clone();
                                        self.advance();
                                        value
                                    }
                                    _ => return Err(ParseError::ExpectedToken("default value".to_string())),
                                })
                            } else {
                                None
                            };
                            state.push(StateVar { name: state_name, type_: state_type, default_value });
                        }
                        self.expect(Token::RightBrace)?;
                    } else if section == "view" {
                        self.expect(Token::LeftBrace)?;
                        while !matches!(self.current_token(), Token::RightBrace) {
                            view.push(self.parse_element()?);
                        }
                        self.expect(Token::RightBrace)?;
                    } else if section == "style" {
                        self.expect(Token::LeftBrace)?;
                        while !matches!(self.current_token(), Token::RightBrace) {
                            style.push(self.parse_style_rule()?);
                        }
                        self.expect(Token::RightBrace)?;
                    }
                }
                _ => {
                    // Fallback to old behavior for simple components
                    view.push(self.parse_element()?);
                }
            }
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(Component { 
            name, 
            props, 
            state, 
            view, 
            style 
        })
    }

    fn parse_style_rule(&mut self) -> Result<StyleRule, ParseError> {
        let selector = match self.current_token() {
            Token::Identifier(ref selector) => {
                let selector = selector.clone();
                self.advance();
                selector
            }
            _ => return Err(ParseError::ExpectedToken("style selector".to_string())),
        };
        
        self.expect(Token::LeftBrace)?;
        
        let mut properties = Vec::new();
        while !matches!(self.current_token(), Token::RightBrace) {
            let prop_name = match self.current_token() {
                Token::Identifier(ref name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => return Err(ParseError::ExpectedToken("property name".to_string())),
            };
            self.expect(Token::Colon)?;
            let prop_value = match self.current_token() {
                Token::String(ref value) => {
                    let value = value.clone();
                    self.advance();
                    value
                }
                Token::Identifier(ref value) => {
                    let value = value.clone();
                    self.advance();
                    value
                }
                _ => return Err(ParseError::ExpectedToken("property value".to_string())),
            };
            properties.push(StyleProperty { name: prop_name, value: prop_value });
        }
        
        self.expect(Token::RightBrace)?;
        
        Ok(StyleRule { selector, properties })
    }

    fn parse_element(&mut self) -> Result<Element, ParseError> {
        match self.current_token() {
            Token::Identifier(ref tag_name) => {
                let tag_name = tag_name.clone();
                self.advance();
                
                // Check if it's a slot
                if tag_name == "slot" {
                    let slot_name = match self.current_token() {
                        Token::Identifier(ref name) => {
                            let name = name.clone();
                            self.advance();
                            name
                        }
                        _ => "content".to_string(),
                    };
                    return Ok(Element::Slot(slot_name));
                }
                
                // Parse attributes
                let mut attributes = Vec::new();
                while !matches!(self.current_token(), Token::LeftBrace) && !matches!(self.current_token(), Token::String(_)) && !matches!(self.current_token(), Token::Eof) {
                    // Check if the next token is an identifier that could be a sibling element
                    let current_pos = self.pos;
                    let next_token = if current_pos + 1 < self.tokens.len() {
                        &self.tokens[current_pos + 1]
                    } else {
                        &Token::Eof
                    };
                    
                    // If we see an identifier followed by a left brace, it's likely a sibling element
                    if matches!(self.current_token(), Token::Identifier(_)) && matches!(next_token, Token::LeftBrace) {
                        break;
                    }
                    let attr_name = match self.current_token() {
                        Token::Identifier(ref name) => {
                            let name = name.clone();
                            self.advance();
                            name
                        }
                        _ => break,
                    };
                    
                    if matches!(self.current_token(), Token::Equals) {
                        self.advance();
                        let attr_value = match self.current_token() {
                            Token::String(ref value) => {
                                let value = value.clone();
                                self.advance();
                                AttributeValue::String(value)
                            }
                            Token::LeftBrace => {
                                self.advance();
                                let mut expr = String::new();
                                while !matches!(self.current_token(), Token::RightBrace) {
                                    match self.current_token() {
                                        Token::Identifier(ref id) => expr.push_str(id),
                                        Token::Number(ref num) => expr.push_str(num),
                                        Token::Plus => expr.push('+'),
                                        Token::Minus => expr.push('-'),
                                        Token::Equals => expr.push('='),
                                        Token::LeftParen => expr.push('('),
                                        Token::RightParen => expr.push(')'),
                                        Token::Comma => expr.push(','),
                                        Token::Dot => expr.push('.'),
                                        Token::Arrow => expr.push_str("=>"),
                                        _ => expr.push(' '),
                                    }
                                    self.advance();
                                }
                                self.advance();
                                AttributeValue::Expression(expr)
                            }
                            _ => AttributeValue::Boolean(true),
                        };
                        attributes.push(Attribute { name: attr_name, value: attr_value });
                    } else {
                        attributes.push(Attribute { name: attr_name, value: AttributeValue::Boolean(true) });
                    }
                }
                
                // Check if there's content (string or nested elements)
                let content = if matches!(self.current_token(), Token::String(_)) {
                    // Simple text content (with possible interpolations)
                    let text = match self.current_token() {
                        Token::String(ref text) => {
                            let text = text.clone();
                            self.advance();
                            text
                        }
                        _ => return Err(ParseError::ExpectedToken("text content".to_string())),
                    };
                    // Split mixed text with interpolations into multiple elements
                    split_interpolated_text(&text)
                } else if matches!(self.current_token(), Token::LeftBrace) {
                    // Nested elements
                    self.expect(Token::LeftBrace)?;
                    let mut nested = Vec::new();
                    while !matches!(self.current_token(), Token::RightBrace) {
                        nested.push(self.parse_element()?);
                    }
                    self.expect(Token::RightBrace)?;
                    nested
                } else {
                    // Self-closing element or element without braces
                    Vec::new()
                };
                
                // Check if it's a component (capitalized) or regular tag
                if tag_name.chars().next().map_or(false, |c| c.is_uppercase()) {
                    Ok(Element::Component {
                        name: tag_name,
                        attributes,
                        content,
                    })
                } else {
                    Ok(Element::Tag {
                        name: tag_name,
                        attributes,
                        content,
                    })
                }
            }
            Token::String(ref text) => {
                let text = text.clone();
                self.advance();
                // Fallback: if there is a single interpolation with no surrounding text, return it
                if text.starts_with('{') && text.ends_with('}') && text.len() >= 2 {
                    let var = text[1..text.len()-1].to_string();
                    Ok(Element::Interpolation(var))
                } else {
                    Ok(Element::Text(text))
                }
            }
            _ => Err(ParseError::UnexpectedToken(format!("{:?}", self.current_token()))),
        }
    }
}

// Split a string potentially containing multiple {var} interpolations into a sequence of Elements
fn split_interpolated_text(text: &str) -> Vec<Element> {
    let mut elements: Vec<Element> = Vec::new();
    let mut i = 0usize;
    let bytes = text.as_bytes();
    let len = bytes.len();
    while i < len {
        // find next '{'
        if let Some(start) = text[i..].find('{') {
            let start_idx = i + start;
            // push prefix text if any
            if start_idx > i {
                elements.push(Element::Text(text[i..start_idx].to_string()));
            }
            // find matching '}' after '{'
            if let Some(end) = text[start_idx..].find('}') {
                let end_idx = start_idx + end;
                let var_name = text[start_idx + 1..end_idx].trim().to_string();
                elements.push(Element::Interpolation(var_name));
                i = end_idx + 1; // move after '}'
            } else {
                // no closing brace, treat rest as text
                elements.push(Element::Text(text[start_idx..].to_string()));
                break;
            }
        } else {
            // no more '{'
            elements.push(Element::Text(text[i..].to_string()));
            break;
        }
    }
    if elements.is_empty() {
        elements.push(Element::Text(text.to_string()));
    }
    elements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_mixed_text_and_interpolation() {
        let src = r#"
component CounterPage {
  view {
    p "Nombre de clics: {count}"
  }
}
"#;
        let doc = parse_webc(src).expect("parse ok");
        let comp = doc.components.get("CounterPage").expect("component exists");
        // Expect: p tag with children [Text("Nombre de clics: "), Interpolation("count")]
        match &comp.view[0] {
            Element::Tag { name, content, .. } => {
                assert_eq!(name, "p");
                assert!(matches!(content.get(0), Some(Element::Text(t)) if t == "Nombre de clics: "));
                assert!(matches!(content.get(1), Some(Element::Interpolation(v)) if v == "count"));
            }
            _ => panic!("expected p tag"),
        }
    }
}
