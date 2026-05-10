use serde::{Deserialize, Serialize};

/// Reasons for adjusting the confidence score of a conversion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    ComplexSelector,
    CascadeDependent,
    SpecificityConflict,
    MediaQueryConflict,
    UnsupportedProperty,
    UnsupportedValue,
    CssVariableUnresolved,
    KeyframesUnsupported,
    AnimationUnsupported,
    PseudoElementUnsupported,
    DynamicClassUsage,
    MultipleClassDependency,
    UnknownTailwindThemeValue,
    ParseError,
    // Add positive reasons
    AllDeclarationsMapped,
    SimpleClassSelector,
    PseudoClassHoverMapped,
}
