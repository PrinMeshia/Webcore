#[derive(Debug)]
pub struct Component {
    pub name: String,
    pub states: Vec<State>,
    pub view: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug)]
pub struct State {
    pub name: String,
    pub ty: String,
    pub value: String,
}
