use crate::hbf::HBF;
use crate::snapshot::{self, Reader, SnapshotError, Writer};
use crate::tvc::Tvc;
use crate::vid::VidModel;

pub(crate) fn save(tvc: &Tvc) -> Vec<u8> {
    let mut chunks = Vec::new();

    let mut meta = Writer::new();
    meta.u8(tvc.bus.mmu.is_plus() as u8);
    meta.u8(match tvc.vid_model {
        VidModel::FastFrame => 0,
        VidModel::Interleaved => 1,
        VidModel::Line => 2,
    });
    meta.u64(tvc.clock);
    meta.u8(tvc.frame_complete as u8);
    chunks.push((*b"META", meta.into_inner()));

    let mut cpu = Writer::new();
    cpu.raw_bytes(&tvc.z80.state.r8);
    for reg in tvc.z80.state.r16 {
        cpu.u16(reg);
    }
    cpu.u8(tvc.z80.state.halted);
    cpu.u8(tvc.z80.state.im);
    cpu.u8(tvc.z80.state.iff1);
    cpu.u8(tvc.z80.state.iff2);
    chunks.push((*b"CPUZ", cpu.into_inner()));

    let mut mmu = Writer::new();
    tvc.bus.mmu.write_snapshot(&mut mmu);
    chunks.push((*b"MMU ", mmu.into_inner()));

    let mut vid = Writer::new();
    tvc.bus.vid.write_snapshot(&mut vid);
    chunks.push((*b"VID ", vid.into_inner()));

    if let Some(ext) = tvc.bus.extensions.slot0() {
        let mut hbf = Writer::new();
        ext.write_snapshot(&mut hbf);
        chunks.push((*b"HBF ", hbf.into_inner()));
    }

    let mut bus = Writer::new();
    bus.u8(tvc.bus.pend_it);
    bus.u8(tvc.bus.extensions.type_status());
    bus.u8(tvc.bus.extensions.selected_mapping());
    chunks.push((*b"BUS ", bus.into_inner()));

    snapshot::write_file(&chunks)
}

pub(crate) fn load(tvc: &mut Tvc, data: &[u8]) -> snapshot::Result<()> {
    let chunks = snapshot::read_file(data)?;
    let meta = chunks
        .iter()
        .find(|chunk| chunk.id == *b"META")
        .ok_or(SnapshotError::InvalidChunk("META"))?;
    let mut meta = Reader::new(meta.data);
    let is_plus = meta.u8()? != 0;
    let vid_model = match meta.u8()? {
        0 => VidModel::FastFrame,
        1 => VidModel::Interleaved,
        2 => VidModel::Line,
        _ => {
            return Err(SnapshotError::InvalidData(
                "unknown video model".to_string(),
            ));
        }
    };
    let clock = meta.u64()?;
    let frame_complete = meta.u8()? != 0;

    *tvc = Tvc::new_with_vid_model(is_plus, vid_model);
    tvc.clock = clock;
    tvc.bus.set_tape_cycles(clock);
    tvc.frame_complete = frame_complete;

    for chunk in chunks {
        let mut reader = Reader::new(chunk.data);
        match &chunk.id {
            b"META" => {}
            b"CPUZ" => {
                tvc.z80.state.r8.copy_from_slice(reader.raw_bytes(22)?);
                for reg in &mut tvc.z80.state.r16 {
                    *reg = reader.u16()?;
                }
                tvc.z80.state.halted = reader.u8()?;
                tvc.z80.state.im = reader.u8()?;
                tvc.z80.state.iff1 = reader.u8()?;
                tvc.z80.state.iff2 = reader.u8()?;
            }
            b"MMU " => tvc.bus.mmu.read_snapshot(&mut reader)?,
            b"VID " => tvc.bus.vid.read_snapshot(&mut reader)?,
            b"HBF " => {
                tvc.bus
                    .extensions
                    .replace_slot0(HBF::read_snapshot(&mut reader)?);
            }
            b"BUS " => {
                tvc.bus.pend_it = reader.u8()?;
                tvc.bus.extensions.set_type_status(reader.u8()?);
                tvc.bus.extensions.set_selected_mapping(reader.u8()?);
            }
            _ => {}
        }
    }
    tvc.bus.key.reset();
    tvc.bus.log.clear();
    Ok(())
}
