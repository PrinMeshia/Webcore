//! AST definition for WebCore

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WebCoreDocument {
    pub app: Option<App>,
    pub layouts: HashMap<String, Layout>,
    pub pages: HashMap<String, Page>,
    pub components: HashMap<String, Component>,
}

#[derive(Debug, Clone)]
pub struct App {
    pub name: String,
    pub theme: Option<String>,
    pub layout: Option<String>,
    pub routes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub name: String,
    pub content: Vec<Element>,
}

#[derive(Debug, Clone)]
pub struct Page {
    pub name: String,
    pub content: Vec<Element>,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub props: Vec<Prop>,
    pub state: Vec<StateVar>,
    pub view: Vec<Element>,
    pub style: Vec<StyleRule>,
}

#[derive(Debug, Clone)]
pub struct Prop {
    pub name: String,
    pub type_: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StateVar {
    pub name: String,
    pub type_: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Element {
    Text(String),
    Tag {
        name: String,
        attributes: Vec<Attribute>,
        content: Vec<Element>,
    },
    Slot(String),
    Component {
        name: String,
        attributes: Vec<Attribute>,
        content: Vec<Element>,
    },
    Interpolation(String),
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Expression(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector: String,
    pub properties: Vec<StyleProperty>,
}

#[derive(Debug, Clone)]
pub struct StyleProperty {
    pub name: String,
    pub value: String,
}
