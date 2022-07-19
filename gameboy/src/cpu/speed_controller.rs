use crate::bus::CgbCompatibility;

#[derive(Clone, Copy)]
enum CpuSpeedMode {
    Single = 0,
    Double = 1,
}

pub struct CpuSpeedController {
    cpu_speed_mode: CpuSpeedMode,
    prepare_speed_switch: bool,
    cgb_compatibility: CgbCompatibility,
}

impl CpuSpeedController {
    pub fn new(cgb_compatibility: CgbCompatibility) -> Self {
        Self {
            cpu_speed_mode: CpuSpeedMode::Single,
            prepare_speed_switch: false,
            cgb_compatibility,
        }
    }

    pub fn is_double_speed(&self) -> bool {
        matches!(self.cpu_speed_mode, CpuSpeedMode::Double)
    }

    pub fn set_console_compatibility(&mut self, cgb_compatibility: CgbCompatibility) {
        self.cgb_compatibility = cgb_compatibility;
    }

    pub fn set_prepare_speed_switch(&mut self, set: bool) {
        if set {
            log::debug!("CPU SPEED SWITCH PREPARED!!");
        } else {
            log::debug!("Blank speed switch write?");
        }
        self.prepare_speed_switch = set;
    }

    pub fn is_speed_switch_prepared(&self) -> bool {
        self.prepare_speed_switch
    }

    pub fn switch_speed(&mut self) {
        debug_assert!(self.prepare_speed_switch);
        self.prepare_speed_switch = false;
        self.cpu_speed_mode = match self.cpu_speed_mode {
            CpuSpeedMode::Single => CpuSpeedMode::Double,
            CpuSpeedMode::Double => CpuSpeedMode::Single,
        };

        log::debug!("CPU SPEED SWITCHED");
    }

    pub fn read_key1(&self) -> u8 {
        let key1 =
            ((self.cpu_speed_mode as u8) << 7) | (self.prepare_speed_switch as u8) | 0b0111_1110;
        log::debug!("key1: {:#010b}", key1);
        key1
    }
}
