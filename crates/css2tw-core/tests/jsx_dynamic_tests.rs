use css2tw_core::report::FailureReason;
use css2tw_core::source::jsx::JsxParser;
use css2tw_core::source::SourceFile;

#[test]
fn test_jsx_dynamic_clsx_detection() {
    let parser = JsxParser;
    let source = SourceFile {
        path: "test.jsx".to_string(),
        content: "const MyComp = () => <div className={clsx('a', 'b')}></div>;".to_string(),
    };

    let replacements = parser.plan_jsx(&source, &[], 4.0, true, false).unwrap();
    assert_eq!(replacements.len(), 1);
    assert_eq!(
        replacements[0].failure_reason,
        Some(FailureReason::DynamicClass)
    );
    assert!(replacements[0].reasons[0].contains("clsx"));
    assert!(!replacements[0].diagnostics.is_empty());
}

#[test]
fn test_jsx_template_literal_detection() {
    let parser = JsxParser;
    let source = SourceFile {
        path: "test.jsx".to_string(),
        content: "const MyComp = () => <div className={`base ${active ? 'active' : ''}`}></div>;"
            .to_string(),
    };

    let replacements = parser.plan_jsx(&source, &[], 4.0, true, false).unwrap();
    assert_eq!(replacements.len(), 1);
    assert_eq!(
        replacements[0].failure_reason,
        Some(FailureReason::DynamicTemplateLiteral)
    );
    assert!(!replacements[0].diagnostics.is_empty());
}

#[test]
fn test_jsx_conditional_detection() {
    let parser = JsxParser;
    let source = SourceFile {
        path: "test.jsx".to_string(),
        content: "const MyComp = () => <div className={isActive ? 'a' : 'b'}></div>;".to_string(),
    };

    let replacements = parser.plan_jsx(&source, &[], 4.0, true, false).unwrap();
    assert_eq!(replacements.len(), 1);
    assert_eq!(
        replacements[0].failure_reason,
        Some(FailureReason::ConditionalClassExpression)
    );
    assert!(replacements[0].reasons[0].contains("Conditional"));
}

#[test]
fn test_jsx_object_expression_detection() {
    let parser = JsxParser;
    let source = SourceFile {
        path: "test.jsx".to_string(),
        content: "const MyComp = () => <div className={{ 'a-class': true }}></div>;".to_string(),
    };

    let replacements = parser.plan_jsx(&source, &[], 4.0, true, false).unwrap();
    assert_eq!(replacements.len(), 1);
    assert_eq!(
        replacements[0].failure_reason,
        Some(FailureReason::RuntimeClassGeneration)
    );
}

#[test]
fn test_jsx_identifier_detection() {
    let parser = JsxParser;
    let source = SourceFile {
        path: "test.jsx".to_string(),
        content: "const MyComp = () => <div className={myClasses}></div>;".to_string(),
    };

    let replacements = parser.plan_jsx(&source, &[], 4.0, true, false).unwrap();
    assert_eq!(replacements.len(), 1);
    assert_eq!(
        replacements[0].failure_reason,
        Some(FailureReason::DynamicClass)
    );
}
