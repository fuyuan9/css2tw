use lightningcss::selector::{Component, Selector};

pub enum SelectorType {
    /// A single class selector, e.g., `.btn`
    SimpleClass(String),
    /// A single class selector with pseudo-classes, e.g., `.btn:hover`
    ClassWithPseudo(String, Vec<String>),
    /// Anything else, e.g., `.btn .icon`, `div.btn`
    Complex,
}

pub fn classify_selector(selector: &Selector) -> SelectorType {
    let mut class_name = None;
    let mut pseudo_classes = Vec::new();
    let mut is_complex = false;

    // A selector consists of iter_raw_match_order.
    // If we see Combinators or non-class/pseudo-class components, it's complex.
    for component in selector.iter_raw_match_order() {
        match component {
            Component::Class(c) => {
                if class_name.is_some() {
                    is_complex = true;
                } else {
                    class_name = Some(c.0.to_string());
                }
            }
            Component::NonTSPseudoClass(p) => {
                use cssparser::ToCss;
                let mut css = String::new();
                if p.to_css(&mut css).is_ok() {
                    pseudo_classes.push(css);
                }
            }
            // Add other pseudo classes/elements if needed
            _ => {
                is_complex = true;
            }
        }
    }

    if is_complex {
        return SelectorType::Complex;
    }

    if let Some(name) = class_name {
        if pseudo_classes.is_empty() {
            SelectorType::SimpleClass(name)
        } else {
            SelectorType::ClassWithPseudo(name, pseudo_classes)
        }
    } else {
        SelectorType::Complex
    }
}

