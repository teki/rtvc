#![allow(dead_code)]

use crate::bus::CpuBus;
use crate::z80_tables::{mnemonic_to_opcode, opcode_to_mnemonic};

#[derive(Debug, Clone)]
pub struct AsmResult {
    pub addr: u16,
    pub bytes: Vec<u8>,
}

// ── normalization ─────────────────────────────────────────────────

fn normalize_input(line: &str) -> (String, String) {
    let line = line.trim();
    if line.is_empty() {
        return (String::new(), String::new());
    }

    // Collapse whitespace to single spaces, uppercase everything
    let mut normalized = String::new();
    let mut prev_was_space = true;
    for c in line.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                normalized.push(' ');
                prev_was_space = true;
            }
        } else {
            normalized.push(c.to_ascii_uppercase());
            prev_was_space = false;
        }
    }
    if normalized.ends_with(' ') {
        normalized.pop();
    }

    if let Some(pos) = normalized.find(' ') {
        let mnemonic = normalized[..pos].to_string();
        let params = normalized[pos + 1..].replace(' ', "");
        (mnemonic, params)
    } else {
        (normalized, String::new())
    }
}

// ── number parsing ─────────────────────────────────────────────────

fn parse_number(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(hex) = s.strip_prefix("0X") {
        i64::from_str_radix(hex, 16).ok()
    } else if let Some(hex) = s.strip_suffix('H') {
        i64::from_str_radix(hex, 16).ok()
    } else if let Some(hex) = s.strip_prefix('$') {
        i64::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<i64>().ok()
    }
}

fn parse_signed(s: &str) -> Option<i8> {
    parse_number(s).map(|v| v as i8)
}

// ── operand helpers ────────────────────────────────────────────────

fn split_operands(params: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, c) in params.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                result.push(&params[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    result.push(&params[start..]);
    result
}

fn user_operand_count(params: &str) -> usize {
    let comma_count = params.chars().filter(|&c| c == ',').count();
    if comma_count > 0 {
        comma_count + 1
    } else if params.is_empty() {
        0
    } else {
        1
    }
}

fn is_register(s: &str) -> bool {
    matches!(
        s,
        "A" | "B"
            | "C"
            | "D"
            | "E"
            | "H"
            | "L"
            | "I"
            | "R"
            | "F"
            | "IX"
            | "IY"
            | "SP"
            | "BC"
            | "DE"
            | "HL"
            | "AF"
            | "IXH"
            | "IXL"
            | "IYH"
            | "IYL"
            | "AF'"
    )
}

fn is_16bit_reg(s: &str) -> bool {
    matches!(s, "BC" | "DE" | "HL" | "SP" | "IX" | "IY" | "AF")
}

fn is_condition(s: &str) -> bool {
    matches!(s, "NZ" | "Z" | "NC" | "C" | "PO" | "PE" | "P" | "M")
}

fn extract_indexed(s: &str) -> Option<(String, i8)> {
    let inner = s.strip_prefix('(')?.strip_suffix(')')?;
    for reg in ["IX", "IY"] {
        let Some(rest) = inner.strip_prefix(reg) else {
            continue;
        };
        if let Some(num) = rest.strip_prefix('+') {
            if let Some(v) = parse_number(num) {
                return Some((format!("({}+DD)", reg), v as i8));
            }
        }
        if let Some(num) = rest.strip_prefix('-') {
            if let Some(v) = parse_number(num) {
                return Some((format!("({}+DD)", reg), -(v as i8)));
            }
        }
    }
    None
}

fn extract_op_indexed(s: &str) -> Option<(String, i8)> {
    for op in ["RLC", "RRC", "RL", "RR", "SLA", "SRA", "SLL", "SRL"] {
        if let Some((idx, disp)) = s.strip_prefix(op).and_then(extract_indexed) {
            return Some((format!("{}{}", op, idx), disp));
        }
    }
    None
}

// ── canonical key builder ────────────────────────────────────────

fn build_canonical(mnemonic: &str, params: &str) -> Option<(String, Vec<i8>)> {
    let mut key = String::from(mnemonic);
    let mut values: Vec<i8> = Vec::new();

    // Instructions with no params
    if params.is_empty() {
        return Some((key, values));
    }

    let operands = split_operands(params);
    let num_ops = operands.len();

    // Special cases
    match mnemonic {
        "DJNZ" => {
            let v = parse_signed(params)?;
            key.push_str("OFFSET");
            values.push(v);
            return Some((key, values));
        }
        "JR" => {
            if let Some(comma) = params.find(',') {
                let cond = &params[..comma];
                let v = parse_signed(&params[comma + 1..])?;
                key.push_str(cond);
                key.push_str(",OFFSET");
                values.push(v);
            } else {
                let v = parse_signed(params)?;
                key.push_str("OFFSET");
                values.push(v);
            }
            return Some((key, values));
        }
        "RST" => {
            let v = parse_number(params)?;
            key.push_str(&format!("{}", v));
            return Some((key, values));
        }
        "IM" => {
            let v = parse_number(params)?;
            key.push_str(&format!("{}", v));
            return Some((key, values));
        }
        "JP" => {
            if num_ops == 1 {
                let op = operands[0];
                // JP HL, JP IX, JP IY do not use parentheses in .dat file
                if op.starts_with('(') && op.ends_with(')') {
                    let inner = &op[1..op.len() - 1];
                    if is_register(inner) && !inner.contains('+') && !inner.contains('-') {
                        key.push_str(inner);
                        return Some((key, values));
                    }
                }
            }
        }
        "OUT" => {
            if num_ops == 2 && operands[0] == "(C)" && operands[1] == "0" {
                key.push_str("(C),0");
                return Some((key, values));
            }
        }
        _ => {}
    }

    for (i, op) in operands.iter().enumerate() {
        if i > 0 {
            key.push(',');
        }

        // Check for operation+indexed: RLC(IX+dd), etc.
        if let Some((canon, disp)) = extract_op_indexed(op) {
            key.push_str(&canon);
            values.push(disp);
            continue;
        }

        // Check for indexed addressing: (IX+dd), (IY+dd)
        if let Some((canon, disp)) = extract_indexed(op) {
            key.push_str(&canon);
            values.push(disp);
            continue;
        }

        // Check for parenthesized register or memory reference
        if op.starts_with('(') && op.ends_with(')') {
            let inner = &op[1..op.len() - 1];
            if matches!(inner, "BC" | "DE" | "HL" | "SP" | "IX" | "IY" | "C") {
                key.push_str(op);
                continue;
            }
            if let Some(v) = parse_number(inner) {
                match mnemonic {
                    "IN" | "OUT" => {
                        key.push_str("(NN)");
                        values.push(v as i8);
                    }
                    _ => {
                        key.push_str("(NNNN)");
                        values.push((v & 0xFF) as i8);
                        values.push((v >> 8) as i8);
                    }
                }
                continue;
            }
            key.push_str(op);
            continue;
        }

        if is_register(op) || is_condition(op) {
            key.push_str(op);
            continue;
        }

        if let Some(v) = parse_number(op) {
            match mnemonic {
                "BIT" | "RES" | "SET" => {
                    key.push_str(&format!("{}", v));
                }
                "LD" => {
                    let other_idx = if i == 0 && num_ops > 1 {
                        1
                    } else if i == 1 {
                        0
                    } else {
                        usize::MAX
                    };
                    let is_16bit = if other_idx != usize::MAX {
                        is_16bit_reg(operands[other_idx])
                    } else {
                        false
                    };
                    if is_16bit {
                        key.push_str("NNNN");
                        values.push((v & 0xFF) as i8);
                        values.push((v >> 8) as i8);
                    } else {
                        key.push_str("NN");
                        values.push(v as i8);
                    }
                }
                "JP" | "CALL" => {
                    key.push_str("NNNN");
                    values.push((v & 0xFF) as i8);
                    values.push((v >> 8) as i8);
                }
                _ => {
                    // ADD, SUB, AND, OR, XOR, CP, ADC, SBC, etc.
                    key.push_str("NN");
                    values.push(v as i8);
                }
            }
            continue;
        }

        // Unknown token - pass through literally
        key.push_str(op);
    }

    Some((key, values))
}

// ── instruction encoding ───────────────────────────────────────────

fn opcode_bytes(key: u32) -> Vec<u8> {
    if (key >> 8) == 0xFDCB {
        vec![0xFD, 0xCB, (key & 0xFF) as u8]
    } else if (key >> 8) == 0xDDCB {
        vec![0xDD, 0xCB, (key & 0xFF) as u8]
    } else if (key >> 8) == 0xFD {
        vec![0xFD, (key & 0xFF) as u8]
    } else if (key >> 8) == 0xDD {
        vec![0xDD, (key & 0xFF) as u8]
    } else if (key >> 8) == 0xED {
        vec![0xED, (key & 0xFF) as u8]
    } else if (key >> 8) == 0xCB {
        vec![0xCB, (key & 0xFF) as u8]
    } else {
        vec![key as u8]
    }
}

fn write_operand_bytes(template: &str, values: &[i8], mut idx: usize) -> (Vec<u8>, usize) {
    let mut bytes = Vec::new();
    let chars: Vec<char> = template.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 3 < chars.len() && chars[i..i + 4] == ['n', 'n', 'n', 'n'] {
            let lo = values[idx] as u8;
            let hi = values[idx + 1] as u8;
            bytes.push(lo);
            bytes.push(hi);
            i += 4;
            idx += 2;
        } else if i + 5 < chars.len() && chars[i..i + 6] == ['o', 'f', 'f', 's', 'e', 't'] {
            bytes.push(values[idx] as u8);
            i += 6;
            idx += 1;
        } else if i + 1 < chars.len() && chars[i..i + 2] == ['d', 'd'] {
            bytes.push(values[idx] as u8);
            i += 2;
            idx += 1;
        } else if i + 1 < chars.len() && chars[i..i + 2] == ['n', 'n'] {
            bytes.push(values[idx] as u8);
            i += 2;
            idx += 1;
        } else {
            i += 1;
        }
    }
    (bytes, idx)
}

// ── public API ───────────────────────────────────────────────────

pub fn assemble_line<M: CpuBus>(mmu: &mut M, addr: u16, line: &str) -> Result<AsmResult, String> {
    let (mnemonic, params) = normalize_input(line);

    if mnemonic.is_empty() {
        return Err("Empty instruction".to_string());
    }

    let (canonical, values) = build_canonical(&mnemonic, &params)
        .ok_or_else(|| format!("Unknown instruction: {}", line))?;

    let op_count = user_operand_count(&params);
    let mn_to_op = mnemonic_to_opcode();

    let key = *mn_to_op
        .get(&(canonical.clone(), op_count))
        .or_else(|| mn_to_op.get(&(canonical, 0))) // fallback for no-operand exact match
        .ok_or_else(|| format!("Unknown instruction: {}", line))?;

    let template = opcode_to_mnemonic()
        .get(&key)
        .cloned()
        .ok_or("Internal error: missing template")?;

    let mut bytes = opcode_bytes(key);
    let (op_bytes, _) = write_operand_bytes(&template, &values, 0);
    bytes.extend(op_bytes);

    // For DDCB/FDCB, the standard Z80 byte order is: prefix1, prefix2, displacement, opcode
    if matches!(key >> 8, 0xDDCB | 0xFDCB) && bytes.len() >= 4 {
        bytes.swap(2, 3);
    }

    for (i, &b) in bytes.iter().enumerate() {
        mmu.w8(addr.wrapping_add(i as u16), b);
    }

    Ok(AsmResult { addr, bytes })
}
