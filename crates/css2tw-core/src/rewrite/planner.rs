use std::collections::HashMap;
use crate::source::class_usage::ClassUsage;
use crate::rewrite::patch::Replacement;
use crate::error::Css2TwError;
use crate::css::parser::TailwindMapping;

pub struct ConversionPlanner;

impl ConversionPlanner {
    pub fn plan(
        classes: &[ClassUsage],
        rule_map: &HashMap<String, Vec<TailwindMapping>>,
        rem_scale: f32,
        confidence_threshold: f64,
    ) -> Result<Vec<Replacement>, Css2TwError> {
        let mut replacements = Vec::new();
        
        for usage in classes {
            if let Some(rep) = Self::explain(&usage.class_name, rule_map, rem_scale) {
                if rep.confidence as f64 >= confidence_threshold {
                    let mut rep = rep;
                    rep.span = usage.span.clone();
                    replacements.push(rep);
                }
            }
        }

        Ok(replacements)
    }

    pub fn explain(
        class_name: &str,
        rule_map: &HashMap<String, Vec<TailwindMapping>>,
        rem_scale: f32,
    ) -> Option<Replacement> {
        let mappings = rule_map.get(class_name)?;
        let resolved = crate::css::resolver::ResolvedElementStyle {
            properties: mappings.clone(),
        };
        let new_classes = resolved.to_tailwind_string(rem_scale);
        
        if new_classes.is_empty() {
            return None;
        }

        let mut trace = Vec::new();
        trace.push(format!("Found {} mappings for class .{}", mappings.len(), class_name));
        trace.push(format!("Resolved Tailwind utility: {}", new_classes));
        
        Some(Replacement {
            span: crate::source::class_usage::Span { start: 0, end: 0 }, 
            before: class_name.to_string(),
            after: new_classes,
            confidence: 1.0,
            reasons: vec![],
            trace,
        })
    }
}
