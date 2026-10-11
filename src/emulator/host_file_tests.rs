use super::*;
use crate::cas::{TVC_CAS_CONTAINER_LEN, TVC_CAS_LOAD_ADDR, TVC_CAS_TYPE_BASIC, encode_tvc_cas};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rtvc-hostfile-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn card_with(store: Arc<dyn HostFileStore>) -> HostFileCard {
    HostFileCard::new(Some(store), true)
}

fn set_name(card: &mut HostFileCard, name: &str) {
    card.io_write(0x10 | PORT_COMMAND as u16, CMD_NAME_CLEAR);
    for byte in name.bytes() {
        card.io_write(0x10 | PORT_NAME as u16, byte);
    }
}

fn command(card: &mut HostFileCard, cmd: u8) -> u8 {
    card.io_write(PORT_COMMAND as u16, cmd);
    card.io_read(PORT_COMMAND as u16)
}

fn read_all(card: &mut HostFileCard) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let byte = card.io_read(PORT_DATA as u16);
        if card.status() != STATUS_OK {
            return out;
        }
        out.push(byte);
    }
}

fn param(card: &mut HostFileCard) -> u32 {
    u32::from_le_bytes(std::array::from_fn(|i| {
        card.io_read((PORT_PARAM + i as u8) as u16)
    }))
}

fn set_param(card: &mut HostFileCard, value: u32) {
    for (i, byte) in value.to_le_bytes().into_iter().enumerate() {
        card.io_write((PORT_PARAM + i as u8) as u16, byte);
    }
}

#[test]
fn host_names_are_lowercase_single_components() {
    assert_eq!(host_name(b"GAME"), "game");
    assert_eq!(host_name(b"  My Prog  "), "my_prog");
    assert_eq!(host_name(b"../../etc/passwd"), "_._.._etc_passwd");
    assert_eq!(host_name(b"C:\\X.BAS"), "c__x.bas");
    assert_eq!(host_name(b""), "start");
    assert_eq!(host_name(&[0xC1, b'A']), "_a");
    assert_eq!(input_candidates(b"Game"), vec!["game.cas", "game"]);
    assert_eq!(input_candidates(b"GAME.CAS"), vec!["game.cas"]);
    assert_eq!(output_name(b"GAME"), "game.cas");
    assert_eq!(output_name(b"notes.txt"), "notes.txt");
    for name in [&b"../x"[..], b"/abs", b"a/b", b"..", b"."] {
        assert!(is_plain_name(&output_name(name)), "{name:?}");
    }
}

#[test]
fn dir_store_rejects_paths_outside_root() {
    let dir = temp_dir("escape");
    let store = DirStore::new(&dir);
    assert!(store.write("../outside.cas", b"x").is_err());
    assert!(store.write("sub/inner.cas", b"x").is_err());
    assert!(store.read("/etc/hosts").is_err());
    assert!(store.write("ok.cas", b"x").is_ok());
    assert_eq!(store.list().unwrap(), vec!["ok.cas"]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn open_missing_file_reports_not_found() {
    let mut card = card_with(Arc::new(MemoryStore::new()));
    set_name(&mut card, "NOPE");
    assert_eq!(command(&mut card, CMD_OPEN_IN), STATUS_NOT_FOUND);
    assert_eq!(card.io_read(PORT_FILE_FLAGS as u16), 0);
    assert_eq!(card.io_read(PORT_DATA as u16), 0xFF);
    assert_eq!(card.status(), STATUS_NOT_OPEN);
}

#[test]
fn cas_file_is_found_case_insensitively_and_stripped() {
    let store = Arc::new(MemoryStore::new());
    let cas = encode_tvc_cas(b"\x01\x02\x03", TVC_CAS_TYPE_BASIC, 0xFF, TVC_CAS_LOAD_ADDR);
    store.insert("Game.CAS", cas.clone()).unwrap();
    let mut card = card_with(store);
    set_name(&mut card, "GAME");
    assert_eq!(command(&mut card, CMD_OPEN_IN), STATUS_OK);
    assert_eq!(param(&mut card) & 0xFF, TVC_CAS_TYPE_UNBUFFERED as u32);
    assert_eq!(card.io_read(PORT_FILE_FLAGS as u16), FILE_FLAG_INPUT_OPEN);
    let stream = read_all(&mut card);
    assert_eq!(stream, cas[TVC_CAS_CONTAINER_LEN..]);
    assert_eq!(card.status(), STATUS_EOF);
    assert_eq!(
        card.io_read(PORT_FILE_FLAGS as u16),
        FILE_FLAG_INPUT_OPEN | FILE_FLAG_INPUT_EXHAUSTED
    );
}

#[test]
fn exact_name_is_used_when_no_cas_exists() {
    let store = Arc::new(MemoryStore::new());
    store.insert("data", b"raw".to_vec()).unwrap();
    let mut card = card_with(store);
    set_name(&mut card, "DATA");
    assert_eq!(command(&mut card, CMD_OPEN_IN), STATUS_OK);
    assert_eq!(read_all(&mut card), b"raw");
}

#[test]
fn invalid_cas_container_is_rejected() {
    let store = Arc::new(MemoryStore::new());
    store.insert("bad.cas", vec![0x55; 200]).unwrap();
    let mut card = card_with(store);
    set_name(&mut card, "BAD");
    assert_eq!(command(&mut card, CMD_OPEN_IN), STATUS_BAD_CAS);
}

#[test]
fn input_seek_reports_new_position() {
    let store = Arc::new(MemoryStore::new());
    store.insert("data", (0..10).collect()).unwrap();
    let mut card = card_with(store);
    set_name(&mut card, "data");
    command(&mut card, CMD_OPEN_IN);

    set_param(&mut card, 4);
    assert_eq!(command(&mut card, CMD_IN_SEEK_SET), STATUS_OK);
    assert_eq!(param(&mut card), 4);
    assert_eq!(card.io_read(PORT_DATA as u16), 4);

    set_param(&mut card, (-2i32) as u32);
    assert_eq!(command(&mut card, CMD_IN_SEEK_CUR), STATUS_OK);
    assert_eq!(param(&mut card), 3);

    set_param(&mut card, 0);
    assert_eq!(command(&mut card, CMD_IN_SEEK_END), STATUS_OK);
    assert_eq!(param(&mut card), 10);

    set_param(&mut card, 11);
    assert_eq!(command(&mut card, CMD_IN_SEEK_SET), STATUS_EOF);
}

#[test]
fn written_file_is_committed_as_valid_cas_on_close() {
    let dir = temp_dir("write");
    let store: Arc<dyn HostFileStore> = Arc::new(DirStore::new(&dir));
    let mut card = card_with(store.clone());
    let stream = [
        0x00, 0x01, 0x03, 0x00, 0xFF, 0, 0, 0xEF, 0x19, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3,
    ];
    set_name(&mut card, "Saved");
    set_param(&mut card, TVC_CAS_TYPE_UNBUFFERED as u32);
    assert_eq!(command(&mut card, CMD_OPEN_OUT), STATUS_OK);
    for byte in stream {
        card.io_write(PORT_DATA as u16, byte);
        assert_eq!(card.status(), STATUS_OK);
    }
    assert!(store.list().unwrap().is_empty());
    assert_eq!(command(&mut card, CMD_CLOSE_OUT), STATUS_OK);

    let written = std::fs::read(dir.join("saved.cas")).unwrap();
    let mut expected = encode_cas_container(TVC_CAS_TYPE_UNBUFFERED, stream.len()).to_vec();
    expected.extend_from_slice(&stream);
    assert_eq!(written, expected);
    let (file_type, decoded) = decode_cas_container(&written).unwrap();
    assert_eq!(file_type, TVC_CAS_TYPE_UNBUFFERED);
    assert_eq!(decoded, stream);

    set_name(&mut card, "SAVED");
    assert_eq!(command(&mut card, CMD_OPEN_IN), STATUS_OK);
    assert_eq!(read_all(&mut card), stream);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn buffered_output_and_raw_extension_files() {
    let store = Arc::new(MemoryStore::new());
    let mut card = card_with(store.clone());
    set_name(&mut card, "LOG");
    set_param(&mut card, TVC_CAS_TYPE_BUFFERED as u32);
    command(&mut card, CMD_OPEN_OUT);
    card.io_write(PORT_DATA as u16, b'x');
    command(&mut card, CMD_CLOSE_OUT);
    assert_eq!(store.get("log.cas").unwrap()[0], TVC_CAS_TYPE_BUFFERED);

    set_name(&mut card, "notes.txt");
    command(&mut card, CMD_OPEN_OUT);
    card.io_write(PORT_DATA as u16, b'a');
    card.io_write(PORT_DATA as u16, b'b');
    set_param(&mut card, 0);
    assert_eq!(command(&mut card, CMD_OUT_SEEK_SET), STATUS_OK);
    card.io_write(PORT_DATA as u16, b'c');
    command(&mut card, CMD_CLOSE_OUT);
    assert_eq!(store.get("notes.txt").unwrap(), b"cb");
    let pending: Vec<String> = store
        .take_pending()
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(pending, vec!["log.cas", "notes.txt"]);
}

#[test]
fn data_access_without_open_file_reports_not_open() {
    let mut card = card_with(Arc::new(MemoryStore::new()));
    card.io_write(PORT_DATA as u16, 1);
    assert_eq!(card.status(), STATUS_NOT_OPEN);
    assert_eq!(command(&mut card, CMD_CLOSE_IN), STATUS_OK);
    assert_eq!(command(&mut card, CMD_CLOSE_OUT), STATUS_OK);
    assert_eq!(command(&mut card, 0x7F), STATUS_UNKNOWN_COMMAND);
    assert_eq!(command(&mut card, CMD_NOP), STATUS_OK);
}

#[test]
fn card_reports_mops_rom_flags_and_signature() {
    let mut card = HostFileCard::new(None, false);
    assert_eq!(card.id_bits(), 3);
    assert_eq!(&HOST_FILE_ROM[..9], b"MOPS\x04FILE");
    assert_eq!(card.mem_peek(0), b'M');
    assert_eq!(card.mem_peek(0x1FFF), 0xFF);
    assert_eq!(card.io_read(0x28), 0);
    card.set_default_device(true);
    assert_eq!(card.io_read(0x28), CARD_FLAG_DEFAULT_DEVICE);
    assert_eq!(card.io_read(0x2F), SIGNATURE);
}

#[test]
fn snapshot_round_trips_open_files() {
    let store = Arc::new(MemoryStore::new());
    store.insert("data", (0..5).collect()).unwrap();
    let mut card = card_with(store.clone());
    set_name(&mut card, "data");
    command(&mut card, CMD_OPEN_IN);
    card.io_read(PORT_DATA as u16);
    set_name(&mut card, "out");
    command(&mut card, CMD_OPEN_OUT);
    card.io_write(PORT_DATA as u16, 9);

    let mut w = Writer::new();
    card.write_snapshot(&mut w);
    let bytes = w.into_inner();
    let mut restored = card_with(store);
    restored.read_snapshot(&mut Reader::new(&bytes)).unwrap();
    assert_eq!(restored.input, card.input);
    assert_eq!(restored.output, card.output);
    assert_eq!(restored.name, card.name);
    assert_eq!(restored.io_read(PORT_DATA as u16), 1);
}

#[test]
fn rom_binary_matches_its_source() {
    let source = std::fs::read_to_string("roms/RTVC-HOSTFILE.asm").unwrap();
    let program = crate::asm::assemble_program(&source, 0xC000).unwrap();
    assert_eq!(program.bytes, HOST_FILE_ROM);
}
