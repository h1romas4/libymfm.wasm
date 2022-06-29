// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use std::{cell::RefCell, rc::Rc};

pub type RomBank = Option<Rc<RefCell<RomSet>>>;
pub type RomBus<T> = Option<Rc<RefCell<T>>>;

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum RomIndex {
    SEGAPCM_ROM = 0x80,
    YM2608_DELTA_T = 0x81,
    YM2610_ADPCM = 0x82,
    YM2610_DELTA_T = 0x83,
    YMF278B_ROM = 0x84,
    YMF278B_RAM = 0x87,
    Y8950_ROM = 0x88,
    OKIM6295_ROM = 0x8b,
    C140_ROM = 0x8d,
    NOT_SUPPOTED = 0xff,
}

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum RomBusType {
    C140_TYPE_SYSTEM2,
    C140_TYPE_SYSTEM21,
    C219_TYPE_ASIC219,
    OKIM6295,
}

///
/// Rom
///
pub struct Rom {
    start_address: usize,
    end_address: usize,
    memory: Vec<u8>,
}

impl Rom {
    ///
    /// Get rom memory pointer, start address attribute and length.
    ///
    fn get_memory_ref(&self) -> (*const u8, usize, usize) {
        (self.memory.as_ptr(), self.start_address, self.memory.len())
    }
}

///
/// Address decoder
///
pub trait Decoder {
    fn decode(&self, rombank: &RomSet, address: usize) -> u32;
}

///
/// Rom set
///
pub struct RomSet {
    rom: Vec<Rom>,
    rom_bus: Option<Rc<RefCell<dyn Decoder>>>,
}

impl RomSet {
    pub fn new() -> RomSet {
        RomSet {
            rom: Vec::new(),
            rom_bus: None,
        }
    }

    ///
    /// Add a ROM to the rom set.
    ///
    pub fn add_rom(&mut self, memory: &[u8], start_address: usize, end_address: usize) -> usize {
        // println!("rom: {:<08x} - {:<08x}, {:<08x}, {:<02x}", start_address, end_address, memory.len(), memory[0]);
        // to_vec(clone) is external SPI memory simulation.
        self.rom.push(Rom {
            start_address,
            end_address,
            memory: memory.to_vec(),
        });
        // Return index
        self.rom.len() - 1
    }

    #[inline]
    pub fn read(&self, address: usize) -> u8 {
        for r in self.rom.iter() {
            if r.start_address <= address && r.end_address >= address {
                return r.memory[address - r.start_address];
            }
        }
        0
    }

    ///
    /// Read the data from the ROM address.
    ///
    pub fn read_byte(&self, address: usize) -> u8 {
        if self.rom_bus.is_some() {
            return self
                .rom_bus
                .as_ref()
                .unwrap()
                .as_ref()
                .borrow()
                .decode(self, address) as u8;
        }
        self.read(address)
    }

    ///
    /// Read the data from the ROM address.
    ///
    pub fn read_word(&self, address: usize) -> u16 {
        if self.rom_bus.is_some() {
            return self
                .rom_bus
                .as_ref()
                .unwrap()
                .as_ref()
                .borrow()
                .decode(self, address) as u16;
        }
        (u16::from(self.read(address * 2 + 1)) << 8) | u16::from(self.read(address * 2))
    }

    ///
    /// Get specify ROM referance by index.
    ///
    pub fn ger_rom_ref(&self, index_no: usize) -> (*const u8, usize, usize) {
        self.rom[index_no].get_memory_ref()
    }

    ///
    /// Set ROM Bus
    ///
    pub fn set_rom_bus(&mut self, rom_bus: Option<Rc<RefCell<dyn Decoder>>>) {
        self.rom_bus = rom_bus;
    }
}

///
/// Read RomBank by address utility
///
pub fn read_byte(rombank: &RomBank, address: usize) -> u8 {
    rombank
        .as_ref()
        .unwrap()
        .as_ref()
        .borrow()
        .read_byte(address)
}

///
/// Read RomBank by address utility
///
pub fn read_word(rombank: &RomBank, address: usize) -> u16 {
    rombank
        .as_ref()
        .unwrap()
        .as_ref()
        .borrow()
        .read_word(address)
}

///
/// Get Rom referance utility
///
pub fn get_rom_ref(rombank: &RomBank, index_no: usize) -> (*const u8, usize, usize) {
    rombank
        .as_ref()
        .unwrap()
        .as_ref()
        .borrow()
        .ger_rom_ref(index_no)
}

///
/// Set ROM bus utility
///
pub fn set_rom_bus(rombank: &RomBank, rom_bus: Option<Rc<RefCell<dyn Decoder>>>) {
    rombank
        .as_ref()
        .unwrap()
        .as_ref()
        .borrow_mut()
        .set_rom_bus(rom_bus);
}
