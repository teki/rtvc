use crate::hbf::HBF;
use crate::host_file::HostFileCard;
use crate::machine_type::ExpansionCard;
use crate::snapshot::{Reader, Result, Writer};

/// Connector-level interface of one expansion card.
pub(crate) trait SlotDevice {
    /// Two-bit identifier reported in port `0x5A`.
    fn id_bits(&self) -> u8;
    /// IOMEM window access at `0xC000-0xDFFF`, only while this slot is selected.
    fn mem_read(&mut self, offset: u16) -> u8 {
        self.mem_peek(offset)
    }
    fn mem_peek(&self, offset: u16) -> u8;
    fn mem_write(&mut self, offset: u16, val: u8);
    /// Slot I/O with the full 16-bit port address; cards decode the low nibble.
    fn io_read(&mut self, port: u16) -> u8;
    fn io_write(&mut self, port: u16, val: u8);
    fn irq_active(&self) -> bool {
        false
    }
    fn reset(&mut self) {}
    fn write_snapshot(&self, w: &mut Writer);
    fn read_snapshot(&mut self, r: &mut Reader<'_>) -> Result<()>;
}

pub(crate) enum Card {
    Hbf(HBF),
    HostFile(HostFileCard),
}

macro_rules! with_device {
    ($card:expr, $device:ident => $body:expr) => {
        match $card {
            Card::Hbf($device) => $body,
            Card::HostFile($device) => $body,
        }
    };
}

impl Card {
    pub(crate) fn kind(&self) -> ExpansionCard {
        match self {
            Card::Hbf(_) => ExpansionCard::VtDos12,
            Card::HostFile(_) => ExpansionCard::HostFile,
        }
    }

    pub(crate) fn hbf(&self) -> Option<&HBF> {
        match self {
            Card::Hbf(hbf) => Some(hbf),
            _ => None,
        }
    }

    pub(crate) fn hbf_mut(&mut self) -> Option<&mut HBF> {
        match self {
            Card::Hbf(hbf) => Some(hbf),
            _ => None,
        }
    }

    pub(crate) fn host_file_mut(&mut self) -> Option<&mut HostFileCard> {
        match self {
            Card::HostFile(card) => Some(card),
            _ => None,
        }
    }
}

impl SlotDevice for Card {
    fn id_bits(&self) -> u8 {
        with_device!(self, device => device.id_bits())
    }

    fn mem_read(&mut self, offset: u16) -> u8 {
        with_device!(self, device => device.mem_read(offset))
    }

    fn mem_peek(&self, offset: u16) -> u8 {
        with_device!(self, device => device.mem_peek(offset))
    }

    fn mem_write(&mut self, offset: u16, val: u8) {
        with_device!(self, device => device.mem_write(offset, val))
    }

    fn io_read(&mut self, port: u16) -> u8 {
        with_device!(self, device => device.io_read(port))
    }

    fn io_write(&mut self, port: u16, val: u8) {
        with_device!(self, device => device.io_write(port, val))
    }

    fn irq_active(&self) -> bool {
        with_device!(self, device => device.irq_active())
    }

    fn reset(&mut self) {
        with_device!(self, device => device.reset())
    }

    fn write_snapshot(&self, w: &mut Writer) {
        with_device!(self, device => device.write_snapshot(w))
    }

    fn read_snapshot(&mut self, r: &mut Reader<'_>) -> Result<()> {
        with_device!(self, device => device.read_snapshot(r))
    }
}

pub(crate) struct ExpansionSlots {
    slots: [Option<Card>; 4],
    // Two-bit extension type identifiers exposed by the 0x5A/0x5E status port.
    type_status: u8,
    // Selected extension memory mapping from port 0x03 bits 6-7.
    selected_mapping: u8,
    // Per-slot interrupt enables written through bit 7 of ports 0x58-0x5B.
    irq_enable: u8,
}

impl ExpansionSlots {
    pub(crate) fn new() -> Self {
        Self {
            slots: [None, None, None, None],
            type_status: 0xFF,
            selected_mapping: 0,
            irq_enable: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.selected_mapping = 0;
        self.irq_enable = 0;
        for card in self.slots.iter_mut().flatten() {
            card.reset();
        }
        self.recompute_type_status();
    }

    pub(crate) fn attach(&mut self, slot: usize, card: Card) {
        if slot >= self.slots.len() {
            return;
        }
        self.slots[slot] = Some(card);
        self.recompute_type_status();
    }

    fn recompute_type_status(&mut self) {
        self.type_status = 0xFF;
        for (slot, card) in self.slots.iter().enumerate() {
            let Some(card) = card else {
                continue;
            };
            self.type_status &= !(3 << (slot * 2));
            self.type_status |= (card.id_bits() & 3) << (slot * 2);
        }
    }

    pub(crate) fn selected_mapping(&self) -> u8 {
        self.selected_mapping
    }

    pub(crate) fn set_selected_mapping(&mut self, mapping: u8) {
        self.selected_mapping = mapping & 0x03;
    }

    pub(crate) fn type_status(&self) -> u8 {
        self.type_status
    }

    pub(crate) fn set_type_status(&mut self, type_status: u8) {
        self.type_status = type_status;
    }

    pub(crate) fn irq_enable(&self) -> u8 {
        self.irq_enable
    }

    pub(crate) fn set_irq_enable(&mut self, irq_enable: u8) {
        self.irq_enable = irq_enable & 0x0F;
    }

    /// Port `0x58`-`0x5B` write: bit 7 enables (1) or disables and clears (0)
    /// the request line of slot `port - 0x58`.
    pub(crate) fn write_irq_control(&mut self, slot: usize, val: u8) {
        let mask = 1 << slot;
        if val & 0x80 != 0 {
            self.irq_enable |= mask;
        } else {
            self.irq_enable &= !mask;
        }
    }

    /// Active-low slot request bits for port `0x59` bits 0-3.
    pub(crate) fn irq_status(&self) -> u8 {
        let mut bits = 0x0F;
        for (slot, card) in self.slots.iter().enumerate() {
            if card.as_ref().is_some_and(SlotDevice::irq_active)
                && self.irq_enable & (1 << slot) != 0
            {
                bits &= !(1 << slot);
            }
        }
        bits
    }

    pub(crate) fn irq_requested(&self) -> bool {
        self.irq_status() != 0x0F
    }

    pub(crate) fn read_mem(&mut self, offset: u16) -> u8 {
        self.slots[self.selected_mapping as usize]
            .as_mut()
            .map(|card| card.mem_read(offset))
            .unwrap_or(0xFF)
    }

    pub(crate) fn write_mem(&mut self, offset: u16, val: u8) {
        if let Some(card) = self.slots[self.selected_mapping as usize].as_mut() {
            card.mem_write(offset, val);
        }
    }

    pub(crate) fn port_slot(port: u8) -> Option<usize> {
        match port {
            0x10..=0x4F => Some((port >> 4) as usize - 1),
            _ => None,
        }
    }

    pub(crate) fn read_port(&mut self, port: u16) -> Option<u8> {
        let slot = Self::port_slot(port as u8)?;
        Some(
            self.slots[slot]
                .as_mut()
                .map(|card| card.io_read(port))
                .unwrap_or(0xFF),
        )
    }

    pub(crate) fn write_port(&mut self, port: u16, val: u8) -> bool {
        let Some(slot) = Self::port_slot(port as u8) else {
            return false;
        };
        if let Some(card) = self.slots[slot].as_mut() {
            card.io_write(port, val);
        }
        true
    }

    pub(crate) fn card(&self, slot: usize) -> Option<&Card> {
        self.slots.get(slot).and_then(Option::as_ref)
    }

    pub(crate) fn card_mut(&mut self, slot: usize) -> Option<&mut Card> {
        self.slots.get_mut(slot).and_then(Option::as_mut)
    }

    pub(crate) fn kind(&self, slot: usize) -> Option<ExpansionCard> {
        self.card(slot).map(Card::kind)
    }

    pub(crate) fn hbf_at(&self, slot: usize) -> Option<&HBF> {
        self.card(slot).and_then(Card::hbf)
    }

    pub(crate) fn hbf_at_mut(&mut self, slot: usize) -> Option<&mut HBF> {
        self.card_mut(slot).and_then(Card::hbf_mut)
    }

    /// Slots holding an HBF card, in slot order.
    pub(crate) fn hbf_slots(&self) -> Vec<usize> {
        (0..self.slots.len())
            .filter(|slot| self.hbf_at(*slot).is_some())
            .collect()
    }

    pub(crate) fn host_file_cards_mut(&mut self) -> impl Iterator<Item = &mut HostFileCard> {
        self.slots
            .iter_mut()
            .flatten()
            .filter_map(Card::host_file_mut)
    }

    /// First HBF card in slot order, for legacy `HBF ` snapshot chunks.
    pub(crate) fn hbf_mut(&mut self) -> Option<&mut HBF> {
        self.slots.iter_mut().flatten().find_map(Card::hbf_mut)
    }
}
