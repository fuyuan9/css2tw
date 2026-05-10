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
            if let Some(mappings) = rule_map.get(&usage.class_name) {
                let resolved = crate::css::resolver::ResolvedElementStyle {
                    properties: mappings.clone(),
                };
                let new_classes = resolved.to_tailwind_string(rem_scale);

                if !new_classes.is_empty() {
                    let confidence = 1.0; 

                    if confidence >= confidence_threshold {
                        let mut trace = Vec::new();
                        trace.push(format!("Found {} mappings for class .{}", mappings.len(), usage.class_name));
                        trace.push(format!("Resolved Tailwind utility: {}", new_classes));
                        
                        replacements.push(Replacement {
                            span: usage.span.clone(),
                            before: usage.class_name.clone(),
                            after: new_classes,
                            confidence: confidence as f32,
                            reasons: vec![],
                            trace,
                        });
                    }
                }
            }
        }

        Ok(replacements)
    }
}
