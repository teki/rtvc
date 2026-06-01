use std::io::Read;
use std::io::Write;

use crate::cas::TapeBitstreamGenerator;
use crate::snapshot::{self, Reader, SnapshotError, Writer};
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
            rom_version: RomVersion::V2_2,
            has_dos: true,
        });
        emu.tvc.z80.state.r16[11] = 0xBEEF;
        let zipped = zip_snapshot(&emu.save_snapshot()).unwrap();
        assert!(zipped.len() < emu.save_snapshot().len());

        let raw = unzip_snapshot(&zipped).unwrap();
        let mut restored = Emu::new(MachineType {
            is_plus: false,
            rom_version: RomVersion::V1_2,
            has_dos: false,
        });
        restored.load_snapshot(&raw).unwrap();
        assert_eq!(restored.machine_type, emu.machine_type);
        assert_eq!(restored.tvc.z80.state.r16[11], 0xBEEF);
    }

    #[test]
    fn snapshot_restores_selected_program() {
        let mut emu = Emu::new(MachineType {
            is_plus: true,
            rom_version: RomVersion::V1_2,
            has_dos: false,
        });
        emu.progs = vec![
            ProgEntry {
                name: "First".to_string(),
                file_name: "first.cas".to_string(),
                is_cas: true,
            },
            ProgEntry {
                name: "Second".to_string(),
                file_name: "second.cas".to_string(),
                is_cas: true,
            },
        ];
        emu.selected_prog = 1;

        let snapshot = emu.save_snapshot();
        let mut restored = Emu::new(emu.machine_type);
        restored.progs = emu.progs.clone();
        restored.selected_prog = 0;
        restored.load_snapshot(&snapshot).unwrap();

        assert_eq!(restored.selected_prog, 1);
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

    pub fn for_snapshot(is_plus: bool, has_dos: bool, preferred_rom_version: RomVersion) -> Self {
        Self::all_types()
            .into_iter()
            .find(|machine_type| {
                machine_type.is_plus == is_plus
                    && machine_type.has_dos == has_dos
                    && machine_type.rom_version == preferred_rom_version
            })
            .or_else(|| {
                Self::all_types().into_iter().find(|machine_type| {
                    machine_type.is_plus == is_plus && machine_type.has_dos == has_dos
                })
            })
            .unwrap_or(MachineType {
                is_plus,
                rom_version: RomVersion::V1_2,
                has_dos,
            })
    }

    fn rom_version_id(&self) -> u8 {
        match self.rom_version {
            RomVersion::V1_2 => 0,
            RomVersion::V2_2 => 1,
        }
    }

    fn from_snapshot_chunk(data: &[u8]) -> snapshot::Result<Self> {
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
        let has_dos = reader.u8()? != 0;
        Self::all_types()
            .into_iter()
            .find(|machine_type| {
                machine_type.is_plus == is_plus
                    && machine_type.rom_version == rom_version
                    && machine_type.has_dos == has_dos
            })
            .ok_or_else(|| SnapshotError::InvalidData("unknown machine type".to_string()))
    }

    fn write_snapshot_chunk(&self) -> Vec<u8> {
        let mut writer = Writer::new();
        writer.u8(self.is_plus as u8);
        writer.u8(self.rom_version_id());
        writer.u8(self.has_dos as u8);
        writer.into_inner()
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

#[derive(Default)]
struct EmuSnapshotState {
    machine_type: Option<MachineType>,
    selected_prog_file_name: Option<String>,
}

#[derive(Clone)]
pub struct ProgEntry {
    pub name: String,
    pub file_name: String,
    pub is_cas: bool,
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
        let core_snapshot = self.tvc.save_snapshot();
        let Ok(core_chunks) = snapshot::read_file(&core_snapshot) else {
            return core_snapshot;
        };
        let mut chunks: Vec<_> = core_chunks
            .into_iter()
            .map(|chunk| (chunk.id, chunk.data.to_vec()))
            .collect();
        chunks.push((*b"EMUT", self.machine_type.write_snapshot_chunk()));
        if let Some(entry) = self.progs.get(self.selected_prog) {
            let mut writer = Writer::new();
            writer.string(&entry.file_name);
            chunks.push((*b"EMUI", writer.into_inner()));
        }
        snapshot::write_file(&chunks)
    }

    pub fn load_snapshot(&mut self, data: &[u8]) -> crate::snapshot::Result<()> {
        let snapshot_state = Self::read_emu_snapshot_state(data)?;
        self.tvc.load_snapshot(data)?;
        self.machine_type = snapshot_state.machine_type.unwrap_or_else(|| {
            MachineType::for_snapshot(
                self.tvc.is_plus(),
                self.tvc.has_hbf(),
                self.machine_type.rom_version,
            )
        });
        self.roms_loaded = true;
        self.prog_loaded = None;
        if let Some(file_name) = snapshot_state.selected_prog_file_name {
            self.select_prog_by_file_name(&file_name);
            self.restore_accessible_selected_media();
        }
        Ok(())
    }

    fn read_emu_snapshot_state(data: &[u8]) -> crate::snapshot::Result<EmuSnapshotState> {
        let chunks = snapshot::read_file(data)?;
        let mut state = EmuSnapshotState::default();
        for chunk in chunks {
            match &chunk.id {
                b"EMUT" => state.machine_type = Some(MachineType::from_snapshot_chunk(chunk.data)?),
                b"EMUI" => {
                    let mut reader = Reader::new(chunk.data);
                    state.selected_prog_file_name = Some(reader.string()?);
                }
                _ => {}
            }
        }
        Ok(state)
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

    fn select_prog_by_file_name(&mut self, file_name: &str) -> bool {
        if let Some(index) = self
            .progs
            .iter()
            .position(|entry| entry.file_name == file_name)
        {
            self.selected_prog = index;
            true
        } else {
            false
        }
    }

    fn restore_accessible_selected_media(&mut self) {
        if self.progs.is_empty() || self.selected_prog >= self.progs.len() {
            return;
        }
        if !self.progs[self.selected_prog].is_cas {
            self.load_selected_prog();
        }
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
                    let path = e.path();
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    ext.eq_ignore_ascii_case("zip") || ext.eq_ignore_ascii_case("cas")
                })
                .collect(),
            Err(_) => return,
        };
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let name = if file_name.to_lowercase().ends_with(".zip") {
                file_name
                    .strip_suffix(".zip")
                    .unwrap_or(&file_name)
                    .to_string()
            } else {
                file_name
                    .strip_suffix(".cas")
                    .unwrap_or(&file_name)
                    .to_string()
            };

            let path = dir.join(&file_name);
            let mut is_cas = false;
            if file_name.to_lowercase().ends_with(".cas") {
                is_cas = true;
            } else if file_name.to_lowercase().ends_with(".zip") {
                if let Ok(data) = std::fs::read(&path) {
                    let reader = std::io::Cursor::new(data);
                    if let Ok(mut archive) = zip::ZipArchive::new(reader) {
                        for i in 0..archive.len() {
                            if let Ok(file) = archive.by_index(i) {
                                let entry_name = file.name().to_lowercase();
                                if entry_name.ends_with(".cas") {
                                    is_cas = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            self.progs.push(ProgEntry {
                name,
                file_name,
                is_cas,
            });
        }
        if self.selected_prog >= self.progs.len() {
            self.selected_prog = 0;
        }
    }

    pub fn load_selected_prog(&mut self) {
        if self.progs.is_empty() || self.selected_prog >= self.progs.len() {
            return;
        }
        let entry = &self.progs[self.selected_prog];
        let file_name = entry.file_name.clone();
        let path = format!("progs/{}", file_name);

        let data = match std::fs::read(&path) {
            Ok(d) => d,
            Err(_) => return,
        };

        let mut cas_data = None;
        let mut dsk_data = None;

        if file_name.to_lowercase().ends_with(".cas") {
            cas_data = Some(data);
        } else if file_name.to_lowercase().ends_with(".zip") {
            let reader = std::io::Cursor::new(data);
            if let Ok(mut archive) = zip::ZipArchive::new(reader) {
                for i in 0..archive.len() {
                    let mut file = match archive.by_index(i) {
                        Ok(f) => f,
                        Err(_) => continue,
                    };
                    let entry_name = file.name().to_string();
                    if entry_name.to_lowercase().ends_with(".dsk") {
                        let mut buf = Vec::new();
                        if file.read_to_end(&mut buf).is_ok() {
                            dsk_data = Some((entry_name, buf));
                        }
                        break;
                    } else if entry_name.to_lowercase().ends_with(".cas") {
                        let mut buf = Vec::new();
                        if file.read_to_end(&mut buf).is_ok() {
                            cas_data = Some(buf);
                        }
                        break;
                    }
                }
            }
        }

        if let Some((dsk_name, buf)) = dsk_data {
            self.tvc.load_disk(&dsk_name, &buf);
            self.prog_loaded = Some(entry.name.clone());
        } else if let Some(buf) = cas_data {
            if self.tvc.load_cas(&buf) {
                self.prog_loaded = Some(format!("{} (Injected)", entry.name));
            }
        }
    }

    pub fn play_tape(&mut self) {
        if self.progs.is_empty() || self.selected_prog >= self.progs.len() {
            return;
        }
        let entry = &self.progs[self.selected_prog];
        if !entry.is_cas {
            return;
        }
        let file_name = entry.file_name.clone();
        let path = format!("progs/{}", file_name);

        let data = match std::fs::read(&path) {
            Ok(d) => d,
            Err(_) => return,
        };

        let mut cas_data = None;

        if file_name.to_lowercase().ends_with(".cas") {
            cas_data = Some(data);
        } else if file_name.to_lowercase().ends_with(".zip") {
            let reader = std::io::Cursor::new(data);
            if let Ok(mut archive) = zip::ZipArchive::new(reader) {
                for i in 0..archive.len() {
                    let mut file = match archive.by_index(i) {
                        Ok(f) => f,
                        Err(_) => continue,
                    };
                    let entry_name = file.name().to_string();
                    if entry_name.to_lowercase().ends_with(".cas") {
                        let mut buf = Vec::new();
                        if file.read_to_end(&mut buf).is_ok() {
                            cas_data = Some(buf);
                        }
                        break;
                    }
                }
            }
        }

        if let Some(buf) = cas_data {
            if let Ok(generator) = TapeBitstreamGenerator::new(&buf, &entry.name) {
                self.tvc.bus.play_tape(generator);
                self.prog_loaded = Some(format!("{} (Playing)", entry.name));
            }
        }
    }

    pub fn stop_tape(&mut self) {
        self.tvc.bus.stop_tape();
        self.prog_loaded = None;
    }

    pub fn get_current_tape_level(&self) -> f32 {
        if self.tvc.bus.tape_play_active() {
            return self.tvc.bus.current_tape_level();
        }
        0.5
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
