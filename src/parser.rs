use pest::Parser;
use pest_derive::Parser;
use pest::iterators::Pair;

use crate::ast::*;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct WebcParser;

pub fn parse_components(src: &str) -> Result<Vec<Component>, String> {
    let file = WebcParser::parse(Rule::file, src)
        .map_err(|e| format!("Parse error: {}", e))?;

    let mut components = Vec::new();

    for pair in file {
        match pair.as_rule() {
            Rule::component => {
                let component = parse_component(pair)?;
                components.push(component);
            }
            Rule::file => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::component => {
                            let component = parse_component(inner_pair)?;
                            components.push(component);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    Ok(components)
}

fn parse_component(pair: Pair<Rule>) -> Result<Component, String> {
    let mut inner = pair.into_inner();
    
    // Le premier élément est le nom du composant
    let name = inner.next()
        .ok_or("Missing component name")?
        .as_str()
        .to_string();

    let mut states = Vec::new();
    let mut view = None;
    let mut style = None;
    let logic = None;

    for p in inner {
        match p.as_rule() {
            Rule::state_decl => {
                let state = parse_state_decl(p)?;
                states.push(state);
            }
            Rule::view_decl => {
                view = Some(parse_view_decl(p)?);
            }
            Rule::style_decl => {
                style = Some(parse_style_decl(p)?);
            }
            _ => {}
        }
    }

    Ok(Component { name, states, view, style, logic })
}

fn parse_state_decl(pair: Pair<Rule>) -> Result<State, String> {
    let mut inner = pair.into_inner();
    
    let name = inner.next()
        .ok_or("Missing state name")?
        .as_str()
        .to_string();
    
    let ty = parse_type_annotation(inner.next().ok_or("Missing state type")?)?;
    let value = parse_expression(inner.next().ok_or("Missing state value")?)?;

    Ok(State { name, ty, value })
}

fn parse_type_annotation(pair: Pair<Rule>) -> Result<Type, String> {
    let mut inner = pair.into_inner();
    let type_str = inner.next()
        .ok_or("Missing type")?
        .as_str();

    match type_str {
        "String" => Ok(Type::String),
        "Number" => Ok(Type::Number),
        "Boolean" => Ok(Type::Boolean),
        _ => Ok(Type::Custom(type_str.to_string())),
    }
}

fn parse_expression(pair: Pair<Rule>) -> Result<Expression, String> {
    match pair.as_rule() {
        Rule::string_literal => {
            let content = pair.as_str();
            // Enlever les guillemets
            let content = &content[1..content.len()-1];
            Ok(Expression::String(content.to_string()))
        }
        Rule::identifier => {
            Ok(Expression::Identifier(pair.as_str().to_string()))
        }
        _ => Err(format!("Unexpected expression rule: {:?}", pair.as_rule())),
    }
}

fn parse_view_decl(_pair: Pair<Rule>) -> Result<View, String> {
    // Pour l'instant, on retourne une vue vide
    // TODO: Parser la vue correctement
    Ok(View { elements: Vec::new() })
}


fn parse_style_decl(_pair: Pair<Rule>) -> Result<Style, String> {
    // Pour l'instant, on retourne un style vide
    // TODO: Parser le CSS correctement
    Ok(Style { rules: Vec::new() })
}

