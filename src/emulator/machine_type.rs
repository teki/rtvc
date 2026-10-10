use crate::snapshot::{Reader, SnapshotError, Writer};
use crate::tvc::Tvc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RomVersion {
    V1_2,
    V2_2,
}

const CART_ID_MAX: usize = 32;

/// Stem of a `roms/<stem>.crt` image for the cartridge socket.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CartId {
    bytes: [u8; CART_ID_MAX],
    len: u8,
}

impl CartId {
    pub fn new(stem: &str) -> Option<Self> {
        if stem.is_empty() || stem.len() > CART_ID_MAX {
            return None;
        }
        if !stem
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return None;
        }
        let mut bytes = [0; CART_ID_MAX];
        bytes[..stem.len()].copy_from_slice(stem.as_bytes());
        Some(Self {
            bytes,
            len: stem.len() as u8,
        })
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len as usize]).unwrap_or("")
    }

    pub fn file_name(&self) -> String {
        format!("{}.crt", self.as_str())
    }
}

/// Card plugged into one of the four expansion connectors.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExpansionCard {
    VtDos12,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum RomUse {
    System,
    Cart,
    Hbf(usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MachineType {
    pub is_plus: bool,
    pub rom_version: RomVersion,
    pub cart: Option<CartId>,
    pub slots: [Option<ExpansionCard>; 4],
}

impl MachineType {
    /// Memory and system ROM, with the HBF card in slot 0 when `hbf_slot0` is set.
    pub fn standard(is_plus: bool, rom_version: RomVersion, hbf_slot0: bool) -> Self {
        let mut machine = Self {
            is_plus,
            rom_version,
            cart: None,
            slots: [None; 4],
        };
        if hbf_slot0 {
            machine.slots[0] = Some(ExpansionCard::VtDos12);
        }
        machine
    }

    pub fn with_hbf_slot(mut self, slot: usize) -> Self {
        self.set_hbf_slot(Some(slot));
        self
    }

    pub fn with_cart(mut self, stem: &str) -> Self {
        self.cart = CartId::new(stem);
        self
    }

    pub fn set_hbf_slot(&mut self, slot: Option<usize>) {
        self.slots = [None; 4];
        if let Some(slot) = slot.filter(|slot| *slot < self.slots.len()) {
            self.slots[slot] = Some(ExpansionCard::VtDos12);
        }
    }

    pub fn hbf_slot(&self) -> Option<usize> {
        self.slots.iter().position(|slot| slot.is_some())
    }

    pub fn label(&self) -> String {
        self.preset_label()
            .map(str::to_string)
            .unwrap_or_else(|| self.id())
    }

    pub fn preset_label(&self) -> Option<&'static str> {
        if self.cart.is_some() {
            return None;
        }
        match (self.is_plus, self.rom_version, self.hbf_slot()) {
            (true, RomVersion::V1_2, Some(0)) => Some("64k+ 1.2, VT-DOS"),
            (true, RomVersion::V2_2, Some(0)) => Some("64k+ 2.2, VT-DOS"),
            (false, RomVersion::V1_2, None) => Some("64k  1.2"),
            (true, RomVersion::V1_2, None) => Some("64k+ 1.2"),
            (true, RomVersion::V2_2, None) => Some("64k+ 2.2"),
            _ => None,
        }
    }

    pub fn id(&self) -> String {
        let mut id = format!(
            "{}{}",
            if self.is_plus { "64k+" } else { "64k" },
            match self.rom_version {
                RomVersion::V1_2 => "tvc12",
                RomVersion::V2_2 => "tvc22",
            }
        );
        for (slot, card) in self.slots.iter().enumerate() {
            if let Some(ExpansionCard::VtDos12) = card {
                id.push_str(&format!("+ex{slot}:vtdos12"));
            }
        }
        if let Some(cart) = self.cart {
            id.push('+');
            id.push_str("cart:");
            id.push_str(cart.as_str());
        }
        id
    }

    pub fn from_id(id: &str) -> Option<Self> {
        if let Some(machine) = Self::from_legacy_id(id) {
            return Some(machine);
        }
        let (is_plus, rest) = if let Some(rest) = id.strip_prefix("64k+") {
            (true, rest)
        } else if let Some(rest) = id.strip_prefix("64k") {
            (false, rest)
        } else {
            return None;
        };
        let (rom_version, rest) = if let Some(rest) = rest.strip_prefix("tvc12") {
            (RomVersion::V1_2, rest)
        } else if let Some(rest) = rest.strip_prefix("tvc22") {
            (RomVersion::V2_2, rest)
        } else {
            return None;
        };
        let mut machine = Self::standard(is_plus, rom_version, false);
        if rest.is_empty() {
            return Some(machine);
        }
        for part in rest.strip_prefix('+')?.split('+') {
            if let Some(slot) = part.strip_prefix("ex") {
                let (index, card) = slot.split_once(':')?;
                let index: usize = index.parse().ok()?;
                if card != "vtdos12" || machine.hbf_slot().is_some() {
                    return None;
                }
                machine.set_hbf_slot(Some(index));
                if machine.hbf_slot() != Some(index) {
                    return None;
                }
            } else if let Some(stem) = part.strip_prefix("cart:") {
                if machine.cart.is_some() {
                    return None;
                }
                let stem = if stem == "vtdos12" {
                    "VT-DOS12-CART"
                } else {
                    stem
                };
                machine.cart = Some(CartId::new(stem)?);
            } else {
                return None;
            }
        }
        Some(machine)
    }

    fn from_legacy_id(id: &str) -> Option<Self> {
        Some(match id {
            "64k-plus-1.2-vtdos" => Self::standard(true, RomVersion::V1_2, true),
            "64k-plus-2.2-vtdos" => Self::standard(true, RomVersion::V2_2, true),
            "64k-1.2" => Self::standard(false, RomVersion::V1_2, false),
            "64k-plus-1.2" => Self::standard(true, RomVersion::V1_2, false),
            "64k-plus-2.2" => Self::standard(true, RomVersion::V2_2, false),
            _ => return None,
        })
    }

    pub fn all_types() -> Vec<MachineType> {
        vec![
            Self::standard(true, RomVersion::V1_2, true),
            Self::standard(true, RomVersion::V2_2, true),
            Self::standard(false, RomVersion::V1_2, false),
            Self::standard(true, RomVersion::V1_2, false),
            Self::standard(true, RomVersion::V2_2, false),
        ]
    }

    pub fn for_snapshot(is_plus: bool, has_dos: bool, preferred_rom_version: RomVersion) -> Self {
        Self::standard(is_plus, preferred_rom_version, has_dos)
    }

    #[cfg_attr(not(any(feature = "native", feature = "wasm-full")), allow(dead_code))]
    fn rom_version_id(&self) -> u8 {
        match self.rom_version {
            RomVersion::V1_2 => 0,
            RomVersion::V2_2 => 1,
        }
    }

    pub(crate) fn from_snapshot_chunk(data: &[u8]) -> crate::snapshot::Result<Self> {
        let mut reader = Reader::new(data);
        let is_plus = reader.u8()? != 0;
        let rom_version = match reader.u8()? {
            0 => RomVersion::V1_2,
            1 => RomVersion::V2_2,
            value => {
                return Err(SnapshotError::InvalidData(format!(
                    "unknown machine ROM version {value}"
                )));
            }
        };
        let slot0_hbf = reader.u8()? != 0;
        if reader.remaining() == 0 {
            return Ok(Self::standard(is_plus, rom_version, slot0_hbf));
        }
        let version = reader.u8()?;
        let cart = match version {
            1 => match reader.u8()? {
                0 => None,
                1 => CartId::new("VT-DOS12-CART"),
                value => {
                    return Err(SnapshotError::InvalidData(format!(
                        "unknown cartridge image {value}"
                    )));
                }
            },
            2 => {
                let len = reader.u8()? as usize;
                if len > CART_ID_MAX {
                    return Err(SnapshotError::InvalidData(
                        "cartridge name is too long".to_string(),
                    ));
                }
                let bytes = reader.raw_bytes(len)?;
                let stem = std::str::from_utf8(bytes).map_err(|_| {
                    SnapshotError::InvalidData("cartridge name is not utf-8".to_string())
                })?;
                if stem.is_empty() {
                    None
                } else {
                    Some(CartId::new(stem).ok_or_else(|| {
                        SnapshotError::InvalidData(format!("invalid cartridge name {stem}"))
                    })?)
                }
            }
            value => {
                return Err(SnapshotError::InvalidData(format!(
                    "unknown machine config version {value}"
                )));
            }
        };
        let mut slots = [None; 4];
        let mut hbf_count = 0;
        for slot in &mut slots {
            match reader.u8()? {
                0 => {}
                1 => {
                    hbf_count += 1;
                    *slot = Some(ExpansionCard::VtDos12);
                }
                value => {
                    return Err(SnapshotError::InvalidData(format!(
                        "unknown expansion card {value}"
                    )));
                }
            }
        }
        if hbf_count > 1 {
            return Err(SnapshotError::InvalidData(
                "only one HBF card is supported".to_string(),
            ));
        }
        Ok(Self {
            is_plus,
            rom_version,
            cart,
            slots,
        })
    }

    #[cfg_attr(not(any(feature = "native", feature = "wasm-full")), allow(dead_code))]
    pub(crate) fn write_snapshot_chunk(&self) -> Vec<u8> {
        let mut writer = Writer::new();
        writer.u8(self.is_plus as u8);
        writer.u8(self.rom_version_id());
        writer.u8(self.slots[0].is_some() as u8);
        writer.u8(2);
        if let Some(cart) = self.cart {
            let stem = cart.as_str();
            writer.u8(stem.len() as u8);
            writer.raw_bytes(stem.as_bytes());
        } else {
            writer.u8(0);
        }
        for slot in self.slots {
            writer.u8(match slot {
                None => 0,
                Some(ExpansionCard::VtDos12) => 1,
            });
        }
        writer.into_inner()
    }

    pub(crate) fn rom_uses(&self) -> Vec<(String, RomUse)> {
        let mut uses = match self.rom_version {
            RomVersion::V1_2 => vec![
                ("TVC12_D3.64K".to_string(), RomUse::System),
                ("TVC12_D4.64K".to_string(), RomUse::System),
                ("TVC12_D7.64K".to_string(), RomUse::System),
            ],
            RomVersion::V2_2 => vec![
                ("TVC22_D4.64K".to_string(), RomUse::System),
                ("TVC22_D6.64K".to_string(), RomUse::System),
                ("TVC22_D7.64K".to_string(), RomUse::System),
            ],
        };
        if let Some(cart) = self.cart {
            uses.push((cart.file_name(), RomUse::Cart));
        }
        for (slot, card) in self.slots.iter().enumerate() {
            if let Some(ExpansionCard::VtDos12) = card {
                uses.push(("VT-DOS12-DISK.ROM".to_string(), RomUse::Hbf(slot)));
            }
        }
        uses
    }
}

pub(crate) fn place_machine_rom(tvc: &mut Tvc, name: &str, data: &[u8], use_: RomUse) {
    match use_ {
        RomUse::System => tvc.add_rom(name, data),
        RomUse::Cart => tvc.load_cart_rom(data),
        RomUse::Hbf(slot) => tvc.attach_hbf(slot, data),
    }
}
