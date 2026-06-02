#![allow(dead_code)]

const HSYNC: i16 = 0x0400;
const VSYNC: i16 = 0x0800;

const STREAM_SIZE: usize = 608 * 288 * 2 * 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VidModel {
    FastFrame,
    Interleaved,
}

fn to_rgba(val: u8) -> u32 {
    let intens: u32 = 0x7F | ((val as u32 & 0x40) << 1);
    let g = (0x100u32 - ((val as u32 >> 4) & 1)) & intens;
    let r = (0x100u32 - ((val as u32 >> 2) & 1)) & intens;
    let b = (0x100u32 - (val as u32 & 1)) & intens;
    0xFF000000 | (b << 16) | (g << 8) | r
}

fn gen_address(ma: u16, rl: u8) -> u16 {
    let ma = ma & 0xFFF;
    ((rl as u16 & 0x03) << 6) | (ma & 0x3F) | ((ma & 0x3FC0) << 2)
}

pub struct Color {
    pub color: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub rgba: u32,
}

impl Color {
    pub fn new() -> Self {
        Color {
            color: 0,
            r: 0,
            g: 0,
            b: 0,
            rgba: 0xFF,
        }
    }

    pub fn set_color(&mut self, val: u8) {
        self.color = val;
        let intens: u32 = 0x7F | ((val as u32 & 0x40) << 1);
        self.r = ((0x100u32 - ((val as u32 >> 2) & 1)) & intens) as u8;
        self.g = ((0x100u32 - ((val as u32 >> 4) & 1)) & intens) as u8;
        self.b = ((0x100u32 - (val as u32 & 1)) & intens) as u8;
        self.rgba = to_rgba(val);
    }
}

pub struct Vid {
    // Video mode (bits 0-1 of port 0x06)
    mode: u8,
    // CPU clock info
    clock_ch: u32,
    // CRTC register index and registers
    reg_idx: u8,
    reg: [u8; 18],
    // Decoded CRTC register values
    ht: u8,
    hd: u8,
    hsp: u8,
    hsw: u8,
    vsw: u8,
    vt: u8,
    adj: u8,
    vd: u8,
    vsp: u8,
    im: u8,
    skec: u8,
    slr: u8,
    curend: u8,
    curstart: u8,
    curenabled: bool,
    smem: u16,
    curaddr: u16,
    curmemaddr: u16,
    // Palette
    palette: [Color; 4],
    // Border
    border: u8,
    border2: u8,
    // Streaming state
    mem_start: u16,
    mem: u16,
    addr: u16,
    vlines: i32,
    alines: u8,
    row: i32,
    line: u8,
    char_x: u8,
    run_for: u32,
    // Stream ring buffer
    stream: Vec<i16>,
    stream_head: usize,
    stream_tail: usize,
    // Renderer state
    render_phase: u8,
    render_phase_next: u8,
    render_hcnt: u8,
    render_vcnt: u16,
    render_y: u16,
    render_a: usize,
}

impl Vid {
    pub fn new() -> Self {
        Vid {
            mode: 0,
            clock_ch: 2,
            reg_idx: 0,
            reg: [0; 18],
            ht: 0,
            hd: 0,
            hsp: 0,
            hsw: 0,
            vsw: 0,
            vt: 0,
            adj: 0,
            vd: 0,
            vsp: 0,
            im: 0,
            skec: 0,
            slr: 0,
            curend: 0,
            curstart: 0,
            curenabled: false,
            smem: 0,
            curaddr: 0,
            curmemaddr: 0,
            palette: [Color::new(), Color::new(), Color::new(), Color::new()],
            border: 0,
            border2: 0,
            mem_start: 0,
            mem: 0,
            addr: 0,
            vlines: -1,
            alines: 0,
            row: -1,
            line: 0,
            char_x: 0,
            run_for: 0,
            stream: vec![0; STREAM_SIZE],
            stream_head: 0,
            stream_tail: 0,
            render_phase: 0,
            render_phase_next: 0,
            render_hcnt: 0,
            render_vcnt: 0,
            render_y: 0,
            render_a: 0,
        }
    }

    pub fn reset(&mut self) {
        self.reg = [0; 18];
        self.reg_idx = 0;
        self.mode = 0;
        self.border = 0;
        self.border2 = 0;
        self.palette = [Color::new(), Color::new(), Color::new(), Color::new()];
        self.run_for = 0;
        self.stream_head = 0;
        self.stream_tail = 0;
        self.render_phase = 0;
        self.render_phase_next = 0;
        self.render_hcnt = 0;
        self.render_vcnt = 0;
        self.render_y = 0;
        self.render_a = 0;
        self.row = -1;
        self.line = 0;
        self.char_x = 0;
        self.vlines = -1;
        self.alines = 0;
        self.reconfig();
    }

    fn reconfig(&mut self) {
        self.ht = self.reg[0];
        self.hd = self.reg[1];
        self.hsp = self.reg[2];
        self.hsw = self.reg[3] & 0x0F;
        self.vsw = (self.reg[3] >> 4) & 0x0F;
        self.vt = self.reg[4] & 0x7F;
        self.adj = self.reg[5] & 0x1F;
        self.vd = self.reg[6] & 0x7F;
        self.vsp = self.reg[7] & 0x7F;
        self.im = self.reg[8] & 0x03;
        self.skec = (self.reg[8] >> 6) & 0x03;
        self.slr = self.reg[9] & 0x1F;
        self.curenabled = (self.reg[10] & 0x60) != 0x20;
        self.curstart = self.reg[10] & 0x1F;
        self.curend = self.reg[11] & 0x1F;
        self.smem = (self.reg[12] as u16) << 8 | self.reg[13] as u16;
        self.curaddr = ((self.reg[14] as u16 & 0x3F) << 8) | self.reg[15] as u16;
        self.curmemaddr = gen_address(self.curaddr, self.curstart);
    }

    /// Set the CRTC register address (even CRTC ports in 0x70-0x7F)
    pub fn set_reg_idx(&mut self, idx: u8) {
        self.reg_idx = idx & 0x1F;
    }

    /// Get the current CRTC register address.
    pub fn get_reg_idx(&self) -> u8 {
        self.reg_idx
    }

    /// Write to the selected CRTC data register.
    pub fn set_reg(&mut self, val: u8) {
        let idx = self.reg_idx as usize;
        if idx >= 16 {
            return;
        }
        if self.reg[idx] != val {
            self.reg[idx] = val;
            self.reconfig();
        }
    }

    /// Read the selected CRTC data register using TVC/6845-compatible CPU-visible semantics.
    pub fn get_reg(&self) -> u8 {
        match self.reg_idx {
            12 | 14 | 16 => self.reg[self.reg_idx as usize] & 0x3F,
            13 | 15 | 17 => self.reg[self.reg_idx as usize],
            _ => 0xFF,
        }
    }

    /// Write a CPU I/O access to one of the mirrored CRTC ports (0x70-0x7F).
    pub fn write_crtc_port(&mut self, port: u8, val: u8) {
        if port & 1 == 0 {
            self.set_reg_idx(val);
        } else {
            self.set_reg(val);
        }
    }

    /// Read a CPU I/O access from one of the mirrored CRTC ports (0x70-0x7F).
    pub fn read_crtc_port(&self, port: u8) -> u8 {
        if port & 1 == 0 {
            0xFF
        } else {
            self.get_reg()
        }
    }

    /// Set palette entry (port 0x60-0x63)
    pub fn set_palette(&mut self, idx: u8, color: u8) {
        if idx < 4 {
            self.palette[idx as usize].set_color(color);
        }
    }

    /// Get palette entry
    pub fn get_palette(&self, idx: u8) -> u8 {
        if idx < 4 {
            self.palette[idx as usize].color
        } else {
            0
        }
    }

    /// Set border color (port 0x00)
    pub fn set_border(&mut self, color: u8) {
        self.border = color;
        self.border2 = ((color & 0xAA) >> 1) | (color & 0xAA);
    }

    /// Returns true when the CRTC has been configured (hd < ht).
    pub fn is_initialized(&self) -> bool {
        self.hd < self.ht
    }

    pub fn cursor_enabled(&self) -> bool {
        self.curenabled
    }

    /// Set video mode (port 0x06 bits 0-1)
    pub fn set_mode(&mut self, mode: u8) {
        self.mode = mode & 0x03;
    }

    pub fn write_snapshot(&self, w: &mut crate::snapshot::Writer) {
        w.u8(self.mode);
        w.u8(self.reg_idx);
        w.raw_bytes(&self.reg);
        for color in &self.palette {
            w.u8(color.color);
        }
        w.u8(self.border);
    }

    pub fn read_snapshot(
        &mut self,
        r: &mut crate::snapshot::Reader<'_>,
    ) -> crate::snapshot::Result<()> {
        self.reset();
        self.mode = r.u8()? & 0x03;
        self.reg_idx = r.u8()? & 0x1F;
        self.reg.copy_from_slice(r.raw_bytes(18)?);
        self.reconfig();
        for idx in 0..4 {
            let color = r.u8()?;
            self.set_palette(idx, color);
        }
        let border = r.u8()?;
        self.set_border(border);
        Ok(())
    }

    fn stream_init_screen(&mut self) {
        self.mem_start = self.smem;
        self.vlines = -1;
        self.alines = 0;
        self.row = 0;
        self.char_x = 0;
        self.line = 0;
        self.mem = self.mem_start + (self.row as u16) * (self.hd as u16);
        self.addr = gen_address(self.mem, self.line);
    }

    /// Stream characters for `run_for` CPU ticks. Returns true if cursor interrupt fired.
    pub fn stream_some(&mut self, vidmem: &[u8], run_for: u32) -> bool {
        if self.hd >= self.ht {
            return false;
        }

        let mode_val = (self.mode as i16) << 8;
        let mode16 = 2i16 << 8;
        let mut cursor_it = false;

        if self.row == -1 {
            self.stream_init_screen();
        }

        self.run_for += run_for;

        while !cursor_it && self.run_for >= self.clock_ch {
            if self.row < self.vd as i32 {
                if self.char_x < self.hd {
                    if self.curenabled {
                        cursor_it = self.curenabled
                            && self.mem == self.curaddr
                            && self.line == self.curstart;
                    }
                    let addr = self.addr as usize;
                    self.stream_data(mode_val | (vidmem.get(addr).copied().unwrap_or(0) as i16));
                    self.char_x += 1;
                    self.addr += 1;
                    self.mem += 1;
                } else if self.char_x <= self.ht {
                    let hsync = if self.char_x > self.hsp && self.char_x < self.hsp + self.hsw {
                        HSYNC
                    } else {
                        0
                    };
                    self.stream_data(hsync | mode16 | self.border2 as i16);
                    self.char_x += 1;
                } else {
                    // should not happen
                    self.char_x = 0;
                }
            } else if self.row <= self.vt as i32 {
                let vsync;
                if self.vlines >= 0 {
                    vsync = if (self.vlines as u8) < self.vsw {
                        VSYNC
                    } else {
                        0
                    };
                } else if self.row > self.vsp as i32 {
                    vsync = VSYNC;
                    self.vlines = 0;
                } else {
                    vsync = 0;
                }

                if self.char_x <= self.ht {
                    let hsync = if self.char_x > self.hsp && self.char_x < self.hsp + self.hsw {
                        HSYNC
                    } else {
                        0
                    };
                    self.stream_data(vsync | hsync | mode16 | self.border2 as i16);
                    self.char_x += 1;
                }

                if vsync != 0 && self.char_x > self.ht {
                    self.vlines += 1;
                }
            } else if self.adj > 0 && self.alines < self.adj {
                if self.char_x <= self.ht {
                    let hsync = if self.char_x > self.hsp && self.char_x < self.hsp + self.hsw {
                        HSYNC
                    } else {
                        0
                    };
                    self.stream_data(0i16 | hsync | mode16 | self.border2 as i16);
                    self.char_x += 1;
                }

                if self.char_x > self.ht {
                    self.alines += 1;
                }
            } else {
                self.run_for += self.clock_ch;
                self.stream_init_screen();
            }

            if self.char_x > self.ht {
                self.char_x = 0;
                self.line += 1;
                if self.line > self.slr {
                    self.line = 0;
                    self.row += 1;
                }
                self.mem = (self.mem_start + (self.row as u16) * (self.hd as u16)) & 0x3FFF;
                self.addr = gen_address(self.mem, self.line);
            }

            self.run_for -= self.clock_ch;
        }

        cursor_it
    }

    fn stream_data(&mut self, data: i16) {
        let next = (self.stream_head + 1) % STREAM_SIZE;
        if next == self.stream_tail {
            panic!("streamData overflow");
        }
        self.stream[self.stream_head] = data;
        self.stream_head = next;
    }

    fn read_data(&mut self) -> Option<i16> {
        if self.stream_head == self.stream_tail {
            None
        } else {
            let res = self.stream[self.stream_tail];
            self.stream_tail = (self.stream_tail + 1) % STREAM_SIZE;
            Some(res)
        }
    }

    /// Render streamed data into the framebuffer.
    /// Returns true when a full frame has been rendered.
    pub fn render_stream(&mut self, framebuffer: &mut [u32], fb_width: usize) -> bool {
        let mut have_a_frame = false;

        while !have_a_frame {
            match self.read_data() {
                None => break,
                Some(data) => match self.render_phase {
                    0 => {
                        if data & VSYNC != 0 {
                            self.render_phase = 1;
                            self.render_vcnt = 0;
                        }
                    }
                    1 => {
                        if data & HSYNC != 0 {
                            self.render_vcnt += 1;
                            if self.render_vcnt == 26 {
                                self.render_phase = 100;
                                self.render_phase_next = 2;
                                self.render_hcnt = 1;
                                self.render_y = 0;
                                self.render_a = 0;
                            } else {
                                self.render_phase = 100;
                                self.render_phase_next = 1;
                            }
                        }
                    }
                    100 => {
                        if data & HSYNC != 0 {
                            self.render_hcnt += 1;
                        } else {
                            self.render_phase = self.render_phase_next;
                        }
                    }
                    2 => {
                        self.render_hcnt += 1;
                        if self.render_hcnt == 16 {
                            self.render_phase = 3;
                            self.render_hcnt = 0;
                        }
                    }
                    3 => {
                        self.render_hcnt += 1;
                        self.render_a = self.write_pixel(framebuffer, self.render_a, data);
                        if self.render_hcnt == 76 {
                            self.render_y += 1;
                            self.render_a = fb_width * self.render_y as usize;
                            if self.render_y == 288 {
                                self.render_phase = 0;
                                have_a_frame = true;
                            } else {
                                self.render_phase = 4;
                            }
                        }
                    }
                    4 => {
                        if data & HSYNC != 0 {
                            self.render_phase = 100;
                            self.render_phase_next = 2;
                            self.render_hcnt = 1;
                        }
                    }
                    _ => {}
                },
            }
        }

        have_a_frame
    }

    fn write_pixel(&self, fbd: &mut [u32], mut act_pixel: usize, pixel_data: i16) -> usize {
        let mode = ((pixel_data >> 8) & 3) as u8;
        let pixel_data = (pixel_data & 0xFF) as u8;

        match mode {
            0 => {
                for i in (0..8).rev() {
                    let idx = ((pixel_data >> i) & 1) as usize;
                    fbd[act_pixel] = self.palette[idx].rgba;
                    act_pixel += 1;
                }
            }
            1 => {
                let pixel_data2 = (pixel_data >> 4) as u16;
                let mut pd = pixel_data as u16;
                pd <<= 1;
                let d3 = (pd & 2) | (pixel_data2 & 1);
                pd >>= 1;
                let mut pixel_data2 = pixel_data2 >> 1;
                let d2 = (pd & 2) | (pixel_data2 & 1);
                pd >>= 1;
                pixel_data2 >>= 1;
                let d1 = (pd & 2) | (pixel_data2 & 1);
                pd >>= 1;
                pixel_data2 >>= 1;
                let d0 = (pd & 2) | (pixel_data2 & 1);

                let rgba = self.palette[d0 as usize].rgba;
                fbd[act_pixel] = rgba;
                act_pixel += 1;
                fbd[act_pixel] = rgba;
                act_pixel += 1;

                let rgba = self.palette[d1 as usize].rgba;
                fbd[act_pixel] = rgba;
                act_pixel += 1;
                fbd[act_pixel] = rgba;
                act_pixel += 1;

                let rgba = self.palette[d2 as usize].rgba;
                fbd[act_pixel] = rgba;
                act_pixel += 1;
                fbd[act_pixel] = rgba;
                act_pixel += 1;

                let rgba = self.palette[d3 as usize].rgba;
                fbd[act_pixel] = rgba;
                act_pixel += 1;
                fbd[act_pixel] = rgba;
                act_pixel += 1;
            }
            _ => {
                let rgba = to_rgba(pixel_data >> 1);
                for _ in 0..4 {
                    fbd[act_pixel] = rgba;
                    act_pixel += 1;
                }
                let rgba = to_rgba(pixel_data);
                for _ in 0..4 {
                    fbd[act_pixel] = rgba;
                    act_pixel += 1;
                }
            }
        }

        act_pixel
    }

    /// Simplified once-per-frame drawing (no streaming).
    /// Draws the current display state directly to the framebuffer.
    /// The framebuffer should be 608*288 pixels.
    pub fn draw_frame(&self, vram: &[u8], framebuffer: &mut [u32]) {
        let hd = self.hd as usize;
        let vd = self.vd as usize;
        let slr = self.slr as usize;
        let scanlines_per_row = slr + 1;
        let active_height = vd * scanlines_per_row;
        let top_border = (288 - active_height) / 2;
        let left_border = (76 - hd) / 2;
        let border_rgba = to_rgba(self.border2);

        for y in 0..288 {
            let line_start = y * 608;

            if y < top_border || y >= top_border + active_height {
                for x in 0..608 {
                    framebuffer[line_start + x] = border_rgba;
                }
                continue;
            }

            let row = (y - top_border) / scanlines_per_row;
            let line_offset = (y - top_border) % scanlines_per_row;

            for char_x in 0..76 {
                let mut pixel_x = char_x * 8;
                if char_x < left_border || char_x >= left_border + hd {
                    for p in 0..8 {
                        framebuffer[line_start + pixel_x + p] = border_rgba;
                    }
                    continue;
                }

                let active_char_x = char_x - left_border;
                let ma = (self.smem as usize + row * hd + active_char_x) & 0x3FFF;
                let vram_addr = gen_address(ma as u16, line_offset as u8) as usize;
                let byte = vram.get(vram_addr).copied().unwrap_or(0);

                // Decode pixels based on current mode
                match self.mode {
                    0 => {
                        for i in (0..8).rev() {
                            let idx = ((byte >> i) & 1) as usize;
                            framebuffer[line_start + pixel_x] = self.palette[idx].rgba;
                            pixel_x += 1;
                        }
                        continue;
                    }
                    1 => {
                        let pixel_data2 = (byte >> 4) as u16;
                        let mut pd = byte as u16;
                        pd <<= 1;
                        let d3 = (pd & 2) | (pixel_data2 & 1);
                        pd >>= 1;
                        let mut pixel_data2 = pixel_data2 >> 1;
                        let d2 = (pd & 2) | (pixel_data2 & 1);
                        pd >>= 1;
                        pixel_data2 >>= 1;
                        let d1 = (pd & 2) | (pixel_data2 & 1);
                        pd >>= 1;
                        pixel_data2 >>= 1;
                        let d0 = (pd & 2) | (pixel_data2 & 1);

                        let v = self.palette[d0 as usize].rgba;
                        framebuffer[line_start + pixel_x + 0] = v;
                        framebuffer[line_start + pixel_x + 1] = v;
                        let v = self.palette[d1 as usize].rgba;
                        framebuffer[line_start + pixel_x + 2] = v;
                        framebuffer[line_start + pixel_x + 3] = v;
                        let v = self.palette[d2 as usize].rgba;
                        framebuffer[line_start + pixel_x + 4] = v;
                        framebuffer[line_start + pixel_x + 5] = v;
                        let v = self.palette[d3 as usize].rgba;
                        framebuffer[line_start + pixel_x + 6] = v;
                        framebuffer[line_start + pixel_x + 7] = v;
                    }
                    _ => {
                        let v = to_rgba(byte >> 1);
                        framebuffer[line_start + pixel_x + 0] = v;
                        framebuffer[line_start + pixel_x + 1] = v;
                        framebuffer[line_start + pixel_x + 2] = v;
                        framebuffer[line_start + pixel_x + 3] = v;
                        let v = to_rgba(byte);
                        framebuffer[line_start + pixel_x + 4] = v;
                        framebuffer[line_start + pixel_x + 5] = v;
                        framebuffer[line_start + pixel_x + 6] = v;
                        framebuffer[line_start + pixel_x + 7] = v;
                    }
                }
            }
        }
    }
}
