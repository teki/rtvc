pub(crate) struct SoundTimer {
    pub(crate) freq_low: u8,
    pub(crate) ctrl: u8,
    period_cycles: Option<u64>,
    counter: u64,
    running: bool,
}

impl SoundTimer {
    pub(crate) fn new() -> Self {
        Self {
            freq_low: 0,
            ctrl: 0,
            period_cycles: Some(0x1000 * 16),
            counter: 0,
            running: false,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.freq_low = 0;
        self.ctrl = 0;
        self.period_cycles = Some(0x1000 * 16);
        self.counter = 0;
        self.running = false;
    }

    pub(crate) fn write_low(&mut self, val: u8) {
        self.freq_low = val;
        self.update_period_cycles();
    }

    pub(crate) fn write_control(&mut self, val: u8) {
        self.ctrl = val;
        self.update_period_cycles();
    }

    pub(crate) fn divisor(&self) -> u16 {
        ((self.ctrl as u16 & 0x0F) << 8) | self.freq_low as u16
    }

    fn update_period_cycles(&mut self) {
        let divisor = self.divisor();
        self.period_cycles = if divisor == 0x0FFF {
            None
        } else {
            Some((0x1000u64 - divisor as u64) * 16)
        };
    }

    pub(crate) fn interrupt_enabled(&self) -> bool {
        (self.ctrl & 0x20) != 0
    }

    pub(crate) fn period_cycles(&self) -> Option<u64> {
        self.period_cycles
    }

    pub(crate) fn counter(&self) -> u64 {
        self.counter
    }

    pub(crate) fn running(&self) -> bool {
        self.running
    }

    pub(crate) fn restart(&mut self) {
        self.counter = self.period_cycles.unwrap_or(0);
        self.running = self.counter != 0;
    }

    pub(crate) fn advance(&mut self, cycles: u64) -> bool {
        if !self.running {
            return false;
        }

        let Some(period) = self.period_cycles else {
            self.running = false;
            self.counter = 0;
            return false;
        };

        let mut remaining = cycles;
        let mut fired = false;
        while remaining >= self.counter {
            remaining -= self.counter;
            self.counter = period;
            fired |= self.interrupt_enabled();
        }
        self.counter -= remaining;
        fired
    }

    pub(crate) fn write_snapshot(&self, w: &mut crate::snapshot::Writer) {
        w.u8(self.freq_low);
        w.u8(self.ctrl);
        w.u64(self.counter);
        w.u8(self.running as u8);
    }

    pub(crate) fn read_snapshot(
        &mut self,
        r: &mut crate::snapshot::Reader<'_>,
    ) -> crate::snapshot::Result<()> {
        self.freq_low = r.u8()?;
        self.ctrl = r.u8()?;
        self.update_period_cycles();
        self.counter = r.u64()?;
        self.running = r.u8()? != 0;
        Ok(())
    }
}
