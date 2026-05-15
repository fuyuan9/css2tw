use crate::error::Css2TwError;
use crate::source::{
    class_usage::{ClassUsage, Span},
    ClassUsageParser, SourceFile,
};
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Specialized parser for JSX/TSX files.
///
/// Leverages the `oxc` AST parser to accurately identify `className` and `class`
/// attributes within JSX elements, even in complex TypeScript files.
pub struct JsxParser;

/// AST visitor to find all className or class string literals in JSX.
struct ClassNameVisitor {
    classes: Vec<ClassUsage>,
}

impl<'a> Visit<'a> for ClassNameVisitor {
    fn visit_jsx_attribute(&mut self, attr: &JSXAttribute<'a>) {
        if let JSXAttributeName::Identifier(ident) = &attr.name {
            if ident.name.as_str() == "className" || ident.name.as_str() == "class" {
                if let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value {
                    let mut start = lit.span.start as usize;

                    // Lit spans include the quotes, so adjust them
                    // Since lit.value.as_str() is the content without quotes,
                    // we just advance start by 1
                    start += 1;

                    // Split by space
                    let class_names = lit.value.as_str().split_whitespace();
                    let mut current_pos = start;
                    for part in class_names {
                        let part_len = part.len();
                        let part_start = lit.value.as_str()[current_pos - start..]
                            .find(part)
                            .unwrap_or(0)
                            + current_pos;
                        let part_end = part_start + part_len;

                        self.classes.push(ClassUsage {
                            class_name: part.to_string(),
                            span: Span {
                                start: part_start,
                                end: part_end,
                            },
                        });
                        current_pos = part_end;
                    }
                }
            }
        }
    }
}

use crate::css::resolver::StyleResolver;
use crate::rewrite::patch::Replacement;
use oxc_span::GetSpan;
use scraper::{Html, Selector};

/// AST visitor to plan Tailwind conversions in JSX.
struct JsxPlanVisitor<'i, 'a> {
    replacements: Vec<Replacement>,
    resolver: &'a StyleResolver<'i, 'a>,
    rem_scale: f32,
    migrate_only_existing_classes: bool,
}

impl<'a, 'i> Visit<'a> for JsxPlanVisitor<'i, 'a> {
    fn visit_jsx_opening_element(&mut self, elem: &JSXOpeningElement<'a>) {
        let tag_name = match &elem.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            _ => "", // Ignore complex names like <MyComponent /> for now
        };

        if tag_name.is_empty() {
            oxc_ast_visit::walk::walk_jsx_opening_element(self, elem);
            return;
        }

        let mut class_attr_found = false;
        for attr in &elem.attributes {
            if let JSXAttributeItem::Attribute(attr) = attr {
                if let JSXAttributeName::Identifier(ident) = &attr.name {
                    let attr_name = ident.name.as_str();
                    if attr_name == "className" || attr_name == "class" {
                        class_attr_found = true;
                        if let Some(JSXAttributeValue::StringLiteral(lit)) = &attr.value {
                            let class_string = lit.value.as_str();
                            let start = lit.span.start as usize + 1;
                            let end = lit.span.end as usize - 1;

                            // Resolve styles using dummy context
                            let element_html =
                                format!("<{} class=\"{}\"></{}>", tag_name, class_string, tag_name);
                            let fragment = Html::parse_fragment(&element_html);
                            if let Some(el) = fragment
                                .root_element()
                                .select(&Selector::parse(tag_name).unwrap())
                                .next()
                            {
                                let resolved = self.resolver.resolve_styles(el);
                                let raw_css = resolved.get_raw_css();
                                let new_classes = resolved.to_tailwind_string(self.rem_scale);

                                if !new_classes.is_empty() || raw_css.is_some() {
                                    self.replacements.push(resolved.create_replacement(
                                        Span { start, end },
                                        class_string.to_string(),
                                        self.rem_scale,
                                        vec![
                                            "Matched JSX opening element and resolved via OXC AST"
                                                .to_string(),
                                        ],
                                    ));
                                }
                            }
                        } else if let Some(JSXAttributeValue::ExpressionContainer(expr_container)) =
                            &attr.value
                        {
                            // Task 4: Dynamic class detection
                            self.detect_dynamic_patterns(expr_container);
                        }
                    }
                }
            }
        }

        // If no className was found but we have styles, handle injection if allowed
        if !class_attr_found && !self.migrate_only_existing_classes {
            // Check if tag itself matches any styles
            let element_html = format!("<{}></{}>", tag_name, tag_name);
            let fragment = Html::parse_fragment(&element_html);
            if let Some(el) = fragment
                .root_element()
                .select(&Selector::parse(tag_name).unwrap())
                .next()
            {
                let resolved = self.resolver.resolve_styles(el);
                let new_classes = resolved.to_tailwind_string(self.rem_scale);
                if !new_classes.is_empty() {
                    let insert_pos = elem.name.span().end as usize;
                    let mut rep = resolved.create_replacement(
                        Span {
                            start: insert_pos,
                            end: insert_pos,
                        },
                        "".to_string(),
                        self.rem_scale,
                        vec!["Injected new className for JSX element".to_string()],
                    );
                    rep.after = format!(" className=\"{}\"", new_classes);
                    self.replacements.push(rep);
                }
            }
        }

        // Continue visiting children
        oxc_ast_visit::walk::walk_jsx_opening_element(self, elem);
    }
}

impl<'a, 'i> JsxPlanVisitor<'i, 'a> {
    fn detect_dynamic_patterns(&mut self, expr_container: &JSXExpressionContainer<'a>) {
        match &expr_container.expression {
            JSXExpression::CallExpression(call) => {
                if let Expression::Identifier(ident) = &call.callee {
                    let name = ident.name.as_str();
                    if name == "clsx" || name == "classNames" || name == "cva" {
                        self.add_dynamic_replacement(
                            call.span(),
                            format!("Dynamic class library detected: {}", name),
                            crate::report::FailureReason::DynamicClass,
                        );
                    }
                }
            }
            JSXExpression::TemplateLiteral(lit) => {
                self.add_dynamic_replacement(
                    lit.span(),
                    "Dynamic template literal detected in className".to_string(),
                    crate::report::FailureReason::DynamicTemplateLiteral,
                );
            }
            JSXExpression::ConditionalExpression(cond) => {
                self.add_dynamic_replacement(
                    cond.span(),
                    "Conditional JSX className detected".to_string(),
                    crate::report::FailureReason::DynamicClass,
                );
            }
            JSXExpression::ArrayExpression(arr) => {
                self.add_dynamic_replacement(
                    arr.span(),
                    "Array join pattern detected in className".to_string(),
                    crate::report::FailureReason::DynamicClass,
                );
            }
            JSXExpression::EmptyExpression(_) => {}
            _ => {
                // General dynamic expression
                self.add_dynamic_replacement(
                    expr_container.expression.span(),
                    "Complex dynamic expression detected in className".to_string(),
                    crate::report::FailureReason::DynamicClass,
                );
            }
        }
    }

    fn add_dynamic_replacement(
        &mut self,
        span: oxc_span::Span,
        message: String,
        reason: crate::report::FailureReason,
    ) {
        self.replacements.push(Replacement {
            span: Span {
                start: span.start as usize,
                end: span.end as usize,
            },
            before: "".to_string(), // We don't have a single "before" string for complex exprs
            after: "".to_string(),
            confidence: crate::report::ConfidenceReport {
                score: 0.0,
                reasons: vec![],
            },
            reasons: vec![message.clone()],
            trace: vec![format!("Dynamic detection: {}", message)],
            raw_css: None,
            suggestion: Some("Manually migrate dynamic classes to Tailwind utilities.".to_string()),
            failure_reason: Some(reason),
            diagnostics: vec![crate::report::Diagnostic {
                severity: crate::report::Severity::Warning,
                recoverable: false,
                manual_action_required: true,
                reason: format!("{:?}", reason),
                message,
            }],
        });
    }
}

impl JsxParser {
    /// Plans the conversion of a JSX file by performing a full AST traversal.
    ///
    /// This visits each JSX element, resolves its styles against the CSS rules,
    /// and generates precise replacements for the `className` attribute values.
    pub fn plan_jsx(
        &self,
        source: &SourceFile,
        style_rules: &[crate::css::parser::RuleWithContext],
        rem_scale: f32,
        migrate_only_existing_classes: bool,
        include_tag_selectors: bool,
    ) -> Result<Vec<Replacement>, Css2TwError> {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(&source.path)
            .unwrap_or_default()
            .with_jsx(true);
        let ret = Parser::new(&allocator, &source.content, source_type).parse();

        let resolver = StyleResolver::new(style_rules, include_tag_selectors);
        let mut visitor = JsxPlanVisitor {
            replacements: Vec::new(),
            resolver: &resolver,
            rem_scale,
            migrate_only_existing_classes,
        };
        visitor.visit_program(&ret.program);

        Ok(visitor.replacements)
    }
}

impl ClassUsageParser for JsxParser {
    fn extract_classes(&self, source: &SourceFile) -> Result<Vec<ClassUsage>, Css2TwError> {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(&source.path)
            .unwrap_or_default()
            .with_jsx(true);
        let ret = Parser::new(&allocator, &source.content, source_type).parse();

        let mut visitor = ClassNameVisitor {
            classes: Vec::new(),
        };
        visitor.visit_program(&ret.program);

        Ok(visitor.classes)
    }
}
