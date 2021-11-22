use super::{cartridge::Cartridge, input::Input, interrupts::Interrupts, ppu::Ppu, timer::Timer};

pub struct Bus {
    blargg_output_buffer: Vec<char>,
    pub cartridge: Box<dyn Cartridge>,
    pub ppu: Ppu,
    pub working_ram: [u8; 0x2000],
    pub io: [u8; 0x100],
    pub zero_page: [u8; 0x80],

    pub bios_enabled: bool,
    pub bios: [u8; 0x100],

    pub interrupts: Interrupts,
    pub timer: Timer,
    pub input: Input,
}

impl Bus {
    pub fn new(cartridge: Box<dyn Cartridge>) -> Self {
        Self {
            blargg_output_buffer: Vec::new(),
            cartridge,
            ppu: Ppu::new(),
            working_ram: [0; 0x2000],
            io: [0; 0x100],
            zero_page: [0; 0x80],

            bios_enabled: false,
            bios: [
                0x31, 0xFE, 0xFF, 0x21, 0xFF, 0x9F, 0xAF, 0x32, 0xCB, 0x7C, 0x20, 0xFA, 0x0E, 0x11,
                0x21, 0x26, 0xFF, 0x3E, 0x80, 0x32, 0xE2, 0x0C, 0x3E, 0xF3, 0x32, 0xE2, 0x0C, 0x3E,
                0x77, 0x32, 0xE2, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1A, 0xCD, 0xB8, 0x00, 0x1A,
                0xCB, 0x37, 0xCD, 0xB8, 0x00, 0x13, 0x7B, 0xFE, 0x34, 0x20, 0xF0, 0x11, 0xCC, 0x00,
                0x06, 0x08, 0x1A, 0x13, 0x22, 0x23, 0x05, 0x20, 0xF9, 0x21, 0x04, 0x99, 0x01, 0x0C,
                0x01, 0xCD, 0xB1, 0x00, 0x3E, 0x19, 0x77, 0x21, 0x24, 0x99, 0x0E, 0x0C, 0xCD, 0xB1,
                0x00, 0x3E, 0x91, 0xE0, 0x40, 0x06, 0x10, 0x11, 0xD4, 0x00, 0x78, 0xE0, 0x43, 0x05,
                0x7B, 0xFE, 0xD8, 0x28, 0x04, 0x1A, 0xE0, 0x47, 0x13, 0x0E, 0x1C, 0xCD, 0xA7, 0x00,
                0xAF, 0x90, 0xE0, 0x43, 0x05, 0x0E, 0x1C, 0xCD, 0xA7, 0x00, 0xAF, 0xB0, 0x20, 0xE0,
                0xE0, 0x43, 0x3E, 0x83, 0xCD, 0x9F, 0x00, 0x0E, 0x27, 0xCD, 0xA7, 0x00, 0x3E, 0xC1,
                0xCD, 0x9F, 0x00, 0x11, 0x8A, 0x01, 0xF0, 0x44, 0xFE, 0x90, 0x20, 0xFA, 0x1B, 0x7A,
                0xB3, 0x20, 0xF5, 0x18, 0x49, 0x0E, 0x13, 0xE2, 0x0C, 0x3E, 0x87, 0xE2, 0xC9, 0xF0,
                0x44, 0xFE, 0x90, 0x20, 0xFA, 0x0D, 0x20, 0xF7, 0xC9, 0x78, 0x22, 0x04, 0x0D, 0x20,
                0xFA, 0xC9, 0x47, 0x0E, 0x04, 0xAF, 0xC5, 0xCB, 0x10, 0x17, 0xC1, 0xCB, 0x10, 0x17,
                0x0D, 0x20, 0xF5, 0x22, 0x23, 0x22, 0x23, 0xC9, 0x3C, 0x42, 0xB9, 0xA5, 0xB9, 0xA5,
                0x42, 0x3C, 0x00, 0x54, 0xA8, 0xFC, 0x42, 0x4F, 0x4F, 0x54, 0x49, 0x58, 0x2E, 0x44,
                0x4D, 0x47, 0x20, 0x76, 0x31, 0x2E, 0x32, 0x00, 0x3E, 0xFF, 0xC6, 0x01, 0x0B, 0x1E,
                0xD8, 0x21, 0x4D, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x3E, 0x01, 0xE0, 0x50,
            ],

            interrupts: Interrupts::new(),
            timer: Timer::new(),
            input: Input::new(),
        }
    }

    fn handle_blargg_output(&mut self, c: char) {
        if c == "\n".chars().next().unwrap() {
            let string = String::from_iter(self.blargg_output_buffer.iter());
            log::info!("{}", string);
            self.blargg_output_buffer.clear();
        } else {
            self.blargg_output_buffer.push(c);
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr & 0xF000 {
            0x0000 | 0x1000 | 0x2000 | 0x3000 | 0x4000 | 0x5000 | 0x6000 | 0x7000 => {
                if self.bios_enabled && addr < 0x100 {
                    return self.bios[addr as usize];
                }

                self.cartridge.read_rom(addr)
            }
            0x8000 | 0x9000 => self.ppu.gpu_vram[(addr - 0x8000) as usize],
            0xA000 | 0xB000 => self.cartridge.read_ram(addr - 0xA000),
            0xC000 | 0xD000 => self.working_ram[(addr - 0xC000) as usize],
            0xE000 => self.working_ram[(addr - 0xE000) as usize],
            0xF000 => match addr & 0x0F00 {
                0x0000 | 0x0100 | 0x0200 | 0x0300 | 0x0400 | 0x0500 | 0x0600 | 0x0700 | 0x0800
                | 0x0900 | 0x0A00 | 0x0B00 | 0x0C00 | 0x0D00 => {
                    self.working_ram[(addr - 0xE000) as usize]
                }

                0x0E00 => self.ppu.sprite_table[(addr - 0xFE00) as usize],

                0x0F00 => match addr {
                    0xFF00 => self.input.read_joyp(),
                    0xFF04..=0xFF07 => self.timer.read(addr),
                    0xFF0F => 0b1110_0000 | (self.interrupts.flags & 0b0001_1111),
                    0xFFFF => self.interrupts.enable,

                    0xFF40..=0xFF4B => self.ppu.read_u8(addr),

                    0xFF00..=0xFF7F => self.io[(addr - 0xFF00) as usize],
                    0xFF80..=0xFFFE => self.zero_page[(addr - 0xFF80) as usize],

                    _ => todo!("read u8 @{:#06X}", addr),
                },

                _ => todo!("read u8 @{:#06X}", addr),
            },

            _ => todo!("read u8 @{:#06X}", addr),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr & 0xF000 {
            0x0000 | 0x1000 | 0x2000 | 0x3000 | 0x4000 | 0x5000 | 0x6000 | 0x7000 => {
                self.cartridge.write_rom(addr, val);
            }
            0x8000 | 0x9000 => self.ppu.gpu_vram[(addr - 0x8000) as usize] = val,
            0xA000 | 0xB000 => self.cartridge.write_ram(addr - 0xA000, val),
            0xC000 | 0xD000 => self.working_ram[(addr - 0xC000) as usize] = val,
            0xE000 => self.working_ram[(addr - 0xE000) as usize] = val,

            0xF000 => match addr & 0x0F00 {
                0x0000 | 0x0100 | 0x0200 | 0x0300 | 0x0400 | 0x0500 | 0x0600 | 0x0700 | 0x0800
                | 0x0900 | 0x0A00 | 0x0B00 | 0x0C00 | 0x0D00 => {
                    self.working_ram[(addr - 0xE000) as usize] = val;
                }

                0x0E00 => {
                    if addr < 0xFEA0 {
                        self.ppu.sprite_table[(addr - 0xFE00) as usize] = val;
                    }
                }

                0x0F00 => match addr {
                    0xFF00 => self.input.set_column_line(val),
                    0xFF01 => self.handle_blargg_output(val as char),
                    0xFF04..=0xFF07 => self.timer.write(addr, val),
                    0xFF0F => self.interrupts.flags = val,
                    0xFFFF => self.interrupts.enable = val,

                    0xFF46 => {
                        let source_addr: u16 = (val as u16) << 8;

                        for i in 0..160 {
                            let src_val = self.read_u8(source_addr + i);
                            self.write_u8(0xFE00 + i, src_val);
                        }
                    }
                    0xFF40..=0xFF4B => self.ppu.write_u8(addr, val),

                    0xFF00..=0xFF7F => self.io[(addr - 0xFF00) as usize] = val,
                    0xFF80..=0xFFFE => self.zero_page[(addr - 0xFF80) as usize] = val,

                    _ => unreachable!(),
                },

                _ => todo!("read u8 @{:#06X}", addr),
            },

            _ => todo!("write u8 @{:#06X}", addr),
        }
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        self.read_u8(addr) as u16 + ((self.read_u8(addr + 1) as u16) << 8)
    }

    pub fn write_u16(&mut self, addr: u16, val: u16) {
        let lower_val: u8 = (val & 0x00FF) as u8;
        let higher_val: u8 = ((val & 0xFF00) >> 8) as u8;

        self.write_u8(addr, lower_val);
        self.write_u8(addr + 1, higher_val);
    }
}
