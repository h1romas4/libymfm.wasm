// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use std::{cell::RefCell, rc::Rc};

pub type RomBank = Option<Rc<RefCell<RomSet>>>;

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
    C140_ROM = 0x8d,
    NOT_SUPPOTED = 0xff,
}

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum RomBusType {
    C140_TYPE_SYSTEM2,
    C140_TYPE_SYSTEM21,
    C140_TYPE_ASIC219,
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
/// Rom set
///
pub struct RomSet {
    rom: Vec<Rom>,
    bus_type: Option<RomBusType>,
}

impl RomSet {
    pub fn new() -> RomSet {
        RomSet {
            rom: Vec::new(),
            bus_type: None,
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

    ///
    /// Read the data from the ROM address.
    ///
    pub fn read_byte(&self, address: usize) -> u8 {
        self.read(address)
    }

    ///
    /// Read the data from the ROM address.
    ///
    pub fn read_word(&self, address: usize) -> u16 {
        // address decode
        match self.bus_type {
            Some(RomBusType::C140_TYPE_SYSTEM2) => {
                let offset = ((address & 0x200000) >> 2) | (address & 0x7ffff);
                // high 8 bit only
                u16::from(self.read(offset)) << 8
            }
            Some(RomBusType::C140_TYPE_SYSTEM21) => {
                let offset = ((address & 0x300000) >> 1) | (address & 0x7ffff);
                u16::from(self.read(offset)) << 8
            }
            Some(RomBusType::C140_TYPE_ASIC219) => 0, // c140 not used in this mode
            None => {
                (u16::from(self.read(address * 2 + 1)) << 8) | u16::from(self.read(address * 2))
            }
        }
    }

    ///
    /// Get specify ROM referance by index.
    ///
    pub fn ger_rom_ref(&self, index_no: usize) -> (*const u8, usize, usize) {
        self.rom[index_no].get_memory_ref()
    }

    ///
    /// Set ROM Bus type
    ///
    pub fn set_bus_type(&mut self, bus_type: Option<RomBusType>) {
        self.bus_type = bus_type;
    }

    #[inline]
    fn read(&self, address: usize) -> u8 {
        for r in self.rom.iter() {
            if r.start_address <= address && r.end_address >= address {
                return r.memory[address - r.start_address];
            }
        }
        0
    }
}

///
/// Read RomBank by address utility
///
pub fn read_byte_rombank(rombank: &RomBank, address: usize) -> u8 {
    rombank.as_ref().unwrap().borrow().read_byte(address)
}

///
/// Read RomBank by address utility
///
pub fn read_word_rombank(rombank: &RomBank, address: usize) -> u16 {
    rombank.as_ref().unwrap().borrow().read_word(address)
}

///
/// Get Rom referance utility
///
pub fn get_rom_ref(rombank: &RomBank, index_no: usize) -> (*const u8, usize, usize) {
    rombank.as_ref().unwrap().borrow().ger_rom_ref(index_no)
}
