use anyhow::Result;
use css2tw_core::{Config, Converter};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = io::stdin();
    let handle = stdin.lock();

    for line in handle.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                continue;
            }
        };

        let response = handle_request(request).await;
        println!("{}", serde_json::to_string(&response)?);
    }

    Ok(())
}

async fn handle_request(req: JsonRpcRequest) -> JsonRpcResponse {
    let id = req.id.clone();
    let result = match req.method.as_str() {
        "initialize" => Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "css2tw-mcp",
                "version": "0.1.0"
            }
        })),
        "tools/list" => Some(serde_json::json!({
            "tools": [
                {
                    "name": "scan_project",
                    "description": "Analyze a project directory for CSS to Tailwind migration scope",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "Path to the project directory" }
                        },
                        "required": ["path"]
                    }
                },
                {
                    "name": "detect_config",
                    "description": "Detect and extract Tailwind configuration from the project",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "Path to the project root" }
                        },
                        "required": ["path"]
                    }
                }
            ]
        })),
        "tools/call" => {
            let params = req.params.unwrap_or_default();
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::json!({}));

            match tool_name {
                "scan_project" => {
                    let path = arguments
                        .get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or(".");
                    match handle_scan(path) {
                        Ok(res) => Some(serde_json::json!({
                            "content": [{ "type": "text", "text": serde_json::to_string(&res).unwrap() }]
                        })),
                        Err(e) => Some(serde_json::json!({
                            "isError": true,
                            "content": [{ "type": "text", "text": format!("Error: {}", e) }]
                        })),
                    }
                }
                "detect_config" => {
                    let path = arguments
                        .get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or(".");
                    match handle_detect_config(path) {
                        Ok(res) => Some(serde_json::json!({
                            "content": [{ "type": "text", "text": serde_json::to_string(&res).unwrap() }]
                        })),
                        Err(e) => Some(serde_json::json!({
                            "isError": true,
                            "content": [{ "type": "text", "text": format!("Error: {}", e) }]
                        })),
                    }
                }
                _ => Some(serde_json::json!({
                    "isError": true,
                    "content": [{ "type": "text", "text": format!("Unknown tool: {}", tool_name) }]
                })),
            }
        }
        _ => None,
    };

    let is_none = result.is_none();
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result,
        error: if is_none {
            Some(serde_json::json!({"code": -32601, "message": "Method not found"}))
        } else {
            None
        },
        id,
    }
}

fn handle_scan(path: &str) -> Result<serde_json::Value> {
    // Basic scan logic using css2tw-core
    // This is a simplified version of CLI scan
    let files = css2tw_core::source::Scanner::scan_directory(path)?;
    let css_files: Vec<_> = files
        .iter()
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("css"))
        .collect();
    let other_files: Vec<_> = files
        .iter()
        .filter(|p| p.extension().and_then(|s| s.to_str()) != Some("css"))
        .collect();

    let mut css_contents = Vec::new();
    for cp in css_files {
        if let Ok(c) = std::fs::read_to_string(cp) {
            css_contents.push(c);
        }
    }

    let config = Config::default();
    let converter = Converter::new(config);
    let paths: Vec<PathBuf> = other_files.iter().map(|p| (*p).clone()).collect();
    let source_files = css2tw_core::source::Scanner::read_files_parallel(&paths);

    let mut total_classes = 0;
    let mut convertible = 0;

    for sf in source_files {
        if let Ok(reps) = converter.plan_file(&sf, &css_contents) {
            total_classes += reps.len();
            convertible += reps.iter().filter(|r| !r.after.is_empty()).count();
        }
    }

    Ok(serde_json::json!({
        "files_scanned": other_files.len(),
        "total_classes": total_classes,
        "convertible_classes": convertible,
        "conversion_rate": if total_classes > 0 { convertible as f64 / total_classes as f64 } else { 0.0 }
    }))
}

fn handle_detect_config(path: &str) -> Result<serde_json::Value> {
    let config =
        css2tw_core::tailwind::detector::ConfigDetector::detect(std::path::Path::new(path));
    Ok(serde_json::to_value(config)?)
}
