use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

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
                is_disk: false,
            },
            ProgEntry {
                name: "Second".to_string(),
                file_name: "second.cas".to_string(),
                is_cas: true,
                is_disk: false,
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

    #[test]
    fn load_tape_snapshot_restores_selection_and_can_play() {
        let snapshot_path = std::path::Path::new("snapshots/load_tape.rtvcsnap.zip");
        if !snapshot_path.exists() {
            return;
        }

        let mut emu = Emu::new(MachineType {
            is_plus: true,
            rom_version: RomVersion::V1_2,
            has_dos: true,
        });
        emu.load_snapshot_file(snapshot_path).unwrap();
        assert_eq!(
            emu.progs
                .get(emu.selected_prog)
                .map(|entry| entry.file_name.as_str()),
            Some("TVBALL.CAS")
        );

        assert!(emu.tvc.bus.tape_motor_on());
        emu.play_tape();
        assert!(emu.tvc.bus.tape_play_active());
        let before = emu.tvc.bus.tape_elapsed_cycles();
        emu.tvc.run_for_a_frame();
        assert!(emu.tvc.bus.tape_elapsed_cycles() > before);
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
    pub is_disk: bool,
}

pub struct Emu {
    pub tvc: Tvc,
    pub running: bool,
    pub roms_loaded: bool,
    pub machine_type: MachineType,
    pub progs: Vec<ProgEntry>,
    pub selected_prog: usize,
    pub loaded_tape: Option<String>,
    pub loaded_disk: Option<String>,
    pub loaded_tape_file_name: Option<String>,
    pub loaded_disk_file_name: Option<String>,
    pub recent_tapes: Vec<String>,
    pub recent_disks: Vec<String>,
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
            loaded_tape: None,
            loaded_disk: None,
            loaded_tape_file_name: None,
            loaded_disk_file_name: None,
            recent_tapes: Vec::new(),
            recent_disks: Vec::new(),
        };
        emu.scan_progs();
        emu
    }

    pub fn tick(&mut self) -> bool {
        if !self.running {
            return false;
        }
        self.tvc.run_for_a_frame()
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
        self.loaded_tape = None;
        self.loaded_disk = None;
        self.loaded_tape_file_name = None;
        self.loaded_disk_file_name = None;
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
        self.load_roms();

        // Re-load the disk if there was one loaded
        if let Some(file_name) = self.loaded_disk_file_name.clone() {
            self.insert_disk_by_file_name(&file_name);
        }

        // Re-load the tape if there was one loaded/injected
        if let Some(file_name) = self.loaded_tape_file_name.clone() {
            let was_injected = self.loaded_tape.as_ref().map(|s| s.contains("(Injected)")).unwrap_or(false);
            if was_injected {
                self.inject_tape_by_file_name(&file_name);
            } else {
                let path = Path::new(&file_name);
                if path.exists() && path.is_file() {
                    let _ = self.play_tape_file_path(path);
                } else {
                    let path = data_dir("progs").join(&file_name);
                    let _ = self.play_tape_file_path(&path);
                }
            }
        }
    }

    pub fn select_prog_by_file_name(&mut self, file_name: &str) -> bool {
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
        if self.progs[self.selected_prog].is_disk {
            self.insert_selected_disk();
        }
    }

    pub fn load_roms(&mut self) {
        let roms_dir = data_dir("roms");
        if !roms_dir.exists() && roms_dir == Path::new("roms") {
            std::fs::create_dir_all(&roms_dir).ok();
        }
        let mut any_loaded = false;

        for name in self.machine_type.rom_files() {
            match std::fs::read(roms_dir.join(name)) {
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
        let dir = data_dir("progs");
        if !dir.exists() {
            return;
        }
        let mut entries: Vec<_> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    ext.eq_ignore_ascii_case("zip")
                        || ext.eq_ignore_ascii_case("cas")
                        || ext.eq_ignore_ascii_case("dsk")
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
                    .or_else(|| file_name.strip_suffix(".dsk"))
                    .unwrap_or(&file_name)
                    .to_string()
            };

            let path = dir.join(&file_name);
            let mut is_cas = false;
            let mut is_disk = false;
            if file_name.to_lowercase().ends_with(".cas") {
                is_cas = true;
            } else if file_name.to_lowercase().ends_with(".dsk") {
                is_disk = true;
            } else if file_name.to_lowercase().ends_with(".zip") {
                if let Ok(data) = std::fs::read(&path) {
                    let reader = std::io::Cursor::new(data);
                    if let Ok(mut archive) = zip::ZipArchive::new(reader) {
                        for i in 0..archive.len() {
                            if let Ok(file) = archive.by_index(i) {
                                let entry_name = file.name().to_lowercase();
                                if entry_name.ends_with(".cas") {
                                    is_cas = true;
                                } else if entry_name.ends_with(".dsk") {
                                    is_disk = true;
                                }
                                if is_cas && is_disk {
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
                is_disk,
            });
        }
        if self.selected_prog >= self.progs.len() {
            self.selected_prog = 0;
        }
    }

    pub fn load_selected_prog(&mut self) {
        if self
            .progs
            .get(self.selected_prog)
            .map(|entry| entry.is_disk)
            .unwrap_or(false)
        {
            self.insert_selected_disk();
        } else {
            self.inject_selected_tape();
        }
    }

    pub fn inject_selected_tape(&mut self) {
        if self.progs.is_empty() || self.selected_prog >= self.progs.len() {
            return;
        }
        let entry = &self.progs[self.selected_prog];
        if !entry.is_cas {
            return;
        }
        let path = data_dir("progs").join(&entry.file_name);
        if let Err(err) = self.inject_tape_file_path(&path) {
            eprintln!("failed to inject selected tape: {err}");
        }
    }

    pub fn insert_selected_disk(&mut self) {
        if self.progs.is_empty() || self.selected_prog >= self.progs.len() {
            return;
        }
        let entry = &self.progs[self.selected_prog];
        if !entry.is_disk {
            return;
        }
        let path = data_dir("progs").join(&entry.file_name);
        if let Err(err) = self.insert_disk_file_path(&path) {
            eprintln!("failed to insert selected disk: {err}");
        }
    }

    pub fn inject_tape_by_file_name(&mut self, file_name: &str) -> bool {
        let path = Path::new(file_name);
        if path.exists() && path.is_file() {
            if self.inject_tape_file_path(path).is_ok() {
                return true;
            }
        }
        if self.select_prog_by_file_name(file_name) {
            self.inject_selected_tape();
            self.loaded_tape_file_name.as_deref() == Some(file_name)
        } else {
            false
        }
    }

    pub fn insert_disk_by_file_name(&mut self, file_name: &str) -> bool {
        let path = Path::new(file_name);
        if path.exists() && path.is_file() {
            if self.insert_disk_file_path(path).is_ok() {
                return true;
            }
        }
        if self.select_prog_by_file_name(file_name) {
            self.insert_selected_disk();
            self.loaded_disk_file_name.as_deref() == Some(file_name)
        } else {
            false
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
        let path = data_dir("progs").join(&entry.file_name);
        if let Err(err) = self.play_tape_file_path(&path) {
            eprintln!("failed to play selected tape: {err}");
        }
    }

    pub fn play_tape_file_path(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let (display_name, buf) = read_cas_data(path)?;
        let generator = TapeBitstreamGenerator::new(&buf, &display_name)?;
        self.tvc.bus.play_tape(generator);
        self.loaded_tape = Some(display_name);
        let path_str = path.to_string_lossy().to_string();
        self.loaded_tape_file_name = Some(path_str.clone());
        self.add_recent_tape(path_str);
        Ok(())
    }

    pub fn inject_tape_file_path(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let (display_name, buf) = read_cas_data(path)?;
        if self.tvc.load_cas(&buf) {
            self.loaded_tape = Some(format!("{} (Injected)", display_name));
            let path_str = path.to_string_lossy().to_string();
            self.loaded_tape_file_name = Some(path_str.clone());
            self.add_recent_tape(path_str);
            Ok(())
        } else {
            Err("Failed to inject CAS data into memory".into())
        }
    }

    pub fn insert_disk_file_path(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let (display_name, buf) = read_dsk_data(path)?;
        self.tvc.load_disk(&display_name, &buf);
        self.loaded_disk = Some(display_name);
        let path_str = path.to_string_lossy().to_string();
        self.loaded_disk_file_name = Some(path_str.clone());
        self.add_recent_disk(path_str);
        Ok(())
    }

    pub fn stop_tape(&mut self) {
        self.tvc.bus.stop_tape();
    }

    pub fn get_current_tape_level(&self) -> f32 {
        if self.tvc.bus.tape_play_active() {
            return self.tvc.bus.current_tape_level();
        }
        0.5
    }

    pub fn add_recent_tape(&mut self, path_str: String) {
        self.recent_tapes.retain(|x| x != &path_str);
        self.recent_tapes.insert(0, path_str);
        self.recent_tapes.truncate(5);
    }

    pub fn add_recent_disk(&mut self, path_str: String) {
        self.recent_disks.retain(|x| x != &path_str);
        self.recent_disks.insert(0, path_str);
        self.recent_disks.truncate(5);
    }

    pub fn save_screenshot(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        const SRC_W: usize = 608;
        const SRC_H: usize = 288;
        const OUT_W: usize = 768;
        const OUT_H: usize = 576;

        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(writer, OUT_W as u32, OUT_H as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut png_writer = encoder.write_header()?;

        let mut pixels = vec![0; OUT_W * OUT_H * 4];
        for y in 0..OUT_H {
            let src_y = y * SRC_H / OUT_H;
            for x in 0..OUT_W {
                let src_x = x * SRC_W / OUT_W;
                let rgba = self.tvc.framebuffer[src_y * SRC_W + src_x].to_ne_bytes();
                let offset = (y * OUT_W + x) * 4;
                pixels[offset..offset + 4].copy_from_slice(&rgba);
            }
        }

        png_writer.write_image_data(&pixels)?;
        Ok(())
    }

    pub fn read_raw_bank(&self, bank: &str, addr: usize, len: usize) -> Option<Vec<u8>> {
        self.tvc.bus.mmu.read_raw_bank(bank, addr, len)
    }
}

fn data_dir(name: &str) -> PathBuf {
    let cwd_dir = PathBuf::from(name);
    if cwd_dir.exists() {
        return cwd_dir;
    }

    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(name)))
        .filter(|path| path.exists())
        .unwrap_or(cwd_dir)
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

fn read_cas_data(path: &Path) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
    let file_name = path
        .file_name()
        .ok_or("Invalid path")?
        .to_string_lossy()
        .to_string();
    let data = std::fs::read(path)?;
    if file_name.to_lowercase().ends_with(".cas") {
        Ok((file_name, data))
    } else if file_name.to_lowercase().ends_with(".zip") {
        let reader = std::io::Cursor::new(data);
        let mut archive = zip::ZipArchive::new(reader)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let entry_name = file.name().to_string();
            if entry_name.to_lowercase().ends_with(".cas") {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                let display_name = std::path::Path::new(&entry_name)
                    .file_name()
                    .unwrap_or(file.name().as_ref())
                    .to_string_lossy()
                    .to_string();
                return Ok((display_name, buf));
            }
        }
        Err("No .cas file found in zip archive".into())
    } else {
        Err("Unsupported file format (expected .cas or .zip)".into())
    }
}

fn read_dsk_data(path: &Path) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
    let file_name = path
        .file_name()
        .ok_or("Invalid path")?
        .to_string_lossy()
        .to_string();
    let data = std::fs::read(path)?;
    if file_name.to_lowercase().ends_with(".dsk") {
        Ok((file_name, data))
    } else if file_name.to_lowercase().ends_with(".zip") {
        let reader = std::io::Cursor::new(data);
        let mut archive = zip::ZipArchive::new(reader)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let entry_name = file.name().to_string();
            if entry_name.to_lowercase().ends_with(".dsk") {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                let display_name = std::path::Path::new(&entry_name)
                    .file_name()
                    .unwrap_or(file.name().as_ref())
                    .to_string_lossy()
                    .to_string();
                return Ok((display_name, buf));
            }
        }
        Err("No .dsk file found in zip archive".into())
    } else {
        Err("Unsupported file format (expected .dsk or .zip)".into())
    }
}
