use crate::css::parser::TailwindMapping;
use crate::error::Css2TwError;
use crate::rewrite::patch::Replacement;
use crate::source::class_usage::ClassUsage;
use std::collections::HashMap;

/// Plans the conversion of CSS classes to Tailwind utilities.
pub struct ConversionPlanner;

impl ConversionPlanner {
    /// Generates a list of replacements for the given class usages based on the CSS rules.
    /// Only replacements with a confidence score equal to or higher than the threshold are returned.
    pub fn plan(
        classes: &[ClassUsage],
        rule_map: &HashMap<String, Vec<TailwindMapping>>,
        variable_map: &HashMap<String, String>,
        rem_scale: f32,
        confidence_threshold: f64,
    ) -> Result<Vec<Replacement>, Css2TwError> {
        let mut replacements = Vec::new();

        for usage in classes {
            if let Some(rep) = Self::explain(&usage.class_name, rule_map, variable_map, rem_scale) {
                if rep.confidence.score as f64 >= confidence_threshold {
                    let mut rep = rep;
                    rep.span = usage.span.clone();
                    replacements.push(rep);
                }
            }
        }

        Ok(replacements)
    }

    /// Evaluates a single class name and explains how it would be converted to Tailwind.
    /// Returns the proposed replacement along with a confidence report and trace.
    pub fn explain(
        class_name: &str,
        rule_map: &HashMap<String, Vec<TailwindMapping>>,
        variable_map: &HashMap<String, String>,
        rem_scale: f32,
    ) -> Option<Replacement> {
        let mappings = rule_map.get(class_name)?;
        let resolved = crate::css::resolver::ResolvedElementStyle {
            properties: mappings.clone(),
            variable_map: variable_map.clone(),
        };
        let new_classes = resolved.to_tailwind_string(rem_scale);

        if new_classes.is_empty() {
            return None;
        }

        let mut trace = Vec::new();
        trace.push(format!(
            "Found {} mappings for class .{}",
            mappings.len(),
            class_name
        ));
        trace.push(format!("Resolved Tailwind utility: {}", new_classes));

        let mut confidence_reasons = vec![crate::report::ConfidenceReason::FullMatch];
        let mut score = 1.0;

        // Check if any variables were used
        for mapping in mappings {
            let mut dest = String::new();
            let mut printer = lightningcss::printer::Printer::new(
                &mut dest,
                lightningcss::printer::PrinterOptions::default(),
            );
            if mapping.property.to_css(&mut printer, false).is_ok() {
                if dest.contains("var(") {
                    confidence_reasons
                        .push(crate::report::ConfidenceReason::VariableResolved(dest));
                    score *= 0.9;
                }
            }
        }

        Some(Replacement {
            span: crate::source::class_usage::Span { start: 0, end: 0 },
            before: class_name.to_string(),
            after: new_classes,
            confidence: crate::report::ConfidenceReport {
                score,
                reasons: confidence_reasons,
            },
            reasons: vec![],
            trace,
        })
    }
}
