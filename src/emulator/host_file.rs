//! Virtual host file card: a `FILE` device backed by a host directory or a
//! browser file store. The card ROM talks to this device through card-local
//! I/O ports; CAS container handling lives here, not in the ROM.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::cas::{
    TVC_CAS_TYPE_BUFFERED, TVC_CAS_TYPE_UNBUFFERED, decode_cas_container, encode_cas_container,
};
use crate::expansion::SlotDevice;
use crate::snapshot::{Reader, Result, SnapshotError, Writer};

/// Card ROM, assembled from [roms/RTVC-HOSTFILE.asm](../../roms/RTVC-HOSTFILE.asm).
pub const HOST_FILE_ROM: &[u8] = include_bytes!("../../roms/RTVC-HOSTFILE.ROM");

pub const PORT_COMMAND: u8 = 0x0;
pub const PORT_DATA: u8 = 0x1;
pub const PORT_NAME: u8 = 0x2;
pub const PORT_FILE_FLAGS: u8 = 0x3;
pub const PORT_PARAM: u8 = 0x4;
pub const PORT_CARD_FLAGS: u8 = 0x8;
pub const PORT_SIGNATURE: u8 = 0xF;

pub const CMD_NOP: u8 = 0x00;
pub const CMD_RESET: u8 = 0x01;
pub const CMD_NAME_CLEAR: u8 = 0x02;
pub const CMD_OPEN_IN: u8 = 0x03;
pub const CMD_OPEN_OUT: u8 = 0x04;
pub const CMD_CLOSE_IN: u8 = 0x05;
pub const CMD_CLOSE_OUT: u8 = 0x06;
pub const CMD_IN_SEEK_SET: u8 = 0x07;
pub const CMD_IN_SEEK_CUR: u8 = 0x08;
pub const CMD_IN_SEEK_END: u8 = 0x09;
pub const CMD_OUT_SEEK_SET: u8 = 0x0A;
pub const CMD_OUT_SEEK_CUR: u8 = 0x0B;
pub const CMD_OUT_SEEK_END: u8 = 0x0C;

pub const STATUS_OK: u8 = 0x00;
/// VT-DOS "File not found".
pub const STATUS_NOT_FOUND: u8 = 0xA1;
/// VT-DOS "Disk operation aborted", used for host store failures.
pub const STATUS_IO_ERROR: u8 = 0xAD;
/// TVC OS cassette "invalid CAS header" / read failure.
pub const STATUS_BAD_CAS: u8 = 0xE7;
pub const STATUS_NOT_OPEN: u8 = 0xE9;
pub const STATUS_EOF: u8 = 0xEC;
pub const STATUS_UNKNOWN_COMMAND: u8 = 0xFF;

pub const FILE_FLAG_INPUT_EXHAUSTED: u8 = 0x01;
pub const FILE_FLAG_INPUT_OPEN: u8 = 0x02;
pub const FILE_FLAG_OUTPUT_OPEN: u8 = 0x04;
pub const CARD_FLAG_DEFAULT_DEVICE: u8 = 0x01;
pub const SIGNATURE: u8 = b'H';

const NAME_LIMIT: usize = 64;
const SNAPSHOT_VERSION: u8 = 1;

/// Flat file namespace exposed to the card. Names are single path components.
pub trait HostFileStore: Send + Sync {
    fn list(&self) -> std::io::Result<Vec<String>>;
    fn read(&self, name: &str) -> std::io::Result<Vec<u8>>;
    fn write(&self, name: &str, data: &[u8]) -> std::io::Result<()>;
}

fn is_plain_name(name: &str) -> bool {
    let mut components = Path::new(name).components();
    matches!(components.next(), Some(Component::Normal(_)))
        && components.next().is_none()
        && !name.contains(['/', '\\', ':'])
}

fn invalid_name(name: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("invalid host file name {name:?}"),
    )
}

/// Native backend: one host directory, created on first write.
pub struct DirStore {
    root: PathBuf,
}

impl DirStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn path(&self, name: &str) -> std::io::Result<PathBuf> {
        if !is_plain_name(name) {
            return Err(invalid_name(name));
        }
        Ok(self.root.join(name))
    }
}

impl HostFileStore for DirStore {
    fn list(&self) -> std::io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    fn read(&self, name: &str) -> std::io::Result<Vec<u8>> {
        std::fs::read(self.path(name)?)
    }

    fn write(&self, name: &str, data: &[u8]) -> std::io::Result<()> {
        let path = self.path(name)?;
        std::fs::create_dir_all(&self.root)?;
        std::fs::write(path, data)
    }
}

/// In-memory backend for the browser. Writes made by the card are queued so
/// the UI can mirror them into persistent browser storage.
#[derive(Default)]
pub struct MemoryStore {
    files: Mutex<BTreeMap<String, Vec<u8>>>,
    pending: Mutex<Vec<String>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or replaces a file without queueing it as a card write.
    pub fn insert(&self, name: &str, data: Vec<u8>) -> std::io::Result<()> {
        if !is_plain_name(name) {
            return Err(invalid_name(name));
        }
        self.files.lock().unwrap().insert(name.to_string(), data);
        Ok(())
    }

    pub fn remove(&self, name: &str) -> bool {
        self.files.lock().unwrap().remove(name).is_some()
    }

    pub fn get(&self, name: &str) -> Option<Vec<u8>> {
        self.files.lock().unwrap().get(name).cloned()
    }

    pub fn names(&self) -> Vec<String> {
        self.files.lock().unwrap().keys().cloned().collect()
    }

    /// Files written by the card since the last call, with their current data.
    pub fn take_pending(&self) -> Vec<(String, Vec<u8>)> {
        let names = std::mem::take(&mut *self.pending.lock().unwrap());
        let files = self.files.lock().unwrap();
        let mut out: Vec<(String, Vec<u8>)> = Vec::new();
        for name in names {
            if out.iter().any(|(existing, _)| *existing == name) {
                continue;
            }
            if let Some(data) = files.get(&name) {
                out.push((name, data.clone()));
            }
        }
        out
    }
}

impl HostFileStore for MemoryStore {
    fn list(&self) -> std::io::Result<Vec<String>> {
        Ok(self.names())
    }

    fn read(&self, name: &str) -> std::io::Result<Vec<u8>> {
        self.get(name)
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
    }

    fn write(&self, name: &str, data: &[u8]) -> std::io::Result<()> {
        self.insert(name, data.to_vec())?;
        self.pending.lock().unwrap().push(name.to_string());
        Ok(())
    }
}

/// Maps a TVC file name to a lowercase host name stem that is always a single
/// safe path component.
pub fn host_name(tvc_name: &[u8]) -> String {
    let trimmed = tvc_name
        .iter()
        .copied()
        .skip_while(|b| *b == b' ')
        .collect::<Vec<_>>();
    let end = trimmed
        .iter()
        .rposition(|b| *b != b' ')
        .map_or(0, |pos| pos + 1);
    let mut out = String::new();
    for &b in trimmed[..end].iter().take(NAME_LIMIT) {
        let c = match b {
            b'A'..=b'Z' => b.to_ascii_lowercase() as char,
            b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => b as char,
            b'.' if !out.is_empty() => '.',
            _ => '_',
        };
        out.push(c);
    }
    if out.is_empty() {
        out.push_str("start");
    }
    out
}

fn has_extension(name: &str) -> bool {
    name.rfind('.')
        .is_some_and(|pos| pos > 0 && pos + 1 < name.len())
}

fn is_cas_name(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".cas")
}

/// Host names tried when opening `tvc_name` for input, in order.
pub fn input_candidates(tvc_name: &[u8]) -> Vec<String> {
    let name = host_name(tvc_name);
    let mut candidates = Vec::new();
    if !is_cas_name(&name) {
        candidates.push(format!("{name}.cas"));
    }
    candidates.push(name);
    candidates
}

/// Host name used when creating `tvc_name`; names without an extension get `.cas`.
pub fn output_name(tvc_name: &[u8]) -> String {
    let name = host_name(tvc_name);
    if has_extension(&name) {
        name
    } else {
        format!("{name}.cas")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OpenFile {
    host_name: String,
    cas: bool,
    file_type: u8,
    data: Vec<u8>,
    pos: usize,
}

impl OpenFile {
    fn write_snapshot(file: &Option<OpenFile>, w: &mut Writer) {
        let Some(file) = file else {
            w.u8(0);
            return;
        };
        w.u8(1);
        w.string(&file.host_name);
        w.u8(file.cas as u8);
        w.u8(file.file_type);
        w.bytes(&file.data);
        w.u32(file.pos as u32);
    }

    fn read_snapshot(r: &mut Reader<'_>) -> Result<Option<OpenFile>> {
        if r.u8()? == 0 {
            return Ok(None);
        }
        let host_name = r.string()?;
        let cas = r.u8()? != 0;
        let file_type = r.u8()?;
        let data = r.bytes()?.to_vec();
        let pos = r.u32()? as usize;
        if pos > data.len() {
            return Err(SnapshotError::InvalidData(
                "host file position beyond end".to_string(),
            ));
        }
        Ok(Some(OpenFile {
            host_name,
            cas,
            file_type,
            data,
            pos,
        }))
    }

    fn seek(&mut self, whence: u8, offset: u32) -> std::result::Result<(), u8> {
        let base = match whence {
            0 => 0,
            1 => self.pos as i64,
            _ => self.data.len() as i64,
        };
        let offset = if whence == 0 {
            offset as i64
        } else {
            offset as i32 as i64
        };
        let target = base + offset;
        if target < 0 || target > self.data.len() as i64 {
            return Err(STATUS_EOF);
        }
        self.pos = target as usize;
        Ok(())
    }
}

pub struct HostFileCard {
    store: Option<Arc<dyn HostFileStore>>,
    default_device: bool,
    name: Vec<u8>,
    status: u8,
    param: [u8; 4],
    input: Option<OpenFile>,
    output: Option<OpenFile>,
}

impl HostFileCard {
    pub fn new(store: Option<Arc<dyn HostFileStore>>, default_device: bool) -> Self {
        Self {
            store,
            default_device,
            name: Vec::new(),
            status: STATUS_OK,
            param: [0; 4],
            input: None,
            output: None,
        }
    }

    pub fn set_store(&mut self, store: Option<Arc<dyn HostFileStore>>) {
        self.store = store;
    }

    pub fn set_default_device(&mut self, default_device: bool) {
        self.default_device = default_device;
    }

    pub fn default_device(&self) -> bool {
        self.default_device
    }

    pub fn status(&self) -> u8 {
        self.status
    }

    fn param(&self) -> u32 {
        u32::from_le_bytes(self.param)
    }

    fn set_param(&mut self, value: u32) {
        self.param = value.to_le_bytes();
    }

    fn file_flags(&self) -> u8 {
        let mut flags = 0;
        if let Some(input) = &self.input {
            flags |= FILE_FLAG_INPUT_OPEN;
            if input.pos >= input.data.len() {
                flags |= FILE_FLAG_INPUT_EXHAUSTED;
            }
        }
        if self.output.is_some() {
            flags |= FILE_FLAG_OUTPUT_OPEN;
        }
        flags
    }

    fn command(&mut self, cmd: u8) {
        self.status = match cmd {
            CMD_NOP => STATUS_OK,
            CMD_RESET => {
                self.input = None;
                self.output = None;
                self.name.clear();
                STATUS_OK
            }
            CMD_NAME_CLEAR => {
                self.name.clear();
                STATUS_OK
            }
            CMD_OPEN_IN => self.open_input(),
            CMD_OPEN_OUT => self.open_output(),
            CMD_CLOSE_IN => {
                self.input = None;
                STATUS_OK
            }
            CMD_CLOSE_OUT => self.close_output(),
            CMD_IN_SEEK_SET..=CMD_IN_SEEK_END => {
                let offset = self.param();
                match self.input.as_mut() {
                    Some(file) => match file.seek(cmd - CMD_IN_SEEK_SET, offset) {
                        Ok(()) => {
                            let pos = file.pos as u32;
                            self.set_param(pos);
                            STATUS_OK
                        }
                        Err(status) => status,
                    },
                    None => STATUS_NOT_OPEN,
                }
            }
            CMD_OUT_SEEK_SET..=CMD_OUT_SEEK_END => {
                let offset = self.param();
                match self.output.as_mut() {
                    Some(file) => match file.seek(cmd - CMD_OUT_SEEK_SET, offset) {
                        Ok(()) => {
                            let pos = file.pos as u32;
                            self.set_param(pos);
                            STATUS_OK
                        }
                        Err(status) => status,
                    },
                    None => STATUS_NOT_OPEN,
                }
            }
            _ => STATUS_UNKNOWN_COMMAND,
        };
    }

    fn resolve_input(&self, store: &dyn HostFileStore) -> Option<String> {
        let names = store.list().ok()?;
        input_candidates(&self.name)
            .into_iter()
            .find_map(|candidate| {
                names
                    .iter()
                    .find(|name| name.eq_ignore_ascii_case(&candidate))
                    .cloned()
            })
    }

    fn open_input(&mut self) -> u8 {
        self.input = None;
        let Some(store) = self.store.clone() else {
            return STATUS_NOT_FOUND;
        };
        let Some(host_name) = self.resolve_input(store.as_ref()) else {
            return STATUS_NOT_FOUND;
        };
        let raw = match store.read(&host_name) {
            Ok(raw) => raw,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return STATUS_NOT_FOUND,
            Err(_) => return STATUS_IO_ERROR,
        };
        let cas = is_cas_name(&host_name);
        let (file_type, data) = if cas {
            match decode_cas_container(&raw) {
                Some((file_type, stream)) => (file_type, stream.to_vec()),
                None => return STATUS_BAD_CAS,
            }
        } else {
            (TVC_CAS_TYPE_UNBUFFERED, raw)
        };
        self.set_param(file_type as u32);
        self.input = Some(OpenFile {
            host_name,
            cas,
            file_type,
            data,
            pos: 0,
        });
        STATUS_OK
    }

    fn open_output(&mut self) -> u8 {
        if self.output.is_some() {
            let status = self.close_output();
            if status != STATUS_OK {
                return status;
            }
        }
        if self.store.is_none() {
            return STATUS_IO_ERROR;
        }
        let file_type = match self.param[0] {
            TVC_CAS_TYPE_BUFFERED => TVC_CAS_TYPE_BUFFERED,
            _ => TVC_CAS_TYPE_UNBUFFERED,
        };
        let host_name = output_name(&self.name);
        self.output = Some(OpenFile {
            cas: is_cas_name(&host_name),
            host_name,
            file_type,
            data: Vec::new(),
            pos: 0,
        });
        STATUS_OK
    }

    fn close_output(&mut self) -> u8 {
        let Some(file) = self.output.take() else {
            return STATUS_OK;
        };
        let Some(store) = self.store.as_ref() else {
            return STATUS_IO_ERROR;
        };
        let bytes = if file.cas {
            let mut bytes = encode_cas_container(file.file_type, file.data.len()).to_vec();
            bytes.extend_from_slice(&file.data);
            bytes
        } else {
            file.data
        };
        match store.write(&file.host_name, &bytes) {
            Ok(()) => STATUS_OK,
            Err(_) => STATUS_IO_ERROR,
        }
    }

    fn read_data(&mut self) -> u8 {
        let Some(file) = self.input.as_mut() else {
            self.status = STATUS_NOT_OPEN;
            return 0xFF;
        };
        match file.data.get(file.pos) {
            Some(&byte) => {
                file.pos += 1;
                self.status = STATUS_OK;
                byte
            }
            None => {
                self.status = STATUS_EOF;
                0xFF
            }
        }
    }

    fn write_data(&mut self, val: u8) {
        let Some(file) = self.output.as_mut() else {
            self.status = STATUS_NOT_OPEN;
            return;
        };
        if file.pos < file.data.len() {
            file.data[file.pos] = val;
        } else {
            file.data.push(val);
        }
        file.pos += 1;
        self.status = STATUS_OK;
    }

    fn rom_byte(offset: u16) -> u8 {
        HOST_FILE_ROM.get(offset as usize).copied().unwrap_or(0xFF)
    }
}

impl SlotDevice for HostFileCard {
    fn id_bits(&self) -> u8 {
        3
    }

    fn mem_peek(&self, offset: u16) -> u8 {
        Self::rom_byte(offset)
    }

    fn mem_write(&mut self, _offset: u16, _val: u8) {}

    fn io_read(&mut self, port: u16) -> u8 {
        match port as u8 & 0x0F {
            PORT_COMMAND => self.status,
            PORT_DATA => self.read_data(),
            PORT_FILE_FLAGS => self.file_flags(),
            p @ PORT_PARAM..=0x7 => self.param[(p - PORT_PARAM) as usize],
            PORT_CARD_FLAGS => {
                if self.default_device {
                    CARD_FLAG_DEFAULT_DEVICE
                } else {
                    0
                }
            }
            PORT_SIGNATURE => SIGNATURE,
            _ => 0xFF,
        }
    }

    fn io_write(&mut self, port: u16, val: u8) {
        match port as u8 & 0x0F {
            PORT_COMMAND => self.command(val),
            PORT_DATA => self.write_data(val),
            PORT_NAME => {
                if self.name.len() < 255 {
                    self.name.push(val);
                }
            }
            p @ PORT_PARAM..=0x7 => self.param[(p - PORT_PARAM) as usize] = val,
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.command(CMD_RESET);
    }

    fn write_snapshot(&self, w: &mut Writer) {
        w.u8(SNAPSHOT_VERSION);
        w.bytes(&self.name);
        w.u8(self.status);
        w.raw_bytes(&self.param);
        OpenFile::write_snapshot(&self.input, w);
        OpenFile::write_snapshot(&self.output, w);
    }

    fn read_snapshot(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let version = r.u8()?;
        if version != SNAPSHOT_VERSION {
            return Err(SnapshotError::InvalidData(format!(
                "unsupported host file card snapshot version {version}"
            )));
        }
        self.name = r.bytes()?.to_vec();
        self.status = r.u8()?;
        self.param.copy_from_slice(r.raw_bytes(4)?);
        self.input = OpenFile::read_snapshot(r)?;
        self.output = OpenFile::read_snapshot(r)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "host_file_tests.rs"]
mod tests;
