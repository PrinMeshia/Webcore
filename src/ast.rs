#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub states: Vec<State>,
    pub view: Option<View>,
    pub style: Option<Style>,
    pub logic: Option<Logic>,
}

#[derive(Debug, Clone)]
pub struct State {
    pub name: String,
    pub ty: Type,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub enum Type {
    String,
    Number,
    Boolean,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum Expression {
    String(String),
    Number(f64),
    Boolean(bool),
    Identifier(String),
    Interpolation(Box<Expression>),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone)]
pub struct View {
    pub elements: Vec<ViewElement>,
}

#[derive(Debug, Clone)]
pub enum ViewElement {
    HtmlTag {
        name: String,
        attributes: Vec<HtmlAttribute>,
        children: Vec<ViewElement>,
        self_closing: bool,
    },
    Text(String),
    Interpolation(Expression),
}

#[derive(Debug, Clone)]
pub struct HtmlAttribute {
    pub name: String,
    pub value: AttributeValue,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    String(String),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct Style {
    pub rules: Vec<CssRule>,
}

#[derive(Debug, Clone)]
pub struct CssRule {
    pub selector: String,
    pub properties: Vec<CssProperty>,
}

#[derive(Debug, Clone)]
pub struct CssProperty {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Logic {
    pub functions: Vec<Function>,
    pub variables: Vec<Variable>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub ty: Option<Type>,
    pub value: Expression,
}
