use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmError {
    message: String,
}

impl AsmError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AsmError {}

/// Assemble one Z80 instruction at `pc`.
///
/// The program counter is needed for the relative displacement used by JR and
/// DJNZ. Labels, expressions, macros, and multi-line source are intentionally
/// outside this small assembler's scope.
pub fn assemble_line(source: &str, pc: u16) -> Result<Vec<u8>, AsmError> {
    let source = source.split(';').next().unwrap_or("").trim();
    if source.is_empty() {
        return Err(AsmError::new("empty assembly line"));
    }

    let split = source.find(char::is_whitespace).unwrap_or(source.len());
    let mnemonic = source[..split].to_ascii_uppercase();
    let operands = split_operands(source[split..].trim())?;
    let ops: Vec<String> = operands.iter().map(|op| normalize(op)).collect();
    let op: Vec<&str> = ops.iter().map(String::as_str).collect();

    let bytes = match mnemonic.as_str() {
        "DB" | "DEFB" => assemble_db(&op)?,
        "LD" => assemble_ld(&op)?,
        "INC" => assemble_inc_dec(&op, false)?,
        "DEC" => assemble_inc_dec(&op, true)?,
        "ADD" => assemble_add(&op)?,
        "ADC" => assemble_adc_sbc(&op, false)?,
        "SBC" => assemble_adc_sbc(&op, true)?,
        "SUB" => assemble_alu(&op, 2)?,
        "AND" => assemble_alu(&op, 4)?,
        "XOR" => assemble_alu(&op, 5)?,
        "OR" => assemble_alu(&op, 6)?,
        "CP" => assemble_alu(&op, 7)?,
        "JP" => assemble_jp(&op)?,
        "JR" => assemble_jr(&op, pc)?,
        "DJNZ" => {
            expect_count(&op, 1)?;
            vec![0x10, relative(op[0], pc)?]
        }
        "CALL" => assemble_call(&op)?,
        "RET" => assemble_ret(&op)?,
        "RST" => assemble_rst(&op)?,
        "PUSH" => assemble_push_pop(&op, true)?,
        "POP" => assemble_push_pop(&op, false)?,
        "EX" => assemble_ex(&op)?,
        "IN" => assemble_in(&op)?,
        "OUT" => assemble_out(&op)?,
        "IM" => assemble_im(&op)?,
        "BIT" => assemble_bit(&op, 1)?,
        "RES" => assemble_bit(&op, 2)?,
        "SET" => assemble_bit(&op, 3)?,
        "RLC" => assemble_rotate(&op, 0)?,
        "RRC" => assemble_rotate(&op, 1)?,
        "RL" => assemble_rotate(&op, 2)?,
        "RR" => assemble_rotate(&op, 3)?,
        "SLA" => assemble_rotate(&op, 4)?,
        "SRA" => assemble_rotate(&op, 5)?,
        "SLL" => assemble_rotate(&op, 6)?,
        "SRL" => assemble_rotate(&op, 7)?,
        _ => assemble_fixed(&mnemonic, &op)?,
    };

    Ok(bytes)
}

fn split_operands(source: &str) -> Result<Vec<String>, AsmError> {
    if source.is_empty() {
        return Ok(Vec::new());
    }

    let mut depth = 0u8;
    let mut start = 0usize;
    let mut operands = Vec::new();
    for (index, ch) in source.char_indices() {
        match ch {
            '(' => depth = depth.saturating_add(1),
            ')' => {
                if depth == 0 {
                    return Err(AsmError::new("unmatched ')'"));
                }
                depth -= 1;
            }
            ',' if depth == 0 => {
                let operand = source[start..index].trim();
                if operand.is_empty() {
                    return Err(AsmError::new("missing operand"));
                }
                operands.push(operand.to_string());
                start = index + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(AsmError::new("unmatched '('"));
    }

    let operand = source[start..].trim();
    if operand.is_empty() {
        return Err(AsmError::new("missing operand"));
    }
    operands.push(operand.to_string());
    Ok(operands)
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .flat_map(char::to_uppercase)
        .collect()
}

fn assemble_db(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    if op.is_empty() {
        return Err(AsmError::new("DB requires at least one byte"));
    }
    op.iter().map(|value| byte(value)).collect()
}

fn assemble_ld(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    let dst = op[0];
    let src = op[1];

    if let (Some(d), Some(s)) = (reg8(dst), reg8(src)) {
        if d == 6 && s == 6 {
            return Err(AsmError::new("LD (HL),(HL) is not an instruction"));
        }
        return Ok(vec![0x40 | (d << 3) | s]);
    }
    match (dst, src) {
        ("(BC)", "A") => return Ok(vec![0x02]),
        ("A", "(BC)") => return Ok(vec![0x0A]),
        ("(DE)", "A") => return Ok(vec![0x12]),
        ("A", "(DE)") => return Ok(vec![0x1A]),
        ("SP", "HL") => return Ok(vec![0xF9]),
        ("I", "A") => return Ok(vec![0xED, 0x47]),
        ("R", "A") => return Ok(vec![0xED, 0x4F]),
        ("A", "I") => return Ok(vec![0xED, 0x57]),
        ("A", "R") => return Ok(vec![0xED, 0x5F]),
        ("SP", "IX") => return Ok(vec![0xDD, 0xF9]),
        ("SP", "IY") => return Ok(vec![0xFD, 0xF9]),
        _ => {}
    }

    if let Some(addr) = indirect_word(dst) {
        if src == "A" {
            let mut out = vec![0x32];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if src == "HL" {
            let mut out = vec![0x22];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if let Some(prefix) = index_prefix(src) {
            let mut out = vec![prefix, 0x22];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if let Some(p) = reg16(src) {
            let mut out = vec![0xED, 0x43 | (p << 4)];
            push_word(&mut out, addr);
            return Ok(out);
        }
    }

    if let Some(addr) = indirect_word(src) {
        if dst == "A" {
            let mut out = vec![0x3A];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if dst == "HL" {
            let mut out = vec![0x2A];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if let Some(prefix) = index_prefix(dst) {
            let mut out = vec![prefix, 0x2A];
            push_word(&mut out, addr);
            return Ok(out);
        }
        if let Some(p) = reg16(dst) {
            let mut out = vec![0xED, 0x4B | (p << 4)];
            push_word(&mut out, addr);
            return Ok(out);
        }
    }

    if let Some((prefix, d)) = indexed(dst)? {
        if let Some(s) = reg8(src) {
            return Ok(vec![prefix, 0x70 | s, d as u8]);
        }
        return Ok(vec![prefix, 0x36, d as u8, byte(src)?]);
    }
    if let Some((prefix, d)) = indexed(src)? {
        if let Some(r) = reg8(dst) {
            return Ok(vec![prefix, 0x46 | (r << 3), d as u8]);
        }
    }

    if let Some(d) = reg8(dst) {
        return Ok(vec![0x06 | (d << 3), byte(src)?]);
    }
    if let Some(p) = reg16(dst) {
        let mut out = vec![0x01 | (p << 4)];
        push_word(&mut out, word(src)?);
        return Ok(out);
    }
    if let Some(prefix) = index_prefix(dst) {
        let mut out = vec![prefix, 0x21];
        push_word(&mut out, word(src)?);
        return Ok(out);
    }

    Err(form_error("LD", op))
}

fn assemble_inc_dec(op: &[&str], decrement: bool) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 1)?;
    let base = if decrement { 0x05 } else { 0x04 };
    if let Some(r) = reg8(op[0]) {
        return Ok(vec![base | (r << 3)]);
    }
    if let Some(p) = reg16(op[0]) {
        return Ok(vec![(if decrement { 0x0B } else { 0x03 }) | (p << 4)]);
    }
    if let Some(prefix) = index_prefix(op[0]) {
        return Ok(vec![prefix, if decrement { 0x2B } else { 0x23 }]);
    }
    if let Some((prefix, d)) = indexed(op[0])? {
        return Ok(vec![prefix, base | (6 << 3), d as u8]);
    }
    Err(form_error(if decrement { "DEC" } else { "INC" }, op))
}

fn assemble_add(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    if op[0] == "HL" {
        if let Some(p) = reg16(op[1]) {
            return Ok(vec![0x09 | (p << 4)]);
        }
    }
    if let Some(prefix) = index_prefix(op[0]) {
        let p = match op[1] {
            "BC" => 0,
            "DE" => 1,
            value if value == op[0] => 2,
            "SP" => 3,
            _ => return Err(form_error("ADD", op)),
        };
        return Ok(vec![prefix, 0x09 | (p << 4)]);
    }
    if op[0] == "A" {
        return assemble_alu(&op[1..], 0);
    }
    Err(form_error("ADD", op))
}

fn assemble_adc_sbc(op: &[&str], subtract: bool) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    if op[0] == "HL" {
        if let Some(p) = reg16(op[1]) {
            let base = if subtract { 0x42 } else { 0x4A };
            return Ok(vec![0xED, base | (p << 4)]);
        }
    }
    if op[0] == "A" {
        return assemble_alu(&op[1..], if subtract { 3 } else { 1 });
    }
    Err(form_error(if subtract { "SBC" } else { "ADC" }, op))
}

fn assemble_alu(op: &[&str], operation: u8) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 1)?;
    if let Some(r) = reg8(op[0]) {
        return Ok(vec![0x80 | (operation << 3) | r]);
    }
    if let Some((prefix, d)) = indexed(op[0])? {
        return Ok(vec![prefix, 0x80 | (operation << 3) | 6, d as u8]);
    }
    Ok(vec![0xC6 | (operation << 3), byte(op[0])?])
}

fn assemble_jp(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    match op {
        ["(HL)"] => Ok(vec![0xE9]),
        ["(IX)"] => Ok(vec![0xDD, 0xE9]),
        ["(IY)"] => Ok(vec![0xFD, 0xE9]),
        [target] => {
            let mut out = vec![0xC3];
            push_word(&mut out, word(target)?);
            Ok(out)
        }
        [condition, target] => {
            let cc = condition_code(condition)
                .ok_or_else(|| AsmError::new(format!("invalid JP condition '{}'", condition)))?;
            let mut out = vec![0xC2 | (cc << 3)];
            push_word(&mut out, word(target)?);
            Ok(out)
        }
        _ => Err(form_error("JP", op)),
    }
}

fn assemble_jr(op: &[&str], pc: u16) -> Result<Vec<u8>, AsmError> {
    match op {
        [target] => Ok(vec![0x18, relative(target, pc)?]),
        [condition, target] => {
            let cc = match *condition {
                "NZ" => 0,
                "Z" => 1,
                "NC" => 2,
                "C" => 3,
                _ => {
                    return Err(AsmError::new(format!(
                        "JR condition must be NZ, Z, NC, or C, got '{}'",
                        condition
                    )));
                }
            };
            Ok(vec![0x20 | (cc << 3), relative(target, pc)?])
        }
        _ => Err(form_error("JR", op)),
    }
}

fn assemble_call(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    let (opcode, target) = match op {
        [target] => (0xCD, *target),
        [condition, target] => {
            let cc = condition_code(condition)
                .ok_or_else(|| AsmError::new(format!("invalid CALL condition '{}'", condition)))?;
            (0xC4 | (cc << 3), *target)
        }
        _ => return Err(form_error("CALL", op)),
    };
    let mut out = vec![opcode];
    push_word(&mut out, word(target)?);
    Ok(out)
}

fn assemble_ret(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    match op {
        [] => Ok(vec![0xC9]),
        [condition] => {
            let cc = condition_code(condition)
                .ok_or_else(|| AsmError::new(format!("invalid RET condition '{}'", condition)))?;
            Ok(vec![0xC0 | (cc << 3)])
        }
        _ => Err(form_error("RET", op)),
    }
}

fn assemble_rst(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 1)?;
    let address = number(op[0])?;
    if !(0..=0x38).contains(&address) || address % 8 != 0 {
        return Err(AsmError::new("RST address must be 00H, 08H, ..., or 38H"));
    }
    Ok(vec![0xC7 | address as u8])
}

fn assemble_push_pop(op: &[&str], push: bool) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 1)?;
    if let Some(prefix) = index_prefix(op[0]) {
        return Ok(vec![prefix, if push { 0xE5 } else { 0xE1 }]);
    }
    let p = reg16_stack(op[0]).ok_or_else(|| {
        AsmError::new(format!(
            "{} requires BC, DE, HL, AF, IX, or IY",
            if push { "PUSH" } else { "POP" }
        ))
    })?;
    Ok(vec![(if push { 0xC5 } else { 0xC1 }) | (p << 4)])
}

fn assemble_ex(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    match (op[0], op[1]) {
        ("AF", "AF'") => Ok(vec![0x08]),
        ("DE", "HL") => Ok(vec![0xEB]),
        ("(SP)", "HL") => Ok(vec![0xE3]),
        ("(SP)", "IX") => Ok(vec![0xDD, 0xE3]),
        ("(SP)", "IY") => Ok(vec![0xFD, 0xE3]),
        _ => Err(form_error("EX", op)),
    }
}

fn assemble_in(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    if op[0] == "A" {
        if let Some(port) = parenthesized_byte(op[1]) {
            if let Ok(port) = port {
                return Ok(vec![0xDB, port]);
            }
        }
    }
    if op[1] == "(C)" {
        let r = match op[0] {
            "F" => 6,
            value => reg8(value)
                .filter(|r| *r != 6)
                .ok_or_else(|| AsmError::new(format!("invalid IN register '{}'", value)))?,
        };
        return Ok(vec![0xED, 0x40 | (r << 3)]);
    }
    Err(form_error("IN", op))
}

fn assemble_out(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    if op[1] == "A" {
        if let Some(port) = parenthesized_byte(op[0]) {
            if let Ok(port) = port {
                return Ok(vec![0xD3, port]);
            }
        }
    }
    if op[0] == "(C)" {
        if op[1] == "0" {
            return Ok(vec![0xED, 0x71]);
        }
        let r = reg8(op[1])
            .filter(|r| *r != 6)
            .ok_or_else(|| AsmError::new(format!("invalid OUT register '{}'", op[1])))?;
        return Ok(vec![0xED, 0x41 | (r << 3)]);
    }
    Err(form_error("OUT", op))
}

fn assemble_im(op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 1)?;
    let opcode = match number(op[0])? {
        0 => 0x46,
        1 => 0x56,
        2 => 0x5E,
        _ => return Err(AsmError::new("IM mode must be 0, 1, or 2")),
    };
    Ok(vec![0xED, opcode])
}

fn assemble_bit(op: &[&str], group: u8) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 2)?;
    let bit = number(op[0])?;
    if !(0..=7).contains(&bit) {
        return Err(AsmError::new("bit number must be between 0 and 7"));
    }
    let operation = (group << 6) | ((bit as u8) << 3);
    if let Some(r) = reg8(op[1]) {
        return Ok(vec![0xCB, operation | r]);
    }
    if let Some((prefix, d)) = indexed(op[1])? {
        return Ok(vec![prefix, 0xCB, d as u8, operation | 6]);
    }
    Err(form_error(
        match group {
            1 => "BIT",
            2 => "RES",
            _ => "SET",
        },
        op,
    ))
}

fn assemble_rotate(op: &[&str], operation: u8) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 1)?;
    if let Some(r) = reg8(op[0]) {
        return Ok(vec![0xCB, (operation << 3) | r]);
    }
    if let Some((prefix, d)) = indexed(op[0])? {
        return Ok(vec![prefix, 0xCB, d as u8, (operation << 3) | 6]);
    }
    Err(AsmError::new(format!(
        "invalid rotate/shift operand '{}'",
        op[0]
    )))
}

fn assemble_fixed(mnemonic: &str, op: &[&str]) -> Result<Vec<u8>, AsmError> {
    expect_count(op, 0)?;
    let bytes: &[u8] = match mnemonic {
        "NOP" => &[0x00],
        "EXX" => &[0xD9],
        "HALT" => &[0x76],
        "RLCA" => &[0x07],
        "RRCA" => &[0x0F],
        "RLA" => &[0x17],
        "RRA" => &[0x1F],
        "DAA" => &[0x27],
        "CPL" => &[0x2F],
        "SCF" => &[0x37],
        "CCF" => &[0x3F],
        "DI" => &[0xF3],
        "EI" => &[0xFB],
        "NEG" => &[0xED, 0x44],
        "RETN" => &[0xED, 0x45],
        "RETI" => &[0xED, 0x4D],
        "RRD" => &[0xED, 0x67],
        "RLD" => &[0xED, 0x6F],
        "LDI" => &[0xED, 0xA0],
        "CPI" => &[0xED, 0xA1],
        "INI" => &[0xED, 0xA2],
        "OUTI" => &[0xED, 0xA3],
        "LDD" => &[0xED, 0xA8],
        "CPD" => &[0xED, 0xA9],
        "IND" => &[0xED, 0xAA],
        "OUTD" => &[0xED, 0xAB],
        "LDIR" => &[0xED, 0xB0],
        "CPIR" => &[0xED, 0xB1],
        "INIR" => &[0xED, 0xB2],
        "OTIR" => &[0xED, 0xB3],
        "LDDR" => &[0xED, 0xB8],
        "CPDR" => &[0xED, 0xB9],
        "INDR" => &[0xED, 0xBA],
        "OTDR" => &[0xED, 0xBB],
        _ => return Err(AsmError::new(format!("unknown mnemonic '{}'", mnemonic))),
    };
    Ok(bytes.to_vec())
}

fn reg8(value: &str) -> Option<u8> {
    match value {
        "B" => Some(0),
        "C" => Some(1),
        "D" => Some(2),
        "E" => Some(3),
        "H" => Some(4),
        "L" => Some(5),
        "(HL)" => Some(6),
        "A" => Some(7),
        _ => None,
    }
}

fn reg16(value: &str) -> Option<u8> {
    match value {
        "BC" => Some(0),
        "DE" => Some(1),
        "HL" => Some(2),
        "SP" => Some(3),
        _ => None,
    }
}

fn reg16_stack(value: &str) -> Option<u8> {
    match value {
        "BC" => Some(0),
        "DE" => Some(1),
        "HL" => Some(2),
        "AF" => Some(3),
        _ => None,
    }
}

fn condition_code(value: &str) -> Option<u8> {
    match value {
        "NZ" => Some(0),
        "Z" => Some(1),
        "NC" => Some(2),
        "C" => Some(3),
        "PO" => Some(4),
        "PE" => Some(5),
        "P" => Some(6),
        "M" => Some(7),
        _ => None,
    }
}

fn index_prefix(value: &str) -> Option<u8> {
    match value {
        "IX" => Some(0xDD),
        "IY" => Some(0xFD),
        _ => None,
    }
}

fn indexed(value: &str) -> Result<Option<(u8, i8)>, AsmError> {
    if !value.starts_with('(') || !value.ends_with(')') {
        return Ok(None);
    }
    let inner = &value[1..value.len() - 1];
    let (prefix, rest) = if let Some(rest) = inner.strip_prefix("IX") {
        (0xDD, rest)
    } else if let Some(rest) = inner.strip_prefix("IY") {
        (0xFD, rest)
    } else {
        return Ok(None);
    };

    let displacement = if rest.is_empty() {
        0
    } else {
        let value = number(rest)?;
        i8::try_from(value).map_err(|_| {
            AsmError::new(format!(
                "index displacement '{}' is outside -128..127",
                rest
            ))
        })?
    };
    Ok(Some((prefix, displacement)))
}

fn indirect_word(value: &str) -> Option<u16> {
    if !value.starts_with('(') || !value.ends_with(')') {
        return None;
    }
    word(&value[1..value.len() - 1]).ok()
}

fn parenthesized_byte(value: &str) -> Option<Result<u8, AsmError>> {
    if !value.starts_with('(') || !value.ends_with(')') {
        return None;
    }
    Some(byte(&value[1..value.len() - 1]))
}

fn number(value: &str) -> Result<i32, AsmError> {
    let value = value.replace('_', "");
    let (sign, unsigned) = if let Some(rest) = value.strip_prefix('-') {
        (-1, rest)
    } else if let Some(rest) = value.strip_prefix('+') {
        (1, rest)
    } else {
        (1, value.as_str())
    };
    if unsigned.is_empty() {
        return Err(AsmError::new(format!("invalid number '{}'", value)));
    }

    let (digits, radix) = if let Some(rest) = unsigned.strip_prefix("0X") {
        (rest, 16)
    } else if let Some(rest) = unsigned.strip_prefix('$') {
        (rest, 16)
    } else if let Some(rest) = unsigned.strip_suffix('H') {
        (rest, 16)
    } else if let Some(rest) = unsigned.strip_prefix("0B") {
        (rest, 2)
    } else if let Some(rest) = unsigned.strip_suffix('B') {
        (rest, 2)
    } else {
        (unsigned, 10)
    };

    i32::from_str_radix(digits, radix)
        .map(|number| sign * number)
        .map_err(|_| AsmError::new(format!("invalid number '{}'", value)))
}

fn byte(value: &str) -> Result<u8, AsmError> {
    let value_number = number(value)?;
    u8::try_from(value_number)
        .map_err(|_| AsmError::new(format!("byte '{}' is outside 0..255", value)))
}

fn word(value: &str) -> Result<u16, AsmError> {
    let value_number = number(value)?;
    u16::try_from(value_number)
        .map_err(|_| AsmError::new(format!("word '{}' is outside 0..65535", value)))
}

fn relative(target: &str, pc: u16) -> Result<u8, AsmError> {
    let target = word(target)?;
    let next = pc.wrapping_add(2);
    let displacement = target.wrapping_sub(next) as i16;
    if !(-128..=127).contains(&displacement) {
        return Err(AsmError::new(format!(
            "relative target {:04X}H is out of range from {:04X}H",
            target, pc
        )));
    }
    Ok(displacement as i8 as u8)
}

fn push_word(out: &mut Vec<u8>, value: u16) {
    out.push(value as u8);
    out.push((value >> 8) as u8);
}

fn expect_count(op: &[&str], count: usize) -> Result<(), AsmError> {
    if op.len() == count {
        Ok(())
    } else {
        Err(AsmError::new(format!(
            "expected {} operand{}, got {}",
            count,
            if count == 1 { "" } else { "s" },
            op.len()
        )))
    }
}

fn form_error(mnemonic: &str, op: &[&str]) -> AsmError {
    let suffix = if op.is_empty() {
        String::new()
    } else {
        format!(" {}", op.join(","))
    };
    AsmError::new(format!("unsupported instruction '{}{}'", mnemonic, suffix))
}

#[cfg(test)]
#[path = "asm_tests.rs"]
mod tests;
