/// Represents a range of bytes in a source file.
#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Represents the usage of a CSS class name at a specific location in a source file.
#[derive(Debug, Clone)]
pub struct ClassUsage {
    pub class_name: String,
    pub span: Span,
}
