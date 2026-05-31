use crate::cas::TapeBitstreamGenerator;

pub(crate) struct TapeInterface {
    generator: Option<TapeBitstreamGenerator>,
    play_active: bool,
    start_cycle: u64,
    cycles: u64,
    motor_on: bool,
    output_flip_flop: bool,
}

impl TapeInterface {
    pub(crate) fn new() -> Self {
        Self {
            generator: None,
            play_active: false,
            start_cycle: 0,
            cycles: 0,
            motor_on: false,
            output_flip_flop: false,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.play_active = false;
        self.cycles = 0;
        self.start_cycle = 0;
        self.motor_on = false;
        self.output_flip_flop = false;
    }

    pub(crate) fn set_cycles(&mut self, cycles: u64) {
        self.cycles = cycles;
    }

    pub(crate) fn advance(&mut self, cycles: u64) {
        self.cycles += cycles;
    }

    pub(crate) fn set_motor_from_port5(&mut self, val: u8) {
        self.motor_on = (val & 0xC0) != 0;
    }

    pub(crate) fn toggle_output(&mut self) {
        self.output_flip_flop = !self.output_flip_flop;
    }

    pub(crate) fn play(&mut self, generator: TapeBitstreamGenerator) {
        self.generator = Some(generator);
        self.play_active = true;
        self.start_cycle = self.cycles;
    }

    pub(crate) fn stop(&mut self) {
        self.play_active = false;
    }

    pub(crate) fn is_active(&self) -> bool {
        self.play_active
    }

    pub(crate) fn motor_on(&self) -> bool {
        self.motor_on
    }

    pub(crate) fn cycles(&self) -> u64 {
        self.cycles
    }

    pub(crate) fn state(&self) -> (u64, f32, u8) {
        let elapsed = self.cycles - self.start_cycle;
        let level = if self.play_active {
            self.generator
                .as_ref()
                .map(|generator| generator.get_signal_at_cycle(elapsed))
                .unwrap_or(0.5)
        } else {
            0.5
        };
        let bit = if self.motor_on && self.play_active && level > 0.5 {
            1
        } else {
            0
        };
        (elapsed, level, bit)
    }

    pub(crate) fn input_bit(&mut self) -> u8 {
        let (_, _, mut tape_bit) = self.state();
        if !self.motor_on || !self.play_active {
            tape_bit = 0;
        } else if let Some(ref generator) = self.generator {
            let elapsed = self.cycles - self.start_cycle;
            if elapsed >= generator.total_cycles {
                self.play_active = false;
                tape_bit = 0;
            }
        }
        tape_bit
    }

    pub(crate) fn current_level(&self) -> f32 {
        self.state().1
    }
}
