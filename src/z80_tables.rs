use std::collections::HashMap;
use std::sync::OnceLock;

fn parse_dat(content: &str) -> Vec<(u32, String)> {
    let mut result = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(space_idx) = line.find(' ') else { continue };
        let hex_part = line[..space_idx].trim();
        let mnem_part = line[space_idx + 1..].trim();
        if hex_part.is_empty() || mnem_part.is_empty() {
            continue;
        }
        let subkey = match u32::from_str_radix(&hex_part[2..], 16) {
            Ok(v) => v,
            Err(_) => continue,
        };
        result.push((subkey, mnem_part.to_string()));
    }
    result
}

fn count_template_operands(mnem: &str) -> usize {
    let comma_count = mnem.chars().filter(|&c| c == ',').count();
    if comma_count > 0 {
        comma_count + 1
    } else {
        let token_count = mnem.split_whitespace().count();
        if token_count > 1 { 1 } else { 0 }
    }
}

fn build_tables() -> (HashMap<u32, String>, HashMap<(String, usize), u32>) {
    let mut op_to_mn: HashMap<u32, String> = HashMap::new();
    let mut mn_to_op: HashMap<(String, usize), u32> = HashMap::new();

    for (content, base) in [
        (include_str!("../docs/opcodes_base.dat"), 0u32),
        (include_str!("../docs/opcodes_cb.dat"), 0xCB00u32),
        (include_str!("../docs/opcodes_ed.dat"), 0xED00u32),
    ] {
        for (subkey, mnem) in parse_dat(content) {
            let key = base + subkey;
            op_to_mn.insert(key, mnem.clone());
            let cleaned = mnem.replace([' ', '\t'], "").to_ascii_uppercase();
            let op_count = count_template_operands(&mnem);
            mn_to_op.insert((cleaned.clone(), op_count), key);

            // Add implicit-A aliases for ALU instructions with explicit A in template
            let first_token = mnem.split_whitespace().next().unwrap_or("");
            if matches!(first_token, "ADD" | "ADC" | "SUB" | "SBC" | "AND" | "XOR" | "OR" | "CP")
                && cleaned.contains("A,")
            {
                let implicit = cleaned.replace("A,", "");
                mn_to_op.insert((implicit, op_count.saturating_sub(1)), key);
            }

            // Add decimal alias for RST
            if let Some(v) = cleaned.strip_prefix("RST").and_then(|s| u32::from_str_radix(s, 16).ok()) {
                let dec_key = format!("RST{}", v);
                mn_to_op.insert((dec_key, op_count), key);
            }
        }
    }

    for (content, ix_prefix, iy_prefix) in [
        (
            include_str!("../docs/opcodes_ddfd.dat"),
            0xDD00u32,
            0xFD00u32,
        ),
        (
            include_str!("../docs/opcodes_ddfdcb.dat"),
            0xDDCB00u32,
            0xFDCB00u32,
        ),
    ] {
        for (subkey, template) in parse_dat(content) {
            let ix_key = ix_prefix + subkey;
            let ix_mnem = template
                .replace("REGISTERH", "IXh")
                .replace("REGISTERL", "IXl")
                .replace("REGISTER", "IX");
            op_to_mn.insert(ix_key, ix_mnem.clone());
            let ix_clean = ix_mnem.replace([' ', '\t'], "").to_ascii_uppercase();
            let ix_count = count_template_operands(&ix_mnem);
            mn_to_op.insert((ix_clean.clone(), ix_count), ix_key);

            // Implicit-A alias for IX variants
            let first_token = ix_mnem.split_whitespace().next().unwrap_or("");
            if matches!(first_token, "ADD" | "ADC" | "SUB" | "SBC" | "AND" | "XOR" | "OR" | "CP")
                && ix_clean.contains("A,")
            {
                let implicit = ix_clean.replace("A,", "");
                mn_to_op.insert((implicit, ix_count.saturating_sub(1)), ix_key);
            }

            let iy_key = iy_prefix + subkey;
            let iy_mnem = template
                .replace("REGISTERH", "IYh")
                .replace("REGISTERL", "IYl")
                .replace("REGISTER", "IY");
            op_to_mn.insert(iy_key, iy_mnem.clone());
            let iy_clean = iy_mnem.replace([' ', '\t'], "").to_ascii_uppercase();
            let iy_count = count_template_operands(&iy_mnem);
            mn_to_op.insert((iy_clean.clone(), iy_count), iy_key);

            // Implicit-A alias for IY variants
            let first_token = iy_mnem.split_whitespace().next().unwrap_or("");
            if matches!(first_token, "ADD" | "ADC" | "SUB" | "SBC" | "AND" | "XOR" | "OR" | "CP")
                && iy_clean.contains("A,")
            {
                let implicit = iy_clean.replace("A,", "");
                mn_to_op.insert((implicit, iy_count.saturating_sub(1)), iy_key);
            }
        }
    }

    (op_to_mn, mn_to_op)
}

pub fn opcode_to_mnemonic() -> &'static HashMap<u32, String> {
    static MAP: OnceLock<HashMap<u32, String>> = OnceLock::new();
    MAP.get_or_init(|| build_tables().0)
}

pub fn mnemonic_to_opcode() -> &'static HashMap<(String, usize), u32> {
    static MAP: OnceLock<HashMap<(String, usize), u32>> = OnceLock::new();
    MAP.get_or_init(|| build_tables().1)
}
