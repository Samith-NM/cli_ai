use anyhow::Result;
use regex::Regex;
use serde::Serialize;

#[derive(Debug,Serialize,Clone)]
pub struct Reference{
    pub file: String,
    pub line:usize,
    pub snippet:Option<String>,
}

#[derive(Debug,Serialize)]

pub struct Contextpayload{
    pub file:Option<String>,
    pub line:Option<usize>,
    pub extracctedsnippet:Option<String>,
    pub references:Vec<Reference>,
    pub rawstacktrace: String,
    pub signalname:Option<String>,


}

fn extractall_location(stderr:&str) -> Vec<(String,usize)> {
    let re = Regex::new(r"([\w./\-]+\.(?:c|cpp|cc|h|hpp|rs)):(\d+)").unwrap();
     let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for caps in re.captures_iter(stderr) {
        let file = caps[1].to_string();
        let line: usize = match caps[2].parse() {
            Ok(l) => l,
            Err(_) => continue,
        };
        if seen.insert((file.clone(), line)) {
            out.push((file, line));
        }
    }
    out

}
fn extract_snippet(path: &str, line: usize, context: usize) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    if line == 0 || line > lines.len() {
        return None;
    }
    let idx = line - 1; // to 0-based
    let start = idx.saturating_sub(context);
    let end = (idx + context + 1).min(lines.len());

    let mut out = String::new();
    for (i, l) in lines[start..end].iter().enumerate() {
        let lineno = start + i + 1;
        let marker = if lineno == line { ">>" } else { "  " };
        out.push_str(&format!("{marker} {lineno:>4} | {l}\n"));
    }
    Some(out)
}
pub fn build_payload(
    stderr: &str,
    signal_name: Option<&str>,
    context_lines: usize,
) -> Result<Contextpayload> {
    let locations = extractall_locations(stderr);

    let references: Vec<Reference> = locations
        .iter()
        .map(|(file, line)| Reference {
            file: file.clone(),
            line: *line,
            snippet: extract_snippet(file, *line, context_lines),
        })
        .collect();

    let primary = references.first();

    Ok(ContextPayload {
        file: primary.map(|r| r.file.clone()),
        line: primary.map(|r| r.line),
        extracted_snippet: primary.and_then(|r| r.snippet.clone()),
        references,
        raw_stack_trace: stderr.to_string(),
        signal_name: signal_name.map(|s| s.to_string()),
    })
}


