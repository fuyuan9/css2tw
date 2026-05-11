use crate::error::Css2TwError;
use crate::source::{
    class_usage::{ClassUsage, Span},
    ClassUsageParser, SourceFile,
};
use regex::Regex;

/// A generic parser that uses regular expressions to find class names in any text file.
/// This is a fallback parser for file types that don't have a specialized parser.
pub struct GenericRegexParser;

impl ClassUsageParser for GenericRegexParser {
    fn extract_classes(&self, source: &SourceFile) -> Result<Vec<ClassUsage>, Css2TwError> {
        let mut classes = Vec::new();
        // Robust regex for class/className attributes, handling multiline and varied whitespace
        let re = Regex::new(r#"(?i)\b(?:class|className)\s*=\s*(?:"([^"]*)"|'([^']*)')"#).unwrap();

        for cap in re.captures_iter(&source.content) {
            let match_val = cap.get(1).or_else(|| cap.get(2));

            if let Some(match_val) = match_val {
                let attr_content_start = match_val.start();
                let class_string = match_val.as_str();

                let mut current_offset = 0;

                for part in class_string.split_whitespace() {
                    // Skip parts that look like template tags
                    if part.contains("{{")
                        || part.contains("{%")
                        || part.contains("<%")
                        || part.contains("@")
                    {
                        continue;
                    }

                    let part_len = part.len();
                    if let Some(relative_start) = class_string[current_offset..].find(part) {
                        let absolute_start = attr_content_start + current_offset + relative_start;
                        let absolute_end = absolute_start + part_len;

                        classes.push(ClassUsage {
                            class_name: part.to_string(),
                            span: Span {
                                start: absolute_start,
                                end: absolute_end,
                            },
                        });

                        current_offset += relative_start + part_len;
                    }
                }
            }
        }

        Ok(classes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_classes_php() {
        let parser = GenericRegexParser;
        let source = SourceFile {
            path: "test.php".to_string(),
            content: r#"<div class="btn primary"> <?php echo "hello"; ?> <span className='text-red'></span> </div>"#.to_string(),
        };

        let classes = parser.extract_classes(&source).unwrap();

        assert_eq!(classes.len(), 3);
        assert_eq!(classes[0].class_name, "btn");
        assert_eq!(classes[1].class_name, "primary");
        assert_eq!(classes[2].class_name, "text-red");

        // Verify spans
        assert_eq!(
            &source.content[classes[0].span.start..classes[0].span.end],
            "btn"
        );
        assert_eq!(
            &source.content[classes[1].span.start..classes[1].span.end],
            "primary"
        );
        assert_eq!(
            &source.content[classes[2].span.start..classes[2].span.end],
            "text-red"
        );
    }

    #[test]
    fn test_duplicate_classes() {
        let parser = GenericRegexParser;
        let source = SourceFile {
            path: "test.html".to_string(),
            content: r#"<div class="btn btn"></div>"#.to_string(),
        };

        let classes = parser.extract_classes(&source).unwrap();
        assert_eq!(classes.len(), 2);
        assert_eq!(classes[0].class_name, "btn");
        assert_eq!(classes[1].class_name, "btn");
        assert_ne!(classes[0].span.start, classes[1].span.start);
    }
}
