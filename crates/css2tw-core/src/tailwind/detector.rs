use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

/// Result of Tailwind configuration detection.
#[derive(Debug, Serialize, Default)]
pub struct ConfigDetectionResult {
    pub file_found: Option<String>,
    pub spacing: HashMap<String, String>,
    pub colors: HashMap<String, String>,
    pub screens: HashMap<String, String>,
    pub font_family: HashMap<String, Vec<String>>,
    pub arbitrary_utilities: bool,
}

pub struct ConfigDetector;

impl ConfigDetector {
    /// Attempts to find and analyze a Tailwind configuration file in the given directory.
    pub fn detect(path: &Path) -> ConfigDetectionResult {
        let mut result = ConfigDetectionResult::default();

        let config_files = [
            "tailwind.config.js",
            "tailwind.config.ts",
            "tailwind.config.mjs",
            "tailwind.config.cjs",
        ];

        for file in config_files {
            let config_path = path.join(file);
            if config_path.exists() {
                result.file_found = Some(file.to_string());
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    Self::analyze_content(&content, &config_path, &mut result);
                }
                break;
            }
        }

        result
    }

    fn analyze_content(content: &str, path: &Path, result: &mut ConfigDetectionResult) {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap_or_default();
        let ret = Parser::new(&allocator, content, source_type).parse();

        let mut visitor = ConfigVisitor {
            result,
            _marker: std::marker::PhantomData,
        };
        use oxc_ast_visit::Visit;
        visitor.visit_program(&ret.program);
    }

    fn parse_config_object(obj: &ObjectExpression, result: &mut ConfigDetectionResult) {
        for prop in &obj.properties {
            if let ObjectPropertyKind::ObjectProperty(p) = prop {
                let key = match &p.key {
                    PropertyKey::StaticIdentifier(ident) => ident.name.to_string(),
                    PropertyKey::StringLiteral(lit) => lit.value.to_string(),
                    _ => continue,
                };
                if key == "theme" {
                    if let Expression::ObjectExpression(theme_obj) = &p.value {
                        Self::parse_theme_object(theme_obj, result);
                    }
                }
            }
        }
    }

    fn parse_theme_object(obj: &ObjectExpression, result: &mut ConfigDetectionResult) {
        for prop in &obj.properties {
            if let ObjectPropertyKind::ObjectProperty(p) = prop {
                let key = match &p.key {
                    PropertyKey::StaticIdentifier(ident) => ident.name.to_string(),
                    PropertyKey::StringLiteral(lit) => lit.value.to_string(),
                    _ => continue,
                };
                match key.as_str() {
                    "extend" => {
                        if let Expression::ObjectExpression(extend_obj) = &p.value {
                            Self::parse_theme_object(extend_obj, result);
                        }
                    }
                    "spacing" => {
                        if let Expression::ObjectExpression(spacing_obj) = &p.value {
                            result.spacing.extend(Self::extract_string_map(spacing_obj));
                        }
                    }
                    "colors" => {
                        if let Expression::ObjectExpression(colors_obj) = &p.value {
                            result.colors.extend(Self::extract_string_map(colors_obj));
                        }
                    }
                    "screens" => {
                        if let Expression::ObjectExpression(screens_obj) = &p.value {
                            result.screens.extend(Self::extract_string_map(screens_obj));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn extract_string_map(obj: &ObjectExpression) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for prop in &obj.properties {
            if let ObjectPropertyKind::ObjectProperty(p) = prop {
                let key = match &p.key {
                    PropertyKey::StaticIdentifier(ident) => ident.name.to_string(),
                    PropertyKey::StringLiteral(lit) => lit.value.to_string(),
                    _ => continue,
                };
                if let Expression::StringLiteral(lit) = &p.value {
                    map.insert(key, lit.value.to_string());
                } else if let Expression::NumericLiteral(lit) = &p.value {
                    map.insert(key, lit.value.to_string());
                }
            }
        }
        map
    }
}

struct ConfigVisitor<'a, 'b> {
    result: &'b mut ConfigDetectionResult,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, 'b> oxc_ast_visit::Visit<'a> for ConfigVisitor<'a, 'b> {
    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        if let ExportDefaultDeclarationKind::ObjectExpression(obj) = &decl.declaration {
            ConfigDetector::parse_config_object(obj, self.result);
        }
    }

    fn visit_assignment_expression(&mut self, assign: &AssignmentExpression<'a>) {
        // Match module.exports = { ... }
        if let AssignmentTarget::StaticMemberExpression(member) = &assign.left {
            if let Expression::Identifier(ident) = &member.object {
                if ident.name == "module" && member.property.name == "exports" {
                    if let Expression::ObjectExpression(obj) = &assign.right {
                        ConfigDetector::parse_config_object(obj, self.result);
                    }
                }
            }
        }
    }
}
