//! The C80 documentation as tests (plan H6).
//!
//! - Every fenced code block in `info/c80.md` and `info/c80/tutorial.md` has
//!   an info string. A `c80` block is a complete unit that must compile in
//!   both optimize modes; `c80 error=code,code` must instead report exactly
//!   those error codes (as a multiset) in both modes. `target=tvc` selects
//!   the TVC target. A `c`/`cpp`/`C` block or a block without an info string
//!   fails: C80 in the docs is always marked and always checked.
//! - A `text output` block right after a `c80` block lists diagnostics as
//!   `error code: message`; each line must be a diagnostic of that block.
//! - An `asm golden=NAME.asm` block must appear, line for line, in that
//!   golden listing under `info/c80/expected/`. Other `asm`/`z80` blocks
//!   fail.
//! - Relative Markdown links in the C80 docs and the repository documents
//!   that describe C80 must name an existing file, and a `#anchor` must name
//!   a heading of the target Markdown file.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use rtvc_c80::{C80Target, CompileInput, SourceInput, compile};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Documents whose code blocks are checked.
const CODE_DOCS: &[&str] = &["info/c80.md", "info/c80/tutorial.md"];

/// Documents whose relative links are checked.
const LINK_DOCS: &[&str] = &[
    "info/c80.md",
    "info/c80/tutorial.md",
    "README.md",
    "TODO.md",
    "CHANGES.md",
    ".agents/plans/README.md",
    ".agents/plans/c80-compiler.md",
    ".agents/plans/c80-compiler-findings.md",
    ".agents/plans/c80-hardening.md",
    ".agents/plans/c80-hardening-progress.md",
    ".agents/skills/development/SKILL.md",
];

struct Block {
    line: usize,
    info: String,
    body: String,
}

fn fences(text: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let mut open: Option<(usize, String, String, String)> = None;
    for (i, raw) in text.lines().enumerate() {
        let trimmed = raw.trim_start();
        let indent = &raw[..raw.len() - trimmed.len()];
        match &mut open {
            None => {
                if let Some(info) = trimmed.strip_prefix("```") {
                    open = Some((
                        i + 1,
                        info.trim().to_string(),
                        indent.to_string(),
                        String::new(),
                    ));
                }
            }
            Some((line, info, ind, body)) => {
                if trimmed.starts_with("```") {
                    out.push(Block {
                        line: *line,
                        info: info.clone(),
                        body: std::mem::take(body),
                    });
                    open = None;
                } else {
                    body.push_str(raw.strip_prefix(ind.as_str()).unwrap_or(raw));
                    body.push('\n');
                }
            }
        }
    }
    assert!(open.is_none(), "unclosed code fence");
    out
}

fn rendered(src: &str) -> Vec<String> {
    let result = compile(CompileInput {
        files: vec![SourceInput {
            name: "doc.c80",
            text: src,
        }],
        origin: 0x8000,
        optimize: true,
        target: C80Target::GenericZ80,
    });
    result
        .diagnostics
        .iter()
        .map(|d| {
            let severity = if d.is_error() { "error" } else { "warning" };
            format!("{severity} {}: {}", d.code.as_str(), d.message)
        })
        .collect()
}

fn check_output(doc: &str, block: &Block, previous: Option<&Block>) -> Result<(), String> {
    let Some(previous) = previous else {
        return Err(format!(
            "{doc}:{}: a `text output` block must follow a c80 block",
            block.line
        ));
    };
    let got = rendered(&previous.body);
    for line in block.body.lines().filter(|l| !l.trim().is_empty()) {
        if !got.iter().any(|g| g == line.trim()) {
            return Err(format!(
                "{doc}:{}: '{line}' is not a diagnostic of the block at line {}; got {got:?}",
                block.line, previous.line
            ));
        }
    }
    Ok(())
}

fn error_codes(src: &str, optimize: bool, target: C80Target) -> Vec<String> {
    let result = compile(CompileInput {
        files: vec![SourceInput {
            name: "doc.c80",
            text: src,
        }],
        origin: 0x8000,
        optimize,
        target,
    });
    let mut codes: Vec<String> = result
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .map(|d| format!("{} ({})", d.code.as_str(), d.message))
        .collect();
    if codes.is_empty() {
        assert!(result.code.is_some(), "no code and no error:\n{src}");
    }
    codes.sort();
    codes
}

fn check_c80(doc: &str, block: &Block, attrs: &[&str]) -> Result<(), String> {
    let mut expected: Vec<String> = Vec::new();
    let mut target = C80Target::GenericZ80;
    for attr in attrs {
        if let Some(list) = attr.strip_prefix("error=") {
            expected.extend(list.split(',').map(str::to_string));
        } else if *attr == "target=tvc" {
            target = C80Target::Tvc;
        } else {
            return Err(format!(
                "{doc}:{}: unknown c80 attribute '{attr}'",
                block.line
            ));
        }
    }
    expected.sort();
    for optimize in [true, false] {
        let got = error_codes(&block.body, optimize, target);
        let got_codes: Vec<String> = got
            .iter()
            .map(|c| c.split(' ').next().unwrap().to_string())
            .collect();
        if got_codes != expected {
            return Err(format!(
                "{doc}:{} (optimize {optimize}): expected errors {expected:?}, got {got:?}\n{}",
                block.line, block.body
            ));
        }
    }
    Ok(())
}

fn check_golden(doc: &str, block: &Block, attrs: &[&str]) -> Result<(), String> {
    let Some(name) = attrs.iter().find_map(|a| a.strip_prefix("golden=")) else {
        return Err(format!(
            "{doc}:{}: an asm block needs golden=FILE (or link to the golden listing)",
            block.line
        ));
    };
    let path = repo().join("info/c80/expected").join(name);
    let golden = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let golden: Vec<&str> = golden.lines().map(str::trim).collect();
    let want: Vec<&str> = block
        .body
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let found = golden
        .windows(want.len().max(1))
        .any(|w| w == want.as_slice());
    if !found {
        return Err(format!(
            "{doc}:{}: block is not in {name}:\n{}",
            block.line, block.body
        ));
    }
    Ok(())
}

#[test]
fn doc_code_blocks_compile_or_fail_as_stated() {
    let mut errors = Vec::new();
    let mut counts: HashMap<&str, (usize, usize)> = HashMap::new();
    for doc in CODE_DOCS {
        let text = fs::read_to_string(repo().join(doc)).unwrap();
        let mut previous: Option<Block> = None;
        for block in fences(&text) {
            let mut words = block.info.split_whitespace();
            let lang = words.next().unwrap_or("");
            let attrs: Vec<&str> = words.collect();
            let result = match lang {
                "c80" => {
                    let entry = counts.entry(doc).or_default();
                    if attrs.iter().any(|a| a.starts_with("error=")) {
                        entry.1 += 1;
                    } else {
                        entry.0 += 1;
                    }
                    check_c80(doc, &block, &attrs)
                }
                "asm" | "z80" => check_golden(doc, &block, &attrs),
                "text" if attrs == ["output"] => check_output(doc, &block, previous.as_ref()),
                "" => Err(format!(
                    "{doc}:{}: code block without an info string",
                    block.line
                )),
                "c" | "C" | "cpp" | "c++" | "h" => Err(format!(
                    "{doc}:{}: C80 code must be marked `c80` so it is compiled",
                    block.line
                )),
                _ => Ok(()),
            };
            if let Err(e) = result {
                errors.push(e);
            }
            if lang == "c80" {
                previous = Some(block);
            }
        }
    }
    for (doc, (ok, fail)) in &counts {
        println!("{doc}: {ok} compiling c80 blocks, {fail} rejected c80 blocks");
    }
    assert!(errors.is_empty(), "{}", errors.join("\n\n"));
}

/// GitHub-style heading anchors of a Markdown file.
fn anchors(text: &str) -> Vec<String> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let Some(rest) = line.strip_prefix('#') else {
            continue;
        };
        let heading = rest.trim_start_matches('#');
        if !heading.starts_with(' ') {
            continue;
        }
        let plain = strip_links(heading.trim());
        let mut slug = String::new();
        for ch in plain.chars() {
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                slug.extend(ch.to_lowercase());
            } else if ch == ' ' {
                slug.push('-');
            }
        }
        let n = seen.entry(slug.clone()).or_insert(0);
        if *n > 0 {
            out.push(format!("{slug}-{n}"));
        } else {
            out.push(slug.clone());
        }
        *n += 1;
    }
    out
}

/// `[text](url)` to `text`.
fn strip_links(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find("](").and_then(|close| {
            after[close + 2..]
                .find(')')
                .map(|end| (close, close + 2 + end))
        }) {
            Some((close, end)) => {
                out.push_str(&after[..close]);
                rest = &after[end + 1..];
            }
            None => {
                out.push('[');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Relative link targets outside code fences and inline code.
fn links(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for (i, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        // Drop inline code spans.
        let mut plain = String::new();
        let mut in_code = false;
        for ch in line.chars() {
            if ch == '`' {
                in_code = !in_code;
            } else if !in_code {
                plain.push(ch);
            }
        }
        let mut rest = plain.as_str();
        while let Some(at) = rest.find("](") {
            let tail = &rest[at + 2..];
            let Some(end) = tail.find(')') else {
                break;
            };
            let target = tail[..end].trim();
            let target = target.split_whitespace().next().unwrap_or("");
            if !target.is_empty() && !target.contains("://") && !target.starts_with("mailto:") {
                out.push((i + 1, target.to_string()));
            }
            rest = &tail[end..];
        }
    }
    out
}

#[test]
fn doc_relative_links_resolve() {
    let mut errors = Vec::new();
    let mut checked = 0usize;
    let mut cache: HashMap<PathBuf, Vec<String>> = HashMap::new();
    for doc in LINK_DOCS {
        let path = repo().join(doc);
        let text = fs::read_to_string(&path).unwrap();
        let dir = path.parent().unwrap().to_path_buf();
        for (line, target) in links(&text) {
            checked += 1;
            let (file, anchor) = match target.split_once('#') {
                Some((f, a)) => (f, Some(a)),
                None => (target.as_str(), None),
            };
            let resolved = if file.is_empty() {
                path.clone()
            } else {
                dir.join(file)
            };
            if !resolved.exists() {
                errors.push(format!("{doc}:{line}: missing file {target}"));
                continue;
            }
            let Some(anchor) = anchor else {
                continue;
            };
            if resolved.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let list = cache
                .entry(resolved.clone())
                .or_insert_with(|| anchors(&fs::read_to_string(&resolved).unwrap_or_default()));
            if !list.iter().any(|a| a == anchor) {
                errors.push(format!(
                    "{doc}:{line}: no heading for #{anchor} in {}",
                    rel(&resolved)
                ));
            }
        }
    }
    println!("{checked} relative links checked");
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

fn rel(path: &Path) -> String {
    path.strip_prefix(repo())
        .unwrap_or(path)
        .display()
        .to_string()
}
