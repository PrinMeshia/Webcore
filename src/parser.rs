use pest::Parser;
use pest_derive::Parser;

use crate::ast::{Component, State};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct WebcParser;

pub fn parse_component(src: &str) -> Result<Vec<Component>, String> {
    let file = WebcParser::parse(Rule::file, src)
        .map_err(|e| format!("Parse error: {}", e))?;

    let mut components = Vec::new();

    for pair in file {
        match pair.as_rule() {
            Rule::component => {
                let mut name = String::new();
                let mut states = Vec::new();
                let mut view = None;
                let mut style = None;

                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::identifier => {
                            name = inner.as_str().to_string();
                        }
                        Rule::state_decl => {
                            let mut parts = inner.into_inner();
                            let s_name = parts.next().unwrap().as_str().to_string();
                            let s_ty = parts.next().unwrap().as_str().to_string();
                            let s_val = parts.next().unwrap().as_str().trim().to_string();
                            states.push(State { name: s_name, ty: s_ty, value: s_val });
                        }
                        Rule::view_decl => {
                            view = Some(inner.as_str().to_string());
                        }
                        Rule::style_decl => {
                            style = Some(inner.as_str().to_string());
                        }
                        _ => {}
                    }
                }

                components.push(Component { name, states, view, style });
            }
            _ => {}
        }
    }

    Ok(components)
}
