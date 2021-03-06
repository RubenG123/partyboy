mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
pub mod rom;

use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::cartridge::{mbc1::Mbc1, mbc2::Mbc2, mbc3::Mbc3, mbc5::Mbc5, rom::Rom};

pub struct RamIter {
    pub(self) ram: Vec<u8>,
}

impl RamIter {
    pub fn empty() -> Self {
        Self { ram: Vec::new() }
    }
}

impl From<Vec<u8>> for RamIter {
    fn from(ram: Vec<u8>) -> Self {
        Self { ram }
    }
}

impl RamIter {
    pub fn as_slice(&self) -> &[u8] {
        self.ram.as_slice()
    }
}

pub trait Cartridge {
    fn read_rom(&self, addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, value: u8);

    fn read_ram(&self, addr: u16) -> u8;
    fn write_ram(&mut self, addr: u16, value: u8);

    // we cant return `impl trait` in rust yet so lets do this
    fn iter_ram(&self) -> RamIter;
    fn has_ram(&self) -> bool;
    fn save_file_path(&self) -> Option<&PathBuf>;

    fn create_save_file(&self) {
        let ram_iter = self.iter_ram();
        let save_file_path = self.save_file_path();

        if let Some(save_file_path) = save_file_path {
            if self.has_ram() {
                let mut sav_file = File::create(save_file_path).unwrap();
                // TODO: handle result
                let _ = sav_file.write_all(ram_iter.as_slice());
                log::info!("Save file written!");
            }
        }
    }
}

pub fn create(rom_path: &str) -> Box<dyn Cartridge> {
    let mut rom_bank_0 = [0u8; 0x4000];

    let path = Path::new(rom_path);
    let file = File::open(path);
    let mut file = match file {
        Ok(f) => f,
        Err(err) => panic!("Something went wrong reading the ROM: {}", err),
    };

    file.read_exact(&mut rom_bank_0).ok();

    // parse cart header
    // CGB flag
    log::info!("CGB compat mode: {:#04X}", rom_bank_0[0x143]);
    if rom_bank_0[0x143] == 0xC0 {
        log::warn!("This rom is only supported for game boy color");
    }

    let cartridge_type_code = rom_bank_0[0x147];
    let rom_size_code = rom_bank_0[0x148];
    let ram_size_code = rom_bank_0[0x149];

    // This includes rom bank 0
    let num_rom_banks: u16 = match rom_size_code {
        0x00 => 2,   // 32KB
        0x01 => 4,   // 64KB
        0x02 => 8,   // 128KB
        0x03 => 16,  // 256KB
        0x04 => 32,  // 512KB
        0x05 => 64,  // 1MB
        0x06 => 128, // 2MB
        0x07 => 256, // 4MB
        0x08 => 512, // 8MB
        0x09 => 1024,

        // pandocs says there are some other special codes
        // but is not sure if they are legit
        // lets define them anyway
        0x52 => 72, // 1.1MB
        0x53 => 80, // 1.2MB
        0x54 => 96, // 1.5MB

        _ => panic!(
            "Cartridge has invalid ROM size code? Code: {:#04X}",
            rom_size_code
        ),
    };

    log::debug!("rom size code: {}, banks: {}", rom_size_code, num_rom_banks);

    let num_ram_banks: u16 = match ram_size_code {
        0x00 => 0,
        0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        0x05 => 8,

        _ => panic!(
            "Cartridge has invalid RAM size code? Code: {:#04X}",
            ram_size_code
        ),
    };

    log::debug!("ram size code: {}, banks: {}", ram_size_code, num_ram_banks);

    match cartridge_type_code {
        0x00 => Box::new(Rom::new(file, rom_bank_0)),

        0x01 | 0x02 | 0x03 => {
            log::info!("MBC1 cart detected!");
            Box::new(Mbc1::new(
                file,
                path,
                rom_bank_0,
                num_rom_banks,
                num_ram_banks,
            ))
        }

        0x05 | 0x06 => {
            log::info!("MBC2 cart detected!");
            Box::new(Mbc2::new(file, path, rom_bank_0, num_rom_banks))
        }

        0x0F..=0x13 => {
            log::info!("MBC3 cart detected!");
            Box::new(Mbc3::new(
                file,
                path,
                rom_bank_0,
                num_rom_banks,
                num_ram_banks,
            ))
        }

        0x19..=0x1E => {
            log::info!("MBC5 cart detected!");
            Box::new(Mbc5::new(
                file,
                path,
                rom_bank_0,
                num_rom_banks,
                num_ram_banks,
            ))
        }

        _ => unimplemented!(
            "Unable to handle cartridge type: {:#04X}",
            cartridge_type_code
        ),
    }
}

fn get_save_file_path_from_rom_path(path: &Path) -> PathBuf {
    let mut save_file_path = PathBuf::from(path);
    let file_name = save_file_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    save_file_path.pop();
    save_file_path.push(format!("{}.sav", file_name));
    save_file_path
}

fn try_read_save_file(sav_file_path: &Path, num_ram_banks: u16, ram_banks: &mut Vec<[u8; 0x2000]>) {
    load_new_ram(ram_banks, num_ram_banks);

    let save_file = File::open(sav_file_path);
    if let Ok(mut file) = save_file {
        let mut buf: Vec<u8> = Vec::new();

        if let Ok(bytes_read) = file.read_to_end(&mut buf) {
            if bytes_read != num_ram_banks as usize * 0x2000 {
                log::warn!(
                    "Save file was an unexpected length. Expected {}, actual: {}",
                    num_ram_banks as usize * 0x2000,
                    bytes_read
                );
                return;
            }

            // load save file
            let mut index: usize = 0;
            for bank in ram_banks {
                for val in bank.iter_mut().take(0x2000) {
                    *val = buf[index];
                    index += 1;
                }
            }

            log::info!("Save file loaded!");
        }
    }
}

fn load_new_ram(ram_banks: &mut Vec<[u8; 0x2000]>, num_ram_banks: u16) {
    // fill ram banks with blank memory
    for _ in 0..num_ram_banks {
        let bank = [0; 0x2000];
        ram_banks.push(bank);
    }
}

macro_rules! impl_drop_save_cartridge {
    ($($cart:ty),*) => {
        $(
            impl Drop for $cart {
                fn drop(&mut self) {
                    self.create_save_file()
                }
            }
        )*
    };
}

impl_drop_save_cartridge! {
    Mbc1, Mbc2, Mbc3, Mbc5
}
