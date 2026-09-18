use super::*;
use crate::basic::tokenize_program;

fn cas_with_payload(payload_size: usize) -> Vec<u8> {
    let dfsize = 144 + payload_size;
    let blocks = dfsize / 128;
    let remainder = dfsize % 128;
    let mut data = vec![0; dfsize];
    data[0] = 0x11;
    data[2] = (blocks & 0xFF) as u8;
    data[3] = (blocks >> 8) as u8;
    data[4] = (remainder & 0xFF) as u8;
    data[5] = (remainder >> 8) as u8;
    data[0x80] = 0x00;
    data[0x83] = 0x00;
    for (i, byte) in data[144..].iter_mut().enumerate() {
        *byte = i as u8;
    }
    data
}

#[test]
fn exact_sector_sized_payload_does_not_overrun() {
    let generator = TapeBitstreamGenerator::new(&cas_with_payload(256), "TEST");

    assert!(generator.is_ok());
}

fn stub() -> Vec<u8> {
    tokenize_program("10 PRINT 1\n").unwrap()
}

#[test]
fn flatten_preserves_out_of_order_bytes_and_zero_gaps() {
    let stub = stub();
    let code = [0xC9u8];
    let code_addr = 0x1C00u16;
    assert!(stub.len() < (code_addr - TVC_CAS_LOAD_ADDR) as usize);
    let image = flatten_cas_image(&[
        (code_addr, code.as_slice()),
        (TVC_CAS_LOAD_ADDR, stub.as_slice()),
    ])
    .unwrap();
    assert_eq!(image.load_addr, TVC_CAS_LOAD_ADDR);
    assert_eq!(image.bytes[0], stub[0]);
    assert_eq!(image.bytes[stub.len() - 1], 0);
    let gap_start = stub.len();
    let code_off = (code_addr - TVC_CAS_LOAD_ADDR) as usize;
    assert!(image.bytes[gap_start..code_off].iter().all(|b| *b == 0));
    assert_eq!(image.bytes[code_off], 0xC9);
    assert_eq!(image.payload_bytes, stub.len() + 1);
    assert_eq!(image.padding_bytes, code_off - stub.len());
}

#[test]
fn flatten_basic_only_image_has_no_padding() {
    let stub = stub();
    let image = flatten_cas_image(&[(TVC_CAS_LOAD_ADDR, stub.as_slice())]).unwrap();
    assert_eq!(image.bytes, stub);
    assert_eq!(image.padding_bytes, 0);
    assert_eq!(image.payload_bytes, stub.len());
}

#[test]
fn cas_autostart_follows_header_byte() {
    let on = encode_tvc_cas(&[0x00], TVC_CAS_TYPE_BASIC, 0xFF, TVC_CAS_LOAD_ADDR);
    let off = encode_tvc_cas(&[0x00], TVC_CAS_TYPE_BASIC, 0x00, TVC_CAS_LOAD_ADDR);
    assert!(tvc_cas_autostarts(&on));
    assert!(!tvc_cas_autostarts(&off));
    assert!(!tvc_cas_autostarts(&[0x11]));
}

#[test]
fn flatten_allows_adjacent_segments() {
    let stub = stub();
    let next = TVC_CAS_LOAD_ADDR + stub.len() as u16;
    let image =
        flatten_cas_image(&[(TVC_CAS_LOAD_ADDR, stub.as_slice()), (next, &[0xC9])]).unwrap();
    assert_eq!(image.padding_bytes, 0);
    assert_eq!(*image.bytes.last().unwrap(), 0xC9);
}

#[test]
fn flatten_rejects_overlap_relocated_basic_and_overflow() {
    let stub = stub();
    let overlap = flatten_cas_image(&[
        (TVC_CAS_LOAD_ADDR, stub.as_slice()),
        (TVC_CAS_LOAD_ADDR, &[0xC9]),
    ])
    .unwrap_err();
    assert!(overlap.contains("overlap"), "{overlap}");
    let relocated = flatten_cas_image(&[(0x4000, stub.as_slice())]).unwrap_err();
    assert!(relocated.contains("4000"), "{relocated}");
    assert!(relocated.contains("19EF"), "{relocated}");
    let overflow =
        flatten_cas_image(&[(TVC_CAS_LOAD_ADDR, stub.as_slice()), (0xBFFF, &[0, 0])]).unwrap_err();
    assert!(overflow.contains("C000"), "{overflow}");
    let empty = flatten_cas_image(&[]).unwrap_err();
    assert!(empty.contains("at least one"), "{empty}");
    let empty_prog = flatten_cas_image(&[(TVC_CAS_LOAD_ADDR, &[0])]).unwrap_err();
    assert!(empty_prog.contains("empty"), "{empty_prog}");
    let no_basic = flatten_cas_image(&[(TVC_CAS_LOAD_ADDR, &[0xC9, 0xC9])]).unwrap_err();
    assert!(no_basic.contains("BASIC"), "{no_basic}");
}
