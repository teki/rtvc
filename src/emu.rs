use crate::tvc::Tvc;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RomVersion {
    V1_2,
    V2_2,
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
            MachineType { is_plus: true, rom_version: RomVersion::V1_2, has_dos: true },
            MachineType { is_plus: true, rom_version: RomVersion::V2_2, has_dos: true },
            MachineType { is_plus: false, rom_version: RomVersion::V1_2, has_dos: false },
            MachineType { is_plus: true, rom_version: RomVersion::V1_2, has_dos: false },
            MachineType { is_plus: true, rom_version: RomVersion::V2_2, has_dos: false },
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

pub struct Emu {
    pub tvc: Tvc,
    pub running: bool,
    pub roms_loaded: bool,
    pub machine_type: MachineType,
}

impl Emu {
    pub fn new(machine_type: MachineType) -> Self {
        Emu {
            tvc: Tvc::new(machine_type.is_plus),
            running: true,
            roms_loaded: false,
            machine_type,
        }
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

    pub fn toggle_running(&mut self) {
        self.running = !self.running;
    }

    pub fn reload(&mut self, machine_type: MachineType) {
        self.machine_type = machine_type;
        self.tvc = Tvc::new(machine_type.is_plus);
        self.roms_loaded = false;
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
}
