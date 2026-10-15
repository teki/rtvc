//! 6845 CRTC counter and register model shared by the TVC and the CPC.
//!
//! The CRTC is advanced one character clock per [`Crtc::tick`] and reports the
//! memory address, raster address, display enable, raw sync levels, and cursor
//! for that character. Machine-specific address mapping, pixel decoding, and
//! sync shaping stay with the caller.

/// Chip variant, for the behaviors that differ between the fitted CRTCs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrtcVariant {
    /// Motorola MC6845 (TVC): VSYNC is always 16 lines, an HSYNC width of 0
    /// means 16, and write-only registers read as `0xFF`.
    Mc6845,
    /// Hitachi HD6845S, CPC "type 0": the R3 high nibble sets the VSYNC
    /// width (0 means 16), an HSYNC width of 0 produces no HSYNC, and
    /// write-only registers read as `0x00`.
    Hd6845s,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CrtcTick {
    pub ma: u16,
    pub ra: u8,
    pub display_enable: bool,
    pub raw_hsync: bool,
    pub raw_vsync: bool,
    pub line_start: bool,
    pub cursor: bool,
}

pub struct Crtc {
    variant: CrtcVariant,
    reg_idx: u8,
    reg: [u8; 18],
    configured: bool,
    h_count: u16,
    row: u16,
    raster: u16,
    adjust_line: u16,
    in_adjust: bool,
    row_address: u16,
    hsync_remaining: u8,
    vsync_lines_remaining: u8,
}

impl Crtc {
    pub fn new(variant: CrtcVariant) -> Self {
        Self {
            variant,
            reg_idx: 0,
            reg: [0; 18],
            configured: false,
            h_count: 0,
            row: 0,
            raster: 0,
            adjust_line: 0,
            in_adjust: false,
            row_address: 0,
            hsync_remaining: 0,
            vsync_lines_remaining: 0,
        }
    }

    pub fn variant(&self) -> CrtcVariant {
        self.variant
    }

    /// Clears the registers and counters, as after power-on.
    pub fn reset(&mut self) {
        self.reg_idx = 0;
        self.reg = [0; 18];
        self.configured = false;
        self.reset_transient();
    }

    /// Restores registers from a snapshot and restarts the counters.
    pub fn load_registers(&mut self, reg_idx: u8, reg: &[u8]) {
        self.reg_idx = reg_idx & 0x1f;
        self.reg.copy_from_slice(reg);
        self.configured = true;
        self.reset_transient();
    }

    /// Writes registers and counters, so a restored CRTC continues from the
    /// same raster position. (The TVC snapshot stores registers only.)
    pub fn write_state(&self, writer: &mut crate::snapshot::Writer) {
        writer.u8(self.reg_idx);
        writer.raw_bytes(&self.reg);
        writer.u8(self.configured as u8);
        for value in [
            self.h_count,
            self.row,
            self.raster,
            self.adjust_line,
            self.row_address,
        ] {
            writer.u16(value);
        }
        writer.u8(self.in_adjust as u8);
        writer.u8(self.hsync_remaining);
        writer.u8(self.vsync_lines_remaining);
    }

    pub fn read_state(
        &mut self,
        reader: &mut crate::snapshot::Reader<'_>,
    ) -> crate::snapshot::Result<()> {
        self.reg_idx = reader.u8()? & 0x1f;
        self.reg.copy_from_slice(reader.raw_bytes(18)?);
        self.configured = reader.u8()? != 0;
        self.h_count = reader.u16()? & 0x00ff;
        self.row = reader.u16()? & 0x007f;
        self.raster = reader.u16()? & 0x001f;
        self.adjust_line = reader.u16()?;
        self.row_address = reader.u16()? & 0x3fff;
        self.in_adjust = reader.u8()? != 0;
        self.hsync_remaining = reader.u8()?;
        self.vsync_lines_remaining = reader.u8()?;
        Ok(())
    }

    pub fn reset_transient(&mut self) {
        self.h_count = 0;
        self.row = 0;
        self.raster = 0;
        self.adjust_line = 0;
        self.in_adjust = false;
        self.row_address = self.display_start();
        self.hsync_remaining = 0;
        self.vsync_lines_remaining = 0;
    }

    pub fn select(&mut self, index: u8) {
        self.reg_idx = index & 0x1f;
    }

    pub fn selected(&self) -> u8 {
        self.reg_idx
    }

    pub fn registers(&self) -> &[u8; 18] {
        &self.reg
    }

    pub fn reg(&self, index: usize) -> u8 {
        self.reg.get(index).copied().unwrap_or(0)
    }

    pub fn write(&mut self, value: u8) {
        let index = self.reg_idx as usize;
        if index < 16 {
            self.reg[index] = value;
            self.configured = true;
            // Configuration performed before the first tick must use the new
            // start address immediately. A live mid-frame R12/R13 write is
            // instead picked up at the next CRTC frame restart.
            if matches!(index, 12 | 13) && self.h_count == 0 && self.row == 0 && self.raster == 0 {
                self.row_address = self.display_start();
            }
        }
    }

    pub fn read(&self) -> u8 {
        match self.reg_idx {
            12 | 14 | 16 => self.reg[self.reg_idx as usize] & 0x3f,
            13 | 15 | 17 => self.reg[self.reg_idx as usize],
            _ => match self.variant {
                CrtcVariant::Mc6845 => 0xff,
                CrtcVariant::Hd6845s => 0x00,
            },
        }
    }

    pub fn configured(&self) -> bool {
        self.configured
    }

    /// Current (row, raster, horizontal count).
    pub fn position(&self) -> (u16, u16, u16) {
        (self.row, self.raster, self.h_count)
    }

    pub fn vsync(&self) -> bool {
        self.vsync_lines_remaining != 0
    }

    pub fn display_start(&self) -> u16 {
        (((self.reg[12] as u16 & 0x3f) << 8) | self.reg[13] as u16) & 0x3fff
    }

    pub fn cursor_address(&self) -> u16 {
        (((self.reg[14] as u16 & 0x3f) << 8) | self.reg[15] as u16) & 0x3fff
    }

    pub fn cursor_enabled(&self) -> bool {
        self.reg[10] & 0x60 != 0x20
    }

    pub fn horizontal_total(&self) -> u16 {
        self.reg[0] as u16
    }

    pub fn horizontal_displayed(&self) -> u16 {
        self.reg[1] as u16
    }

    pub fn line_character_clocks(&self) -> u16 {
        self.horizontal_total() + 1
    }

    pub fn vertical_displayed(&self) -> u16 {
        (self.reg[6] & 0x7f) as u16
    }

    pub fn max_raster(&self) -> u16 {
        (self.reg[9] & 0x1f) as u16
    }

    fn horizontal_sync_width(&self) -> u8 {
        let width = self.reg[3] & 0x0f;
        match self.variant {
            CrtcVariant::Mc6845 if width == 0 => 16,
            _ => width,
        }
    }

    fn vertical_sync_lines(&self) -> u8 {
        match self.variant {
            CrtcVariant::Mc6845 => 16,
            CrtcVariant::Hd6845s => match self.reg[3] >> 4 {
                0 => 16,
                width => width,
            },
        }
    }

    pub fn tick(&mut self) -> CrtcTick {
        let line_start = self.h_count == 0;
        if line_start
            && !self.in_adjust
            && self.row == (self.reg[7] & 0x7f) as u16
            && self.raster == 0
        {
            self.vsync_lines_remaining = self.vertical_sync_lines();
        }
        if self.h_count == self.reg[2] as u16 {
            self.hsync_remaining = self.horizontal_sync_width();
        }

        let ma = self.row_address.wrapping_add(self.h_count) & 0x3fff;
        let display_enable = !self.in_adjust
            && self.row < self.vertical_displayed()
            && self.h_count < self.horizontal_displayed();
        let cursor_start = (self.reg[10] & 0x1f) as u16;
        let cursor_end = (self.reg[11] & 0x1f) as u16;
        let cursor = self.cursor_enabled()
            && display_enable
            && ma == self.cursor_address()
            && self.raster >= cursor_start
            && self.raster <= cursor_end;

        let result = CrtcTick {
            ma,
            ra: self.raster as u8,
            display_enable,
            raw_hsync: self.hsync_remaining != 0,
            raw_vsync: self.vsync_lines_remaining != 0,
            line_start,
            cursor,
        };

        self.advance();
        result
    }

    fn advance(&mut self) {
        if self.hsync_remaining != 0 {
            self.hsync_remaining -= 1;
        }
        if self.h_count != self.horizontal_total() {
            self.h_count = self.h_count.wrapping_add(1) & 0x00ff;
            return;
        }

        self.h_count = 0;
        if self.vsync_lines_remaining != 0 {
            self.vsync_lines_remaining -= 1;
        }

        if self.in_adjust {
            self.adjust_line += 1;
            if self.adjust_line >= (self.reg[5] & 0x1f) as u16 {
                self.start_frame();
            }
            return;
        }

        if self.raster != self.max_raster() {
            self.raster = self.raster.wrapping_add(1) & 0x001f;
        } else {
            self.raster = 0;
            if self.row != (self.reg[4] & 0x7f) as u16 {
                self.row = self.row.wrapping_add(1) & 0x007f;
                self.row_address =
                    self.row_address.wrapping_add(self.horizontal_displayed()) & 0x3fff;
            } else if self.reg[5] & 0x1f != 0 {
                self.in_adjust = true;
                self.adjust_line = 0;
            } else {
                self.start_frame();
            }
        }
    }

    fn start_frame(&mut self) {
        self.row = 0;
        self.raster = 0;
        self.adjust_line = 0;
        self.in_adjust = false;
        self.row_address = self.display_start();
    }
}

#[cfg(test)]
#[path = "crtc_tests.rs"]
mod tests;
