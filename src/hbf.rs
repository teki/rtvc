#![allow(dead_code)]

use crate::fd1793::FD1793;

struct MemBlock {
    name: String,
    is_ram: bool,
    data: Vec<u8>,
}

impl MemBlock {
    fn new(name: &str, is_ram: bool, size: usize) -> Self {
        MemBlock {
            name: name.to_string(),
            is_ram,
            data: vec![0; size],
        }
    }

    fn from_slice(name: &str, is_ram: bool, buffer: &[u8], offset: usize, size: usize) -> Self {
        let mut data = vec![0; size];
        let len = size.min(buffer.len().saturating_sub(offset));
        data[..len].copy_from_slice(&buffer[offset..offset + len]);
        MemBlock {
            name: name.to_string(),
            is_ram,
            data,
        }
    }

    fn write_snapshot(&self, w: &mut crate::snapshot::Writer) {
        w.string(&self.name);
        w.u8(self.is_ram as u8);
        w.bytes(&self.data);
    }

    fn read_snapshot(r: &mut crate::snapshot::Reader<'_>) -> crate::snapshot::Result<Self> {
        Ok(MemBlock {
            name: r.string()?,
            is_ram: r.u8()? != 0,
            data: r.bytes()?.to_vec(),
        })
    }
}

pub struct HBF {
    rom0: MemBlock,
    rom1: MemBlock,
    rom2: MemBlock,
    rom3: MemBlock,
    rom: usize,
    ram: MemBlock,
    fdc: FD1793,
}

impl HBF {
    pub fn new(rom: &[u8]) -> Self {
        HBF {
            rom0: MemBlock::from_slice("ROM0", false, rom, 0x0000, 0x1000),
            rom1: MemBlock::from_slice("ROM1", false, rom, 0x1000, 0x1000),
            rom2: MemBlock::from_slice("ROM2", false, rom, 0x2000, 0x1000),
            rom3: MemBlock::from_slice("ROM3", false, rom, 0x3000, 0x1000),
            rom: 0,
            ram: MemBlock::new("RAM", true, 4096),
            fdc: FD1793::new(),
        }
    }

    pub fn get_type(&self) -> u8 {
        2
    }

    pub fn read_port(&mut self, addr: u8) -> u8 {
        if addr <= 4 { self.fdc.read(addr) } else { 0 }
    }

    pub fn write_port(&mut self, addr: u8, val: u8) {
        if addr <= 4 {
            self.fdc.write(addr, val);
        } else if addr == 8 {
            self.rom = match val & 0x30 {
                0x00 => 0,
                0x10 => 1,
                0x20 => 2,
                0x30 => 3,
                _ => self.rom,
            };
        }
    }

    pub fn r8(&mut self, addr: u16) -> u8 {
        if addr >= 0x1000 {
            self.ram.data[addr as usize - 0x1000]
        } else {
            let rom = match self.rom {
                0 => &self.rom0,
                1 => &self.rom1,
                2 => &self.rom2,
                _ => &self.rom3,
            };
            rom.data[addr as usize]
        }
    }

    pub fn w8(&mut self, addr: u16, val: u8) {
        if addr >= 0x1000 {
            self.ram.data[addr as usize - 0x1000] = val;
        }
    }

    pub fn load_disk(&mut self, name: &str, data: &[u8]) {
        self.fdc.load_dsk(0, name, data);
    }

    pub fn get_fdc_mut(&mut self) -> &mut FD1793 {
        &mut self.fdc
    }

    pub fn write_snapshot(&self, w: &mut crate::snapshot::Writer) {
        self.rom0.write_snapshot(w);
        self.rom1.write_snapshot(w);
        self.rom2.write_snapshot(w);
        self.rom3.write_snapshot(w);
        w.usize(self.rom);
        self.ram.write_snapshot(w);
        self.fdc.write_snapshot(w);
    }

    pub fn read_snapshot(r: &mut crate::snapshot::Reader<'_>) -> crate::snapshot::Result<Self> {
        let rom0 = MemBlock::read_snapshot(r)?;
        let rom1 = MemBlock::read_snapshot(r)?;
        let rom2 = MemBlock::read_snapshot(r)?;
        let rom3 = MemBlock::read_snapshot(r)?;
        let rom = r.usize()?.min(3);
        let ram = MemBlock::read_snapshot(r)?;
        let mut fdc = FD1793::new();
        fdc.read_snapshot(r)?;
        Ok(HBF {
            rom0,
            rom1,
            rom2,
            rom3,
            rom,
            ram,
            fdc,
        })
    }
}
