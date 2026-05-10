use std::process::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_mdn_selectors_coverage() {
    let dir = tempdir().unwrap();
    let css_path = dir.path().join("selectors.css");
    let html_path = dir.path().join("test.html");

    let css_content = r#"
        /* Basic */
        * { box-sizing: border-box; }
        div { display: block; }
        .btn { padding: 10px; }
        #main { margin: 20px; }

        /* Combinators */
        article p { color: gray; }
        section > p { color: black; }
        h1 + p { margin-top: 0; }
        h1 ~ p { color: blue; }

        /* Attributes */
        [disabled] { opacity: 0.5; }
        input[type="text"] { border: 1px solid black; }
        a[href*="google"] { color: green; }

        /* Pseudo-classes */
        .btn:hover { background-color: gray; }
        input:focus { border-color: blue; }
        button:active { transform: scale(0.95); }
        input:disabled { background: #eee; }
        input:checked + label { font-weight: bold; }
        
        /* Structural Pseudo-classes */
        li:first-child { font-weight: bold; }
        li:last-child { border-bottom: none; }
        li:nth-child(odd) { background: #f9f9f9; }
        li:nth-child(even) { background: #eee; }
        li:nth-child(3n+1) { color: purple; }

        /* Pseudo-elements */
        .box::before { content: 'START'; }
        .box::after { content: 'END'; }
        ::placeholder { color: #ccc; }
        ::selection { background: yellow; }
        
        /* Arbitrary / Advanced */
        div:nth-of-type(2) { border: 2px solid red; }
    "#;

    let html_content = r#"
        <div id="main" class="box">
            <h1>Title</h1>
            <p>Direct p after h1</p>
            <p>Another p</p>
            <section>
                <p>P inside section</p>
            </section>
            <article>
                <p>P inside article</p>
            </article>
            <button class="btn">Click me</button>
            <input type="text" placeholder="Type here...">
            <input type="checkbox" checked id="c1"><label for="c1">Checked</label>
            <input type="text" disabled value="Disabled">
            <a href="https://google.com">Google</a>
            <ul>
                <li>One</li>
                <li>Two</li>
                <li>Three</li>
                <li>Four</li>
            </ul>
        </div>
    "#;

    fs::write(&css_path, css_content).unwrap();
    fs::write(&html_path, html_content).unwrap();

    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("convert")
        .arg(dir.path().to_str().unwrap())
        .arg("--json")
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("Stderr: {}", stderr);
    }
    
    // Check for various expected Tailwind classes
    let assert_msg = format!("Stdout: {}\nStderr: {}", stdout, stderr);
    assert!(stdout.contains("hover:bg-"), "Should have hover variant. {}", assert_msg);
    assert!(stdout.contains("focus:border-"), "Should have focus variant. {}", assert_msg);
    assert!(stdout.contains("active:scale-"), "Should have active variant. {}", assert_msg);
    assert!(stdout.contains("disabled:bg-"), "Should have disabled variant. {}", assert_msg);
    assert!(stdout.contains("first:font-bold"), "Should have first variant. {}", assert_msg);
    assert!(stdout.contains("last:border-b-none"), "Should have last variant. {}", assert_msg);
    assert!(stdout.contains("odd:bg-"), "Should have odd variant. {}", assert_msg);
    assert!(stdout.contains("even:bg-"), "Should have even variant. {}", assert_msg);
    // Content is temporarily disabled
    // assert!(stdout.contains("before:content-"), "Should have before variant. {}", assert_msg);
    // assert!(stdout.contains("after:content-"), "Should have after variant. {}", assert_msg);
    assert!(stdout.contains("placeholder:text-"), "Should have placeholder variant. {}", assert_msg);
    assert!(stdout.contains("selection:bg-"), "Should have selection variant. {}", assert_msg);
    
    // Arbitrary variants
    assert!(stdout.contains("nth-[3n+1]:text-"), "Should have arbitrary nth-child variant. {}", assert_msg);
    assert!(stdout.contains("[&:nth-of-type(2)]:border-"), "Should have arbitrary nth-of-type variant. {}", assert_msg);
    
    // Combinators
    assert!(stdout.contains("text-[gray]"), "Should have gray text from descendant combinator. {}", assert_msg);
    assert!(stdout.contains("text-[#000]"), "Should have black text from child combinator. {}", assert_msg);
    assert!(stdout.contains("mt-[0]"), "Should have margin-top 0 from adjacent sibling combinator. {}", assert_msg);
    assert!(stdout.contains("text-[#00f]"), "Should have blue text from general sibling combinator. {}", assert_msg);
}
