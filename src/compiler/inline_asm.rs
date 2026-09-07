//! Inline-assembly header resources and ordinary-block body validation.

use super::ast::{AsmClobber, AsmGpr, AsmOutReg};
use super::diagnostic::{DiagCode, Diagnostic};
use super::source::SourceSpan;

pub const RES_A: u16 = 1 << 0;
pub const RES_B: u16 = 1 << 1;
pub const RES_C: u16 = 1 << 2;
pub const RES_D: u16 = 1 << 3;
pub const RES_E: u16 = 1 << 4;
pub const RES_H: u16 = 1 << 5;
pub const RES_L: u16 = 1 << 6;
pub const RES_FLAGS: u16 = 1 << 8;
pub const RES_MEMORY: u16 = 1 << 9;
pub const RES_CARRY_OUT: u16 = 1 << 10;
pub const RES_ZERO_OUT: u16 = 1 << 11;

pub const RES_BC: u16 = RES_B | RES_C;
pub const RES_DE: u16 = RES_D | RES_E;
pub const RES_HL: u16 = RES_H | RES_L;
pub const RES_GP: u16 = RES_A | RES_BC | RES_DE | RES_HL;

pub fn gpr_mask(reg: AsmGpr) -> u16 {
    match reg {
        AsmGpr::A => RES_A,
        AsmGpr::B => RES_B,
        AsmGpr::C => RES_C,
        AsmGpr::D => RES_D,
        AsmGpr::E => RES_E,
        AsmGpr::H => RES_H,
        AsmGpr::L => RES_L,
        AsmGpr::Bc => RES_BC,
        AsmGpr::De => RES_DE,
        AsmGpr::Hl => RES_HL,
    }
}

pub fn out_mask(reg: AsmOutReg) -> u16 {
    match reg {
        AsmOutReg::Gpr(r) => gpr_mask(r),
        AsmOutReg::Carry => RES_CARRY_OUT,
        AsmOutReg::Zero => RES_ZERO_OUT,
    }
}

pub fn clobber_mask(name: AsmClobber) -> u16 {
    match name {
        AsmClobber::Gpr(r) => gpr_mask(r),
        AsmClobber::Flags => RES_FLAGS,
        AsmClobber::Memory => RES_MEMORY,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsmEmitLine {
    Label(String),
    Equ { name: String, expr: String },
    Insn(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedAsm {
    pub lines: Vec<AsmEmitLine>,
    pub labels: Vec<String>,
    pub evident_stack: u16,
    pub has_call: bool,
}

pub fn validate_body(body: &str, span: SourceSpan) -> Result<ValidatedAsm, Vec<Diagnostic>> {
    let mut diags = Vec::new();
    let parsed = match parse_steps(body, span, &mut diags) {
        Some(p) => p,
        None => return Err(diags),
    };
    let evident_stack = match check_control_and_stack_peak(&parsed, span) {
        Ok(peak) => peak,
        Err(d) => {
            diags.push(d);
            return Err(diags);
        }
    };
    if !diags.is_empty() {
        return Err(diags);
    }
    Ok(ValidatedAsm {
        lines: parsed.lines,
        labels: parsed.labels,
        evident_stack,
        has_call: parsed.has_call,
    })
}

pub fn rewrite_lines(lines: &[AsmEmitLine], labels: &[String], prefix: &str) -> Vec<String> {
    let labels_upper: Vec<String> = labels.iter().map(|l| l.to_ascii_uppercase()).collect();
    lines
        .iter()
        .map(|line| match line {
            AsmEmitLine::Label(name) => format!("{prefix}{}:", name.to_ascii_uppercase()),
            AsmEmitLine::Equ { name, expr } => format!(
                "{}{} EQU {}",
                prefix,
                name.to_ascii_uppercase(),
                rewrite_idents(expr, &labels_upper, prefix)
            ),
            AsmEmitLine::Insn(text) => rewrite_idents(text, &labels_upper, prefix),
        })
        .collect()
}

fn rewrite_idents(text: &str, labels: &[String], prefix: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_alphabetic() || c == '_' || c == '.' {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.')
            {
                i += 1;
            }
            let ident: String = chars[start..i].iter().collect();
            let key = ident.to_ascii_uppercase();
            if labels.iter().any(|l| l == &key) {
                out.push_str(prefix);
                out.push_str(&key);
            } else {
                out.push_str(&ident);
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

struct ParsedBody {
    lines: Vec<AsmEmitLine>,
    labels: Vec<String>,
    steps: Vec<Step>,
    label_at: Vec<(String, usize)>,
    has_call: bool,
}

#[derive(Clone)]
enum Step {
    Label,
    Equ,
    Insn {
        height_delta: i16,
        ix_delta: i16,
        iy_delta: i16,
        is_call: bool,
        jump: Option<Jump>,
    },
}

#[derive(Clone)]
struct Jump {
    target: String,
    conditional: bool,
}

fn parse_steps(body: &str, span: SourceSpan, diags: &mut Vec<Diagnostic>) -> Option<ParsedBody> {
    let mut lines: Vec<AsmEmitLine> = Vec::new();
    let mut labels: Vec<String> = Vec::new();
    let mut steps: Vec<Step> = Vec::new();
    let mut label_at: Vec<(String, usize)> = Vec::new();
    let mut has_call = false;
    for raw in body.lines() {
        let stripped = match strip_comment(raw) {
            Ok(s) => s,
            Err(msg) => {
                diags.push(Diagnostic::error(DiagCode::CgUnsupported, span, msg));
                continue;
            }
        };
        let mut rest = stripped.as_str().trim();
        if rest.is_empty() {
            continue;
        }
        loop {
            match take_colon_label(rest) {
                Ok(Some((label, after))) => {
                    if labels.iter().any(|l| l.eq_ignore_ascii_case(&label)) {
                        diags.push(Diagnostic::error(
                            DiagCode::CgUnsupported,
                            span,
                            format!("duplicate asm label '{label}'"),
                        ));
                    } else {
                        labels.push(label.clone());
                    }
                    label_at.push((label.clone(), steps.len()));
                    lines.push(AsmEmitLine::Label(label.clone()));
                    steps.push(Step::Label);
                    rest = after.trim();
                    if rest.is_empty() {
                        break;
                    }
                }
                Ok(None) => break,
                Err(msg) => {
                    diags.push(Diagnostic::error(DiagCode::CgUnsupported, span, msg));
                    rest = "";
                    break;
                }
            }
        }
        if rest.is_empty() {
            continue;
        }
        if let Some((name, expr)) = split_equ(rest) {
            lines.push(AsmEmitLine::Equ {
                name: name.to_string(),
                expr: expr.to_string(),
            });
            steps.push(Step::Equ);
            continue;
        }
        match classify_instruction(rest) {
            Ok(info) => {
                if info.is_call {
                    has_call = true;
                }
                lines.push(AsmEmitLine::Insn(rest.to_string()));
                steps.push(Step::Insn {
                    height_delta: info.height_delta,
                    ix_delta: info.ix_delta,
                    iy_delta: info.iy_delta,
                    is_call: info.is_call,
                    jump: info.jump,
                });
            }
            Err(msg) => diags.push(Diagnostic::error(DiagCode::CgUnsupported, span, msg)),
        }
    }
    if diags.iter().any(Diagnostic::is_error) {
        return None;
    }
    Some(ParsedBody {
        lines,
        labels,
        steps,
        label_at,
        has_call,
    })
}

struct InsnInfo {
    height_delta: i16,
    ix_delta: i16,
    iy_delta: i16,
    is_call: bool,
    jump: Option<Jump>,
}

fn classify_instruction(text: &str) -> Result<InsnInfo, String> {
    let (mnem, operands) = split_mnemonic(text);
    let mnem_u = mnem.to_ascii_uppercase();
    if matches!(
        mnem_u.as_str(),
        "ORG"
            | "BASIC_START"
            | "DB"
            | "DEFB"
            | "DW"
            | "DEFW"
            | "DS"
            | "DEFS"
            | "RET"
            | "RETI"
            | "RETN"
            | "EXX"
    ) {
        return Err(format!(
            "'{mnem_u}' is not allowed in ordinary inline asm blocks"
        ));
    }
    if mnem_u == "EX" {
        let joined = operands.replace(' ', "").to_ascii_uppercase();
        if joined.contains("AF") {
            return Err("EX AF,AF' is not allowed in ordinary inline asm blocks".into());
        }
    }
    if contains_word(&operands, "SP")
        || mnem_u == "INC" && operand_is(&operands, "SP")
        || mnem_u == "DEC" && operand_is(&operands, "SP")
    {
        return Err("explicit SP adjustments are not allowed in ordinary inline asm".into());
    }
    let ops_u = operands.to_ascii_uppercase();
    let ix = contains_word(&operands, "IX");
    let iy = contains_word(&operands, "IY");
    let push_ix = mnem_u == "PUSH" && operand_is(&operands, "IX");
    let pop_ix = mnem_u == "POP" && operand_is(&operands, "IX");
    let push_iy = mnem_u == "PUSH" && operand_is(&operands, "IY");
    let pop_iy = mnem_u == "POP" && operand_is(&operands, "IY");
    if (ix && !(push_ix || pop_ix)) || (iy && !(push_iy || pop_iy)) {
        return Err(
            "IX/IY writes are not allowed in ordinary inline asm except balanced PUSH/POP".into(),
        );
    }
    if (ops_u.contains("(HL)") && mnem_u == "JP")
        || (ops_u.contains("(IX)") && mnem_u == "JP")
        || (ops_u.contains("(IY)") && mnem_u == "JP")
    {
        return Err("indirect jumps are not allowed in ordinary inline asm".into());
    }
    if text.contains("@{") {
        return Ok(InsnInfo {
            height_delta: 0,
            ix_delta: 0,
            iy_delta: 0,
            is_call: true,
            jump: None,
        });
    }
    let mut info = InsnInfo {
        height_delta: 0,
        ix_delta: 0,
        iy_delta: 0,
        is_call: false,
        jump: None,
    };
    match mnem_u.as_str() {
        "PUSH" => {
            info.height_delta = 2;
            if push_ix {
                info.ix_delta = 1;
            }
            if push_iy {
                info.iy_delta = 1;
            }
        }
        "POP" => {
            info.height_delta = -2;
            if pop_ix {
                info.ix_delta = -1;
            }
            if pop_iy {
                info.iy_delta = -1;
            }
        }
        "CALL" | "RST" => info.is_call = true,
        "JP" | "JR" | "DJNZ" => {
            info.jump = Some(parse_jump(&mnem_u, &operands)?);
        }
        _ => {}
    }
    Ok(info)
}

fn parse_jump(mnem: &str, operands: &str) -> Result<Jump, String> {
    let ops: Vec<&str> = split_operands(operands);
    if ops.is_empty() {
        return Err(format!("{mnem} needs a target"));
    }
    let (conditional, target) = if mnem == "DJNZ" {
        (true, ops[0])
    } else if ops.len() == 2 && is_cc(ops[0]) {
        (true, ops[1])
    } else {
        (false, ops[0])
    };
    let target = target.trim();
    if target.eq_ignore_ascii_case("$") {
        return Ok(Jump {
            target: "$".into(),
            conditional,
        });
    }
    if !is_ident(target) {
        return Err(format!(
            "{mnem} target '{target}' is not a local label in this asm block"
        ));
    }
    Ok(Jump {
        target: target.to_ascii_uppercase(),
        conditional,
    })
}

fn check_control_and_stack_peak(parsed: &ParsedBody, span: SourceSpan) -> Result<u16, Diagnostic> {
    use std::collections::HashMap;
    let mut label_pc: HashMap<String, usize> = HashMap::new();
    for (name, idx) in &parsed.label_at {
        label_pc.insert(name.to_ascii_uppercase(), *idx);
    }
    let mut seen: HashMap<usize, (i16, i16, i16)> = HashMap::new();
    let mut peak: u16 = 0;
    walk_stack(parsed, &label_pc, &mut seen, &mut peak, span, 0, 0, 0, 0)?;
    Ok(peak)
}

fn walk_stack(
    parsed: &ParsedBody,
    label_pc: &std::collections::HashMap<String, usize>,
    seen: &mut std::collections::HashMap<usize, (i16, i16, i16)>,
    peak: &mut u16,
    span: SourceSpan,
    pc: usize,
    height: i16,
    ix: i16,
    iy: i16,
) -> Result<(), Diagnostic> {
    if pc == parsed.steps.len() {
        if height != 0 || ix != 0 || iy != 0 {
            return Err(Diagnostic::error(
                DiagCode::CgUnsupported,
                span,
                "inline asm stack is unbalanced at the block exit",
            ));
        }
        return Ok(());
    }
    if let Some(&prev) = seen.get(&pc) {
        if prev != (height, ix, iy) {
            return Err(Diagnostic::error(
                DiagCode::CgUnsupported,
                span,
                "inline asm stack height differs at a merged local branch",
            ));
        }
        return Ok(());
    }
    seen.insert(pc, (height, ix, iy));
    match &parsed.steps[pc] {
        Step::Label | Step::Equ => {
            walk_stack(parsed, label_pc, seen, peak, span, pc + 1, height, ix, iy)
        }
        Step::Insn {
            height_delta,
            ix_delta,
            iy_delta,
            is_call,
            jump,
        } => {
            if height < 0 {
                return Err(Diagnostic::error(
                    DiagCode::CgUnsupported,
                    span,
                    "inline asm POP below the entry stack",
                ));
            }
            let extra = if *is_call { 2 } else { 0 };
            let used = height.saturating_add(extra);
            if used > 0 {
                *peak = (*peak).max(used as u16);
            }
            let next_h = height + *height_delta;
            let next_ix = ix + *ix_delta;
            let next_iy = iy + *iy_delta;
            if next_h < 0 {
                return Err(Diagnostic::error(
                    DiagCode::CgUnsupported,
                    span,
                    "inline asm POP below the entry stack",
                ));
            }
            if let Some(j) = jump {
                if j.target == "$" {
                    if j.conditional {
                        walk_stack(
                            parsed,
                            label_pc,
                            seen,
                            peak,
                            span,
                            pc + 1,
                            next_h,
                            next_ix,
                            next_iy,
                        )?;
                    }
                    return Ok(());
                }
                let Some(&target) = label_pc.get(&j.target) else {
                    return Err(Diagnostic::error(
                        DiagCode::CgUnsupported,
                        span,
                        format!(
                            "asm jump to '{}' is not a local label in this block",
                            j.target
                        ),
                    ));
                };
                walk_stack(
                    parsed, label_pc, seen, peak, span, target, next_h, next_ix, next_iy,
                )?;
                if j.conditional {
                    walk_stack(
                        parsed,
                        label_pc,
                        seen,
                        peak,
                        span,
                        pc + 1,
                        next_h,
                        next_ix,
                        next_iy,
                    )?;
                }
                Ok(())
            } else {
                walk_stack(
                    parsed,
                    label_pc,
                    seen,
                    peak,
                    span,
                    pc + 1,
                    next_h,
                    next_ix,
                    next_iy,
                )
            }
        }
    }
}

fn strip_comment(source: &str) -> Result<String, String> {
    let mut quote = None;
    let mut escape = false;
    let mut out = String::new();
    for ch in source.chars() {
        if escape {
            out.push(ch);
            escape = false;
            continue;
        }
        if quote.is_some() && ch == '\\' {
            out.push(ch);
            escape = true;
            continue;
        }
        if let Some(q) = quote {
            out.push(ch);
            if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            ';' => break,
            '"' | '\'' => {
                quote = Some(ch);
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    if quote.is_some() {
        return Err("unterminated string in asm body".into());
    }
    Ok(out)
}

fn take_colon_label(source: &str) -> Result<Option<(String, &str)>, String> {
    let Some(colon) = source.find(':') else {
        return Ok(None);
    };
    let before = source[..colon].trim();
    if before.is_empty() || before.contains(char::is_whitespace) {
        return Ok(None);
    }
    if !is_ident(before) {
        return Err(format!("invalid asm label '{before}'"));
    }
    Ok(Some((before.to_ascii_uppercase(), &source[colon + 1..])))
}

fn split_equ(source: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = source.split_whitespace().collect();
    if parts.len() >= 3 && parts[1].eq_ignore_ascii_case("EQU") {
        let name = parts[0];
        let expr = source[source.find(parts[2])?..].trim();
        if is_ident(name) {
            return Some((name, expr));
        }
    }
    None
}

fn split_mnemonic(text: &str) -> (&str, String) {
    let split = text.find(char::is_whitespace).unwrap_or(text.len());
    (text[..split].trim(), text[split..].trim().to_string())
}

fn split_operands(operands: &str) -> Vec<&str> {
    if operands.is_empty() {
        return Vec::new();
    }
    operands
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

fn operand_is(operands: &str, name: &str) -> bool {
    split_operands(operands)
        .iter()
        .any(|op| op.eq_ignore_ascii_case(name))
}

fn contains_word(hay: &str, word: &str) -> bool {
    let h = hay.to_ascii_uppercase();
    let w = word.to_ascii_uppercase();
    h.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .any(|tok| tok == w)
}

fn is_cc(op: &str) -> bool {
    matches!(
        op.to_ascii_uppercase().as_str(),
        "Z" | "NZ" | "C" | "NC" | "PO" | "PE" | "P" | "M"
    )
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_' || first == '.')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
}
