use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TailwindVariant {
    #[default]
    None,
    Hover,
    Focus,
    Active,
    Before,
    After,
    Disabled,
    Checked,
    Valid,
    Invalid,
    Required,
    First,
    Last,
    Odd,
    Even,
    Placeholder,
    Marker,
    Selection,
    // Arbitrary variants for anything else
    // e.g. TailwindVariant::Arbitrary("nth-[2n]".to_string())
    Arbitrary(String),
}

impl TailwindVariant {
    pub fn to_prefix(&self) -> String {
        match self {
            TailwindVariant::None => "".to_string(),
            TailwindVariant::Hover => "hover:".to_string(),
            TailwindVariant::Focus => "focus:".to_string(),
            TailwindVariant::Active => "active:".to_string(),
            TailwindVariant::Before => "before:".to_string(),
            TailwindVariant::After => "after:".to_string(),
            TailwindVariant::Disabled => "disabled:".to_string(),
            TailwindVariant::Checked => "checked:".to_string(),
            TailwindVariant::Valid => "valid:".to_string(),
            TailwindVariant::Invalid => "invalid:".to_string(),
            TailwindVariant::Required => "required:".to_string(),
            TailwindVariant::First => "first:".to_string(),
            TailwindVariant::Last => "last:".to_string(),
            TailwindVariant::Odd => "odd:".to_string(),
            TailwindVariant::Even => "even:".to_string(),
            TailwindVariant::Placeholder => "placeholder:".to_string(),
            TailwindVariant::Marker => "marker:".to_string(),
            TailwindVariant::Selection => "selection:".to_string(),
            TailwindVariant::Arbitrary(s) => format!("{}:", s),
        }
    }
}
