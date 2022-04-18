use std::boxed::Box;

#[derive(Debug)]
pub enum Expression<'a> {
    SelectionSet(SelectionSet<'a>),
}

#[derive(Debug)]
pub enum SelectionSet<'a> {
    Singular(SingularSelectionSet<'a>),
    Multi(Vec<SingularSelectionSet<'a>>),
}

#[derive(Debug)]
pub enum SingularSelectionSet<'a> {
    Explicit(Field<'a>, Box<Option<Expression<'a>>>),
    Wildcard,
}

#[derive(Debug)]
pub struct Field<'a> {
    pub name: &'a str,
    pub args: Option<Vec<FieldArg<'a>>>,
}

#[derive(Debug)]
pub struct FieldArg<'a> {
    pub name: &'a str,
    pub value: FieldArgValue,
}

#[derive(Debug)]
pub enum FieldArgValue {
    StringLiteral(String),
    NumericLiteral(f64),
    BoolLiteral(bool),
    Wildcard,
}
