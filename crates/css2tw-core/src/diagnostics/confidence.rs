use crate::diagnostics::reason::Reason;

/// Represents the confidence level of a conversion from CSS to Tailwind.
#[derive(Debug, Clone)]
pub struct Confidence {
    /// A score between 0.0 and 1.0 indicating confidence.
    pub score: f32,
    /// A list of reasons explaining the score.
    pub reasons: Vec<Reason>,
}

impl Confidence {
    pub fn new() -> Self {
        Self {
            score: 0.0,
            reasons: Vec::new(),
        }
    }

    pub fn apply_modifier(&mut self, score_change: f32, reason: Reason) {
        self.score += score_change;
        self.reasons.push(reason);
    }

    pub fn clamp(&mut self) {
        self.score = self.score.clamp(0.0, 1.0);
    }
}

/// A utility to calculate confidence scores based on various factors.
pub struct ConfidenceCalculator;

impl ConfidenceCalculator {
    pub fn calculate(
        is_simple_class: bool,
        all_mapped: bool,
        is_complex_selector: bool,
        partial_mapping: bool,
        has_media_query: bool,
    ) -> Confidence {
        let mut conf = Confidence::new();

        if is_simple_class {
            conf.apply_modifier(0.30, Reason::SimpleClassSelector);
        } else if is_complex_selector {
            conf.apply_modifier(-0.30, Reason::ComplexSelector);
        }

        if all_mapped {
            conf.apply_modifier(0.20, Reason::AllDeclarationsMapped);
        }

        if partial_mapping {
            conf.apply_modifier(-0.25, Reason::UnsupportedProperty); // Representing partial mapping for now
        }

        if has_media_query {
            conf.apply_modifier(-0.20, Reason::MediaQueryConflict);
        }

        // Add other factors as needed...

        conf.clamp();
        conf
    }
}
