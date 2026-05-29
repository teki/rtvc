use std::io::Read;
use std::io::Write;

use crate::tvc::Tvc;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RomVersion {
    V1_2,
    V2_2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zipped_snapshot_round_trips() {
        let mut emu = Emu::new(MachineType {
            is_plus: true,
            rom_version: RomVersion::V1_2,
            has_dos: false,
        });
        emu.tvc.z80.state.r16[11] = 0xBEEF;
        let zipped = zip_snapshot(&emu.save_snapshot()).unwrap();
        assert!(zipped.len() < emu.save_snapshot().len());

        let raw = unzip_snapshot(&zipped).unwrap();
        let mut restored = Emu::new(emu.machine_type);
        restored.load_snapshot(&raw).unwrap();
        assert_eq!(restored.tvc.z80.state.r16[11], 0xBEEF);
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MachineType {
    pub is_plus: bool,
    pub rom_version: RomVersion,
    pub has_dos: bool,
}

impl MachineType {
    pub fn label(&self) -> String {
        match (self.is_plus, self.rom_version, self.has_dos) {
            (true, RomVersion::V1_2, true) => "64k+ 1.2, VT-DOS",
            (true, RomVersion::V2_2, true) => "64k+ 2.2, VT-DOS",
            (false, RomVersion::V1_2, false) => "64k  1.2",
            (true, RomVersion::V1_2, false) => "64k+ 1.2",
            (true, RomVersion::V2_2, false) => "64k+ 2.2",
            _ => "64k  1.2",
        }
        .to_string()
    }

    pub fn all_types() -> Vec<MachineType> {
        vec![
            MachineType {
                is_plus: true,
                rom_version: RomVersion::V1_2,
                has_dos: true,
            },
            MachineType {
                is_plus: true,
                rom_version: RomVersion::V2_2,
                has_dos: true,
            },
            MachineType {
                is_plus: false,
                rom_version: RomVersion::V1_2,
                has_dos: false,
            },
            MachineType {
                is_plus: true,
                rom_version: RomVersion::V1_2,
                has_dos: false,
            },
            MachineType {
                is_plus: true,
                rom_version: RomVersion::V2_2,
                has_dos: false,
            },
        ]
    }

    fn rom_files(&self) -> Vec<&'static str> {
        let mut files = match self.rom_version {
            RomVersion::V1_2 => vec!["TVC12_D3.64K", "TVC12_D4.64K", "TVC12_D7.64K"],
            RomVersion::V2_2 => vec!["TVC22_D4.64K", "TVC22_D6.64K", "TVC22_D7.64K"],
        };
        if self.has_dos {
            files.push("D_TVCDOS.128");
        }
        files
    }
}

#[derive(Clone)]
pub struct ProgEntry {
    pub name: String,
    pub file_name: String,
}

pub struct Emu {
    pub tvc: Tvc,
    pub running: bool,
    pub roms_loaded: bool,
    pub machine_type: MachineType,
    pub progs: Vec<ProgEntry>,
    pub selected_prog: usize,
    pub prog_loaded: Option<String>,
}

impl Emu {
    pub fn new(machine_type: MachineType) -> Self {
        let mut emu = Emu {
            tvc: Tvc::new(machine_type.is_plus),
            running: true,
            roms_loaded: false,
            machine_type,
            progs: Vec::new(),
            selected_prog: 0,
            prog_loaded: None,
        };
        emu.scan_progs();
        emu
    }

    pub fn tick(&mut self) {
        if !self.running {
            return;
        }
        self.tvc.run_for_a_frame();
    }

    pub fn reset(&mut self) {
        self.tvc.reset();
    }

    pub fn save_snapshot(&self) -> Vec<u8> {
        self.tvc.save_snapshot()
    }

    pub fn load_snapshot(&mut self, data: &[u8]) -> crate::snapshot::Result<()> {
        self.tvc.load_snapshot(data)
    }

    pub fn save_snapshot_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let snapshot = self.save_snapshot();
        if is_zip_path(path) {
            std::fs::write(path, zip_snapshot(&snapshot)?)
        } else {
            std::fs::write(path, snapshot)
        }
    }

    pub fn load_snapshot_file(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = std::fs::read(path)?;
        if is_zip_data(&data) {
            data = unzip_snapshot(&data)?;
        }
        self.load_snapshot(&data)?;
        Ok(())
    }

    pub fn toggle_running(&mut self) {
        self.running = !self.running;
    }

    pub fn reload(&mut self, machine_type: MachineType) {
        self.machine_type = machine_type;
        self.tvc = Tvc::new(machine_type.is_plus);
        self.roms_loaded = false;
        self.prog_loaded = None;
        self.load_roms();
    }

    pub fn load_roms(&mut self) {
        std::fs::create_dir_all("roms").ok();
        let mut any_loaded = false;

        for name in self.machine_type.rom_files() {
            let path = format!("roms/{}", name);
            match std::fs::read(&path) {
                Ok(data) => {
                    self.tvc.add_rom(name, &data);
                    any_loaded = true;
                }
                Err(_) => {}
            }
        }

        if any_loaded {
            self.roms_loaded = true;
        }
    }

    pub fn scan_progs(&mut self) {
        self.progs.clear();
        let dir = std::path::Path::new("progs");
        if !dir.exists() {
            return;
        }
        let mut entries: Vec<_> = match std::fs::read_dir(dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "zip")
                        .unwrap_or(false)
                })
                .collect(),
            Err(_) => return,
        };
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let name = file_name
                .strip_suffix(".zip")
                .unwrap_or(&file_name)
                .to_string();
            self.progs.push(ProgEntry { name, file_name });
        }
        if self.selected_prog >= self.progs.len() {
            self.selected_prog = 0;
        }
    }

    pub fn load_selected_prog(&mut self) {
        if self.progs.is_empty() || self.selected_prog >= self.progs.len() {
            return;
        }
        let file_name = self.progs[self.selected_prog].file_name.clone();
        let path = format!("progs/{}", file_name);

        let data = match std::fs::read(&path) {
            Ok(d) => d,
            Err(_) => return,
        };

        let reader = std::io::Cursor::new(data);
        let mut archive = match zip::ZipArchive::new(reader) {
            Ok(a) => a,
            Err(_) => return,
        };

        for i in 0..archive.len() {
            let mut file = match archive.by_index(i) {
                Ok(f) => f,
                Err(_) => continue,
            };
            let entry_name = file.name().to_string();
            if entry_name.to_lowercase().ends_with(".dsk") {
                let mut buf = Vec::new();
                if file.read_to_end(&mut buf).is_ok() {
                      self.tvc.load_disk(&entry_name, &buf);
                      self.prog_loaded = Some(self.progs[self.selected_prog].name.clone());
                }
                break;
            }
        }
    }
}

fn is_zip_path(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
}

fn is_zip_data(data: &[u8]) -> bool {
    data.starts_with(b"PK\x03\x04")
}

fn zip_snapshot(snapshot: &[u8]) -> std::io::Result<Vec<u8>> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut archive = zip::ZipWriter::new(cursor);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    archive.start_file("snapshot.rtvcsnap", options)?;
    archive.write_all(snapshot)?;
    Ok(archive.finish()?.into_inner())
}

fn unzip_snapshot(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let reader = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_ascii_lowercase();
        if name.ends_with(".rtvcsnap") {
            let mut snapshot = Vec::new();
            file.read_to_end(&mut snapshot)?;
            return Ok(snapshot);
        }
    }

    Err("zip archive does not contain a .rtvcsnap file".into())
}
