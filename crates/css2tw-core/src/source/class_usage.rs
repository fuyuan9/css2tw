#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct ClassUsage {
    pub class_name: String,
    pub span: Span,
}
