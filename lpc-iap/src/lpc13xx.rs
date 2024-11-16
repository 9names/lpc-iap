pub const PAGE_SIZE: u32 = 256;
pub const SECTOR_SIZE: u32 = 4096;
pub const EMPTY_VAL: u8 = 0xFF;
pub const FLASH_SIZE: u32 = 0x20000;

pub const IAP_ENTRY_ADDRESS: usize = 0x1FFF1FF1;

pub const SYSMEMREMAP: *mut u32 = 0x40048000 as *mut u32;

pub const MAINCLKSEL: *mut u32 = 0x40048070 as *mut u32;
// pub const MAINCLKUEN: *mut u32 = 0x40048074 as *mut u32;
pub const SYSAHBCLKDIV: *mut u32 = 0x40048078 as *mut u32;

pub const STARTUP_CORE_CLOCK_FREQ_KHZ: u32 = 12_000;

pub const CHIP_NAME: &str = "lpc13xx";

#[cfg(feature = "defmt")]
use defmt::Format;
pub struct Chip;

impl crate::iap::Iap for Chip {
    fn new() -> Self {
        Self
    }

    fn iap_address(&self) -> usize {
        IAP_ENTRY_ADDRESS
    }

    fn chip_init(&self) {
        unsafe {
            // Select Internal RC Oscillator
            MAINCLKSEL.write_volatile(0);

            // this might be necessary? doesn't seem to in testing.
            // SYSAHBCLKDIV.write_volatile(1);

            // Map flash to address 0
            SYSMEMREMAP.write_volatile(0x02);
        }
    }

    fn addr_to_sector(&self, addr: u32) -> u32 {
        addr / SECTOR_SIZE
    }
}
