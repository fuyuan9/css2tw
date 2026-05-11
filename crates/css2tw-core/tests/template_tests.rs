use css2tw_core::config::Config;
use css2tw_core::source::SourceFile;
use css2tw_core::Converter;

#[test]
fn test_blade_fragment_conversion() {
    let mut config = Config::default();
    config.tailwind.rem_scale = 4.0;

    let converter = Converter::new(config);

    let source = SourceFile {
        path: "resources/views/welcome.blade.php".to_string(),
        content: r#"
<div class="pt-16">
    @if($show)
        <p class="mb-4 text-blue-500">Hello {{ $name }}</p>
    @endif
</div>
"#
        .to_string(),
    };

    let css = r#"
        .pt-16 { padding-top: 16px; }
        .mb-4 { margin-bottom: 4px; }
        .text-blue-500 { color: blue; }
    "#
    .to_string();

    let result = converter.convert_file(&source, &[css]).unwrap();
    println!("Blade Result: {}", result);

    assert!(result.contains("pt-4"));
    assert!(result.contains("mb-1"));
    // .text-blue-500 { color: blue } matches, but might be converted to hex text-[#00f]
    assert!(result.contains("text-[#00f]") || result.contains("text-blue-500"));
    assert!(result.contains("@if($show)"));
    assert!(result.contains("{{ $name }}"));
}

#[test]
fn test_php_fragment_with_dynamic_classes() {
    let config = Config::default();
    let converter = Converter::new(config);

    let source = SourceFile {
        path: "index.php".to_string(),
        content: r#"<div class="btn-primary <?= $active ? 'active' : '' ?> pt-16"></div>"#
            .to_string(),
    };

    let css = r#".pt-16 { padding-top: 16px; }"#.to_string();
    let result = converter.convert_file(&source, &[css]).unwrap();
    println!("PHP Result: {}", result);

    assert!(result.contains("pt-4"));
    assert!(result.contains("btn-primary")); // Should be preserved as it's not in CSS
    assert!(result.contains("<?= $active ? 'active' : '' ?>")); // Protected
}

#[test]
fn test_jinja2_fragment() {
    let config = Config::default();
    let converter = Converter::new(config);

    let source = SourceFile {
        path: "template.j2".to_string(),
        content: r#"
<div class="pt-16">
    {% for item in items %}
        <span class="mr-8">{{ item }}</span>
    {% endfor %}
</div>
"#
        .to_string(),
    };

    let css = r#"
        .pt-16 { padding-top: 16px; }
        .mr-8 { margin-right: 8px; }
    "#
    .to_string();

    let result = converter.convert_file(&source, &[css]).unwrap();

    assert!(result.contains("pt-4"));
    assert!(result.contains("mr-2"));
    assert!(result.contains("{% for item in items %}"));
}
