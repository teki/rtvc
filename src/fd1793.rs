#![allow(dead_code)]

const ST_NOTREADY: u8 = 0x80;
const ST_READONLY: u8 = 0x40;
const ST_WRFAULT: u8 = 0x20;
const ST_HEADLOADED: u8 = 0x20;
const ST_RECTYPE: u8 = 0x20;
const ST_SEEKERR: u8 = 0x10;
const ST_RECNF: u8 = 0x10;
const ST_CRCERR: u8 = 0x08;
const ST_TRACK0: u8 = 0x04;
const ST_LOSTDATA: u8 = 0x04;
const ST_INDEX: u8 = 0x02;
const ST_DRQ: u8 = 0x02;
const ST_BUSY: u8 = 0x01;

const PRT_INTRQ: u8 = 0x01;
const PRT_DRQ: u8 = 0x80;

const CMD_READSEC: u8 = 1;
const CMD_READADDR: u8 = 2;

struct FDisk {
    log: bool,
    name: String,
    dsk: Option<Vec<u8>>,
    sectors_per_track: u16,
    sector_size: u16,
    tot_sec: u16,
    num_heads: u16,
    tracks_per_side: u16,
    data: u8,
    sector: u8,
    track: u8,
    side: u8,
    position: usize,
    read_offset: usize,
    read_length: usize,
    read_source: u8,
    read_buffer: [u8; 6],
}

impl FDisk {
    fn new() -> Self {
        FDisk {
            log: false,
            name: String::new(),
            dsk: None,
            sectors_per_track: 9,
            sector_size: 512,
            tot_sec: 720,
            num_heads: 1,
            tracks_per_side: 80,
            data: 0,
            sector: 0,
            track: 0,
            side: 0,
            position: 0,
            read_offset: 0,
            read_length: 0,
            read_source: 0,
            read_buffer: [0; 6],
        }
    }

    fn is_ready(&self) -> bool {
        self.dsk.is_some()
    }

    fn load_dsk(&mut self, name: &str, dsk: &[u8]) {
        self.name = name.to_string();
        self.dsk = Some(dsk.to_vec());
        self.parse();
    }

    fn seek(&mut self, track: u8, sector: u8, side: u8) {
        if self.dsk.is_some() {
            let offset_sector = if sector != 0 { (sector - 1) as u16 } else { 0 };
            self.position = (track as u16
                * (self.sectors_per_track * self.num_heads)
                + (self.sectors_per_track * side as u16)
                + offset_sector) as usize
                * self.sector_size as usize;
            self.track = track;
            self.side = side;
        }
    }

    fn read_sector(&mut self, sector: u8) {
        self.read_length = self.sector_size as usize;
        self.read_offset = 0;
        self.read_source = 0;
        self.seek(self.track, sector, self.side);
    }

    fn read_address(&mut self) {
        self.read_length = 6;
        self.read_source = 1;
        self.read_offset = 0;
        self.read_buffer[0] = self.track;
        self.read_buffer[1] = self.side;
        self.read_buffer[2] = 1;
        self.read_buffer[3] = self.sector_length_code();
        self.read_buffer[4] = 0;
        self.read_buffer[5] = 0;
    }

    fn sector_length_code(&self) -> u8 {
        match self.sector_size {
            128 => 0,
            256 => 1,
            512 => 2,
            1024 => 3,
            _ => 2,
        }
    }

    fn read(&mut self) -> bool {
        if self.read_offset < self.read_length {
            if self.read_source != 0 {
                self.data = self.read_buffer[self.read_offset];
            } else if let Some(ref dsk) = self.dsk {
                self.data = dsk[self.position + self.read_offset];
            }
            self.read_offset += 1;
            false
        } else {
            true
        }
    }

    fn parse(&mut self) {
        if !self.is_ready() {
            return;
        }
        let dsk = self.dsk.as_ref().unwrap();
        if dsk[0] == 0xEB && dsk[2] != 0x90 {
            // not really MS-DOS compatible, continue anyway
        } else if dsk[0] != 0xE9 {
            return;
        }
        let sector_size = dsk[11] as u16 | ((dsk[12] as u16) << 8);
        let sectors_per_cluster = dsk[13];
        let rsvd_sec_cnt = dsk[14] as u16 | ((dsk[15] as u16) << 8);
        let num_fat = dsk[16];
        let root_ent_cnt = dsk[17] as u16 | ((dsk[18] as u16) << 8);
        let tot_sec = dsk[19] as u16 | ((dsk[20] as u16) << 8);
        let fat_size = dsk[22] as u16 | ((dsk[23] as u16) << 8);
        let sec_per_trk = dsk[24] as u16 | ((dsk[25] as u16) << 8);
        let num_heads = dsk[26] as u16 | ((dsk[27] as u16) << 8);

        let root_dir_sectors = (root_ent_cnt as u32 * 32 + sector_size as u32 - 1) / sector_size as u32;
        let data_sec = tot_sec as u32 - (rsvd_sec_cnt as u32 + (num_fat as u32 * fat_size as u32) + root_dir_sectors);
        let _count_of_clusters = data_sec / sectors_per_cluster as u32;

        self.sector_size = sector_size;
        self.sectors_per_track = sec_per_trk;
        self.tot_sec = tot_sec;
        self.num_heads = num_heads;
        self.tracks_per_side = tot_sec / sec_per_trk / num_heads;
    }
}

impl Default for FDisk {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FD1793 {
    log: bool,
    disks: [FDisk; 4],
    pr: u8,
    side: u8,
    dsk_idx: usize,
    intrq: u8,
    data: u8,
    track: u8,
    sector: u8,
    command: u8,
    commandtr: u8,
    status: u8,
}

impl FD1793 {
    pub fn new() -> Self {
        FD1793 {
            log: false,
            disks: [FDisk::new(), FDisk::new(), FDisk::new(), FDisk::new()],
            pr: 0,
            side: 0,
            dsk_idx: 0,
            intrq: 0,
            data: 0,
            track: 0,
            sector: 0,
            command: 0,
            commandtr: 0,
            status: 0,
        }
    }

    pub fn load_dsk(&mut self, drive: usize, name: &str, dsk: &[u8]) {
        if drive > 3 {
            return;
        }
        self.disks[drive].load_dsk(name, dsk);
    }

    fn exec(&mut self) {
        if self.commandtr == CMD_READSEC || self.commandtr == CMD_READADDR {
            let finished = self.disks[self.dsk_idx].read();
            if finished {
                self.status &= !ST_BUSY;
                self.intrq = PRT_INTRQ;
            } else {
                self.status |= ST_DRQ;
                self.data = self.disks[self.dsk_idx].data;
            }
        }
    }

    pub fn read(&mut self, addr: u8) -> u8 {
        if self.disks[self.dsk_idx].is_ready() {
            self.status &= !ST_NOTREADY;
        } else {
            self.status |= ST_NOTREADY;
        }

        match addr {
            0 => {
                let return_status = self.status;
                self.status &= ST_BUSY | ST_NOTREADY;
                self.intrq = 0;
                return_status
            }
            1 => self.track,
            2 => self.sector,
            3 => {
                if (self.status & ST_DRQ) == 0 {
                    return 0;
                }
                let result = self.data;
                self.status &= !ST_DRQ;
                result
            }
            4 => {
                if (self.status & ST_BUSY) != 0 {
                    self.exec();
                }
                self.intrq | if (self.status & ST_DRQ) != 0 { PRT_DRQ } else { 0 }
            }
            _ => 0,
        }
    }

    fn command(&mut self, val: u8) {
        let cmd = val >> 4;
        self.intrq = 0;
        self.command = val;
        self.commandtr = 0;

        match cmd {
            0x00 => {
                self.intrq = PRT_INTRQ;
                if self.disks[self.dsk_idx].is_ready() {
                    self.track = 0;
                    let side = self.side;
                    self.disks[self.dsk_idx].seek(0, 1, side);
                } else {
                    self.status |= ST_SEEKERR;
                }
            }
            0x01 => {
                let data = self.data;
                let sector = self.sector;
                let side = self.side;
                self.disks[self.dsk_idx].seek(data, sector, side);
                self.track = self.data;
                self.intrq = PRT_INTRQ;
            }
            0x02 | 0x03 => {}
            0x04 | 0x05 => {}
            0x06 | 0x07 => {}
            0x08 | 0x09 => {
                self.commandtr = CMD_READSEC;
                self.status |= ST_BUSY;
                let sector = self.sector;
                self.disks[self.dsk_idx].read_sector(sector);
            }
            0x0A | 0x0B => {}
            0x0C => {
                self.commandtr = CMD_READADDR;
                self.status |= ST_BUSY;
                self.disks[self.dsk_idx].read_address();
            }
            0x0D => {}
            0x0E => {}
            0x0F => {}
            _ => {}
        }
    }

    pub fn write(&mut self, addr: u8, val: u8) {
        match addr {
            0 => self.command(val),
            1 => {
                self.track = val;
                self.status &= !ST_DRQ;
            }
            2 => {
                self.sector = val;
                self.status &= !ST_DRQ;
            }
            3 => {
                self.data = val;
                self.status &= !ST_DRQ;
            }
            4 => {
                self.pr = val;
                if (val & 1) != 0 {
                    self.dsk_idx = 0;
                } else if (val & 2) != 0 {
                    self.dsk_idx = 1;
                } else if (val & 4) != 0 {
                    self.dsk_idx = 2;
                } else if (val & 8) != 0 {
                    self.dsk_idx = 3;
                } else {
                    self.dsk_idx = 0;
                }
                self.side = (self.pr & 0x80) >> 7;
            }
            _ => {}
        }
    }
}
