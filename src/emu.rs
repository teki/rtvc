use crate::tvc::Tvc;

pub struct Emu {
    pub tvc: Tvc,
    pub running: bool,
    pub roms_loaded: bool,
}

impl Emu {
    pub fn new() -> Self {
        Emu {
            tvc: Tvc::new(false),
            running: true,
            roms_loaded: false,
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

    pub fn load_roms(&mut self) {
        std::fs::create_dir_all("roms").ok();
        let rom_files = [
            "TVC12_D3.64K",
            "TVC12_D4.64K",
            "TVC12_D7.64K",
        ];
        let mut any_loaded = false;
        for name in &rom_files {
            let path = format!("roms/{}", name);
            match std::fs::read(&path) {
                Ok(data) => {
                    self.tvc.add_rom(name, &data);
                    any_loaded = true;
                }
                Err(_) => {
                    // ROM not found, skip
                }
            }
        }
        if any_loaded {
            self.roms_loaded = true;
        }
    }
}
