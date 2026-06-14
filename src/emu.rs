use std::collections::VecDeque;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::cas::TapeBitstreamGenerator;
use crate::snapshot::{self, Reader, SnapshotError, Writer};
use crate::tvc::Tvc;

const GAMEBASE_BOOT_SNAPSHOT: &[u8] = include_bytes!("../data/snapshots/boot12dos.rtvcsnap.zip");

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
        emu.load_roms();
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
    fn game_archive_uses_exact_requested_media() {
        let archive = game_archive(&[
            ("OTHER.CAS", b"wrong"),
            ("folder/TARGET.CAS", b"correct"),
            ("disk.dsk", b"disk"),
        ]);

        assert_eq!(
            extract_game_archive_member("TARGET.CAS", &archive).unwrap(),
            b"correct"
        );
        assert_eq!(
            extract_game_archive_member("folder\\target.cas", &archive).unwrap(),
            b"correct"
        );
    }

    #[test]
    fn game_archive_reports_missing_media() {
        let archive = game_archive(&[("OTHER.CAS", b"wrong")]);
        let error = extract_game_archive_member("MISSING.CAS", &archive)
            .unwrap_err()
            .to_string();
        assert!(error.contains("MISSING.CAS"));
    }

    #[test]
    fn recent_media_replaces_same_display_name() {
        let mut emu = Emu::new(MachineType {
            is_plus: false,
            rom_version: RomVersion::V1_2,
            has_dos: false,
        });
        emu.recent_tapes = vec![
            "progs/TVBALL.CAS".to_string(),
            "old-cache/TVBALL.CAS".to_string(),
        ];

        emu.add_recent_tape("rtvc-media/tapes/tvball/TVBALL.CAS".to_string());

        assert_eq!(
            emu.recent_tapes,
            vec!["rtvc-media/tapes/tvball/TVBALL.CAS".to_string()]
        );
    }

    #[test]
    fn gamebase_cas_starts_from_clean_boot_and_queues_run() {
        let mut emu = Emu::new(MachineType {
            is_plus: false,
            rom_version: RomVersion::V2_2,
            has_dos: false,
        });
        let mut cas = vec![0; 145];
        cas[0] = 0x11;

        emu.start_gamebase_media_bytes("TEST.CAS", &cas).unwrap();

        assert_eq!(
            emu.machine_type,
            MachineType {
                is_plus: true,
                rom_version: RomVersion::V1_2,
                has_dos: true,
            }
        );
        assert!(emu.running);
        assert_eq!(emu.typed_text.iter().collect::<String>(), "RUN\r");
        assert_eq!(emu.loaded_tape.as_deref(), Some("TEST.CAS (Injected)"));
    }

    #[test]
    fn gamebase_disk_starts_from_clean_boot_and_queues_load() {
        let mut emu = Emu::new(MachineType {
            is_plus: false,
            rom_version: RomVersion::V2_2,
            has_dos: false,
        });
        let disk = vec![0; 368_640];

        emu.start_gamebase_media_bytes("TEST.DSK", &disk).unwrap();

        assert!(emu.running);
        assert_eq!(emu.typed_text.iter().collect::<String>(), "LOAD \"*\"\r");
        assert_eq!(emu.loaded_disk.as_deref(), Some("TEST.DSK"));
    }

    fn game_archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let cursor = std::io::Cursor::new(Vec::new());
        let mut archive = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, bytes) in entries {
            archive.start_file(*name, options).unwrap();
            archive.write_all(bytes).unwrap();
        }
        archive.finish().unwrap().into_inner()
    }

    #[test]
    fn load_tape_snapshot_restores_selection_and_can_play() {
        let snapshot_path = std::path::Path::new("data/snapshots/load_tape.rtvcsnap.zip");
        if !snapshot_path.exists() {
            return;
        }
        let snapshot_data = unzip_snapshot(&std::fs::read(snapshot_path).unwrap()).unwrap();
        if snapshot::read_file(&snapshot_data).is_err() {
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
    loaded_disk_file_name: Option<String>,
}

#[derive(Clone)]
pub struct WasmRecentFile {
    pub name: String,
    pub bytes: Vec<u8>,
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
    #[cfg(target_arch = "wasm32")]
    pub recent_tapes_wasm: Vec<WasmRecentFile>,
    #[cfg(target_arch = "wasm32")]
    pub recent_disks_wasm: Vec<WasmRecentFile>,
    #[cfg(target_arch = "wasm32")]
    loaded_tape_wasm: Option<WasmRecentFile>,
    #[cfg(target_arch = "wasm32")]
    loaded_disk_wasm: Option<WasmRecentFile>,
    #[cfg(target_arch = "wasm32")]
    loaded_tape_was_injected: bool,
    typed_text: VecDeque<char>,
    typed_key: Option<u32>,
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
            #[cfg(target_arch = "wasm32")]
            recent_tapes_wasm: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            recent_disks_wasm: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            loaded_tape_wasm: None,
            #[cfg(target_arch = "wasm32")]
            loaded_disk_wasm: None,
            #[cfg(target_arch = "wasm32")]
            loaded_tape_was_injected: false,
            typed_text: VecDeque::new(),
            typed_key: None,
        };
        emu.scan_progs();
        emu
    }

    pub fn tick(&mut self) -> bool {
        if !self.running {
            return false;
        }
        self.advance_typed_text();
        self.tvc.run_for_a_frame()
    }

    pub fn reset(&mut self) {
        self.clear_typed_text();
        self.tvc.reset();
    }

    fn clear_typed_text(&mut self) {
        if let Some(code) = self.typed_key.take() {
            self.tvc.key_up(code);
        }
        self.typed_text.clear();
    }

    fn queue_typed_text(&mut self, text: &str) {
        self.clear_typed_text();
        self.typed_text.extend(text.chars());
    }

    fn advance_typed_text(&mut self) {
        if let Some(code) = self.typed_key.take() {
            self.tvc.key_up(code);
            return;
        }
        let Some(ch) = self.typed_text.pop_front() else {
            return;
        };
        let code = ch as u32;
        self.tvc.key_down(code);
        if ch != '\r' {
            self.tvc.key_press(ch);
        }
        self.typed_key = Some(code);
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
        let mut writer = Writer::new();
        writer.string(
            self.progs
                .get(self.selected_prog)
                .map(|entry| entry.file_name.as_str())
                .unwrap_or(""),
        );
        writer.string(self.loaded_disk_file_name.as_deref().unwrap_or(""));
        chunks.push((*b"EMUI", writer.into_inner()));
        snapshot::write_file(&chunks)
    }

    pub fn load_snapshot(&mut self, data: &[u8]) -> crate::snapshot::Result<()> {
        self.clear_typed_text();
        let fast_boot = self.tvc.fast_boot();
        let snapshot_state = Self::read_emu_snapshot_state(data)?;
        #[cfg(target_arch = "wasm32")]
        let snapshot_disk = snapshot_state
            .loaded_disk_file_name
            .as_ref()
            .and_then(|name| {
                self.loaded_disk_wasm
                    .iter()
                    .chain(self.recent_disks_wasm.iter())
                    .find(|media| recent_media_key(&media.name) == recent_media_key(name))
                    .cloned()
            });
        let machine_type = snapshot_state.machine_type.unwrap_or_else(|| {
            MachineType::for_snapshot(
                self.tvc.is_plus(),
                self.tvc.has_hbf(),
                self.machine_type.rom_version,
            )
        });
        self.machine_type = machine_type;
        self.tvc = Tvc::new(machine_type.is_plus);
        self.roms_loaded = false;
        self.load_roms();
        self.tvc.set_fast_boot(fast_boot);
        self.tvc.load_snapshot(data)?;
        self.loaded_tape = None;
        self.loaded_disk = None;
        self.loaded_tape_file_name = None;
        self.loaded_disk_file_name = None;
        #[cfg(target_arch = "wasm32")]
        {
            self.loaded_tape_wasm = None;
            self.loaded_disk_wasm = None;
            self.loaded_tape_was_injected = false;
        }
        if let Some(file_name) = snapshot_state
            .selected_prog_file_name
            .as_deref()
            .filter(|name| !name.is_empty())
        {
            self.select_prog_by_file_name(&file_name);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(file_name) = snapshot_state.loaded_disk_file_name {
            self.insert_disk_by_file_name(&file_name);
        } else {
            self.restore_accessible_selected_media();
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(media) = snapshot_disk {
            let _ = self.insert_disk_bytes(&media.name, &media.bytes);
        } else {
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
                    let disk_file_name = reader.string()?;
                    if !disk_file_name.is_empty() {
                        state.loaded_disk_file_name = Some(disk_file_name);
                    }
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

    pub fn reload(&mut self, machine_type: MachineType) -> Result<(), String> {
        let fast_boot = self.tvc.fast_boot();
        #[cfg(target_arch = "wasm32")]
        let loaded_disk = self.loaded_disk_wasm.clone();
        #[cfg(target_arch = "wasm32")]
        let loaded_tape = self.loaded_tape_wasm.clone();
        #[cfg(target_arch = "wasm32")]
        let tape_was_injected = self.loaded_tape_was_injected;

        self.machine_type = machine_type;
        self.tvc = Tvc::new(machine_type.is_plus);
        self.tvc.set_fast_boot(fast_boot);
        self.roms_loaded = false;
        self.load_roms();

        #[cfg(target_arch = "wasm32")]
        {
            let mut errors = Vec::new();
            if let Some(media) = loaded_disk {
                if let Err(err) = self.insert_disk_bytes(&media.name, &media.bytes) {
                    self.loaded_disk = None;
                    self.loaded_disk_file_name = None;
                    self.loaded_disk_wasm = None;
                    errors.push(format!("disk restore failed: {err}"));
                }
            }
            if let Some(media) = loaded_tape {
                let result = if tape_was_injected {
                    self.inject_tape_bytes(&media.name, &media.bytes)
                } else {
                    self.play_tape_bytes(&media.name, &media.bytes)
                };
                if let Err(err) = result {
                    self.loaded_tape = None;
                    self.loaded_tape_file_name = None;
                    self.loaded_tape_wasm = None;
                    self.loaded_tape_was_injected = false;
                    errors.push(format!("tape restore failed: {err}"));
                }
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors.join("; "))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut errors = Vec::new();
            if let Some(file_name) = self.loaded_disk_file_name.clone() {
                if !self.insert_disk_by_file_name(&file_name) {
                    self.loaded_disk = None;
                    self.loaded_disk_file_name = None;
                    errors.push(format!("disk restore failed: {file_name}"));
                }
            }

            if let Some(file_name) = self.loaded_tape_file_name.clone() {
                let was_injected = self
                    .loaded_tape
                    .as_ref()
                    .map(|s| s.contains("(Injected)"))
                    .unwrap_or(false);
                let restored = if was_injected {
                    self.inject_tape_by_file_name(&file_name)
                } else {
                    let path = Path::new(&file_name);
                    if path.exists() && path.is_file() {
                        self.play_tape_file_path(path).is_ok()
                    } else {
                        let path = data_dir("progs").join(&file_name);
                        self.play_tape_file_path(&path).is_ok()
                    }
                };
                if !restored {
                    self.loaded_tape = None;
                    self.loaded_tape_file_name = None;
                    errors.push(format!("tape restore failed: {file_name}"));
                }
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors.join("; "))
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
        #[cfg(target_arch = "wasm32")]
        {
            let roms: &[(&str, &[u8])] = &[
                ("TVC12_D3.64K", include_bytes!("../roms/TVC12_D3.64K")),
                ("TVC12_D4.64K", include_bytes!("../roms/TVC12_D4.64K")),
                ("TVC12_D7.64K", include_bytes!("../roms/TVC12_D7.64K")),
                ("TVC22_D4.64K", include_bytes!("../roms/TVC22_D4.64K")),
                ("TVC22_D6.64K", include_bytes!("../roms/TVC22_D6.64K")),
                ("TVC22_D7.64K", include_bytes!("../roms/TVC22_D7.64K")),
                ("D_TVCDOS.128", include_bytes!("../roms/D_TVCDOS.128")),
            ];
            for name in self.machine_type.rom_files() {
                if let Some((_, data)) = roms.iter().find(|(n, _)| *n == name) {
                    self.tvc.add_rom(name, data);
                }
            }
            self.roms_loaded = true;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
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
    }

    pub fn scan_progs(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            self.progs.clear();
            self.selected_prog = 0;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
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

    pub fn can_inject_tape(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        if self.loaded_tape_wasm.is_some() {
            return true;
        }

        #[cfg(not(target_arch = "wasm32"))]
        if self.loaded_tape_file_name.is_some() {
            return true;
        }

        self.progs
            .get(self.selected_prog)
            .map(|entry| entry.is_cas)
            .unwrap_or(false)
    }

    pub fn inject_tape(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_arch = "wasm32")]
        if let Some(media) = self.loaded_tape_wasm.clone() {
            return self.inject_tape_bytes(&media.name, &media.bytes);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(file_name) = self.loaded_tape_file_name.clone() {
            return self.inject_tape_file_path(Path::new(&file_name));
        }

        let entry = self
            .progs
            .get(self.selected_prog)
            .filter(|entry| entry.is_cas)
            .ok_or("No cassette tape is selected")?;
        let path = data_dir("progs").join(&entry.file_name);
        self.inject_tape_file_path(&path)
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
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(recent_path) = self
            .recent_disks
            .iter()
            .find(|recent| {
                recent_media_key(recent) == recent_media_key(file_name)
                    && Path::new(recent).is_file()
            })
            .cloned()
        {
            if self.insert_disk_file_path(Path::new(&recent_path)).is_ok() {
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
        let key = recent_media_key(&path_str);
        self.recent_tapes
            .retain(|existing| recent_media_key(existing) != key);
        self.recent_tapes.insert(0, path_str);
        self.recent_tapes.truncate(5);
    }

    pub fn add_recent_disk(&mut self, path_str: String) {
        let key = recent_media_key(&path_str);
        self.recent_disks
            .retain(|existing| recent_media_key(existing) != key);
        self.recent_disks.insert(0, path_str);
        self.recent_disks.truncate(5);
    }

    pub fn get_screenshot_png(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        const SRC_W: usize = 608;
        const SRC_H: usize = 288;
        const OUT_W: usize = 768;
        const OUT_H: usize = 576;

        let mut buf = Vec::new();
        {
            let writer = std::io::Cursor::new(&mut buf);
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
        }
        Ok(buf)
    }

    pub fn save_screenshot(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let png = self.get_screenshot_png()?;
        std::fs::write(path, png)?;
        Ok(())
    }

    pub fn play_tape_bytes(
        &mut self,
        name: &str,
        bytes: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (display_name, buf) = unpack_cas_bytes(name, bytes)?;
        let generator = TapeBitstreamGenerator::new(&buf, &display_name)?;
        self.tvc.bus.play_tape(generator);
        self.loaded_tape = Some(display_name);
        self.loaded_tape_file_name = Some(name.to_string());
        #[cfg(target_arch = "wasm32")]
        {
            let media = WasmRecentFile {
                name: name.to_string(),
                bytes: bytes.to_vec(),
            };
            self.loaded_tape_wasm = Some(media.clone());
            self.loaded_tape_was_injected = false;
            self.add_recent_tape_wasm(media.name, media.bytes);
        }
        Ok(())
    }

    pub fn inject_tape_bytes(
        &mut self,
        name: &str,
        bytes: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (display_name, buf) = unpack_cas_bytes(name, bytes)?;
        if self.tvc.load_cas(&buf) {
            self.loaded_tape = Some(format!("{} (Injected)", display_name));
            self.loaded_tape_file_name = Some(name.to_string());
            #[cfg(target_arch = "wasm32")]
            {
                let media = WasmRecentFile {
                    name: name.to_string(),
                    bytes: bytes.to_vec(),
                };
                self.loaded_tape_wasm = Some(media.clone());
                self.loaded_tape_was_injected = true;
                self.add_recent_tape_wasm(media.name, media.bytes);
            }
            Ok(())
        } else {
            Err("Failed to inject CAS data into memory".into())
        }
    }

    pub fn insert_disk_bytes(
        &mut self,
        name: &str,
        bytes: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (display_name, buf) = unpack_dsk_bytes(name, bytes)?;
        self.tvc.load_disk(&display_name, &buf);
        self.loaded_disk = Some(display_name);
        self.loaded_disk_file_name = Some(name.to_string());
        #[cfg(target_arch = "wasm32")]
        {
            let media = WasmRecentFile {
                name: name.to_string(),
                bytes: bytes.to_vec(),
            };
            self.loaded_disk_wasm = Some(media.clone());
            self.add_recent_disk_wasm(media.name, media.bytes);
        }
        Ok(())
    }

    pub fn load_game_archive_bytes(
        &mut self,
        file_to_run: &str,
        archive_bytes: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = extract_game_archive_member(file_to_run, archive_bytes)?;
        self.start_gamebase_media_bytes(file_to_run, &bytes)
    }

    pub fn start_gamebase_media_bytes(
        &mut self,
        file_to_run: &str,
        bytes: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.load_gamebase_boot()?;
        let lower_name = file_to_run.to_ascii_lowercase();
        if lower_name.ends_with(".cas") {
            self.inject_tape_bytes(file_to_run, bytes)?;
            self.finish_gamebase_start("RUN\r");
        } else if lower_name.ends_with(".dsk") {
            self.insert_disk_bytes(file_to_run, bytes)?;
            self.finish_gamebase_start("LOAD \"*\"\r");
        } else {
            return Err(format!("Unsupported game media: {file_to_run}").into());
        }
        Ok(())
    }

    pub fn start_gamebase_media_file(
        &mut self,
        file_to_run: &str,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.load_gamebase_boot()?;
        let lower_name = file_to_run.to_ascii_lowercase();
        if lower_name.ends_with(".cas") {
            self.inject_tape_file_path(path)?;
            self.finish_gamebase_start("RUN\r");
        } else if lower_name.ends_with(".dsk") {
            self.insert_disk_file_path(path)?;
            self.finish_gamebase_start("LOAD \"*\"\r");
        } else {
            return Err(format!("Unsupported game media: {file_to_run}").into());
        }
        Ok(())
    }

    fn load_gamebase_boot(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.running = false;
        let snapshot = unzip_snapshot(GAMEBASE_BOOT_SNAPSHOT)?;
        self.load_snapshot(&snapshot)?;
        Ok(())
    }

    fn finish_gamebase_start(&mut self, command: &str) {
        self.queue_typed_text(command);
        self.running = true;
    }

    #[cfg(target_arch = "wasm32")]
    pub fn add_recent_tape_wasm(&mut self, name: String, bytes: Vec<u8>) {
        self.recent_tapes_wasm.retain(|x| x.name != name);
        self.recent_tapes_wasm
            .insert(0, WasmRecentFile { name, bytes });
        self.recent_tapes_wasm.truncate(5);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn add_recent_disk_wasm(&mut self, name: String, bytes: Vec<u8>) {
        self.recent_disks_wasm.retain(|x| x.name != name);
        self.recent_disks_wasm
            .insert(0, WasmRecentFile { name, bytes });
        self.recent_disks_wasm.truncate(5);
    }

    pub fn read_raw_bank(&self, bank: &str, addr: usize, len: usize) -> Option<Vec<u8>> {
        self.tvc.bus.mmu.read_raw_bank(bank, addr, len)
    }
}

pub fn extract_game_archive_member(
    file_to_run: &str,
    archive_bytes: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let reader = std::io::Cursor::new(archive_bytes);
    let mut archive = zip::ZipArchive::new(reader)?;
    let target_name = file_to_run.replace('\\', "/");
    let target_file_name = target_name.rsplit('/').next().unwrap_or(&target_name);

    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        let entry_name = file.name().replace('\\', "/");
        let entry_file_name = entry_name.rsplit('/').next().unwrap_or(&entry_name);
        if entry_name.eq_ignore_ascii_case(&target_name)
            || entry_file_name.eq_ignore_ascii_case(target_file_name)
        {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            return Ok(bytes);
        }
    }

    Err(format!("{file_to_run} was not found in the game archive").into())
}

fn recent_media_key(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_lowercase()
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

pub fn is_zip_data(data: &[u8]) -> bool {
    data.starts_with(b"PK\x03\x04")
}

pub fn zip_snapshot(snapshot: &[u8]) -> std::io::Result<Vec<u8>> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut archive = zip::ZipWriter::new(cursor);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    archive.start_file("snapshot.rtvcsnap", options)?;
    archive.write_all(snapshot)?;
    Ok(archive.finish()?.into_inner())
}

pub fn unzip_snapshot(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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
    unpack_cas_bytes(&file_name, &data)
}

fn read_dsk_data(path: &Path) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
    let file_name = path
        .file_name()
        .ok_or("Invalid path")?
        .to_string_lossy()
        .to_string();
    let data = std::fs::read(path)?;
    unpack_dsk_bytes(&file_name, &data)
}

pub fn unpack_cas_bytes(
    file_name: &str,
    data: &[u8],
) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
    if file_name.to_lowercase().ends_with(".cas") {
        Ok((file_name.to_string(), data.to_vec()))
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

pub fn unpack_dsk_bytes(
    file_name: &str,
    data: &[u8],
) -> Result<(String, Vec<u8>), Box<dyn std::error::Error>> {
    if file_name.to_lowercase().ends_with(".dsk") {
        Ok((file_name.to_string(), data.to_vec()))
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
