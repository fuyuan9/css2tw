use css2tw_core::report::FailureReason;
use css2tw_core::source::html::HtmlParser;
use css2tw_core::source::SourceFile;

#[test]
fn test_html_vue_binding_detection() {
    let parser = HtmlParser;
    let source = SourceFile {
        path: "test.vue".to_string(),
        content: "<div :class=\"{ 'is-active': active }\"></div>".to_string(),
    };

    let replacements = parser.plan_html(&source, &[], 4.0, true, false).unwrap();
    assert!(replacements
        .iter()
        .any(|r| r.failure_reason == Some(FailureReason::DynamicClass)));
}

#[test]
fn test_html_angular_binding_detection() {
    let parser = HtmlParser;
    let source = SourceFile {
        path: "test.component.html".to_string(),
        content: "<div [class]=\"myClass\"></div>".to_string(),
    };

    let replacements = parser.plan_html(&source, &[], 4.0, true, false).unwrap();
    assert!(replacements
        .iter()
        .any(|r| r.failure_reason == Some(FailureReason::DynamicClass)));
}
