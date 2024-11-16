pub const PAGE_SIZE: u32 = 256;
pub const SECTOR_SIZE: u32 = 4096;
pub const EMPTY_VAL: u8 = 0xFF;
pub const FLASH_SIZE: u32 = 0x8000;

pub const IAP_ENTRY_ADDRESS: usize = 0x1FFF1FF1;

pub const MAINCLKSEL: *mut u32 = 0x40048070 as *mut u32;
pub const MAINCLKUEN: *mut u32 = 0x40048074 as *mut u32;
pub const MAINCLKDIV: *mut u32 = 0x40048078 as *mut u32;

pub const PLL0CON: *mut u32 = 0x400FC080 as *mut u32;
pub const PLL0CFG: *mut u32 = 0x400FC084 as *mut u32;
pub const PLL0STAT: *mut u32 = 0x400FC088 as *mut u32;
pub const PLL0FEED: *mut u32 = 0x400FC08C as *mut u32;

pub const MEMMAP: *mut u32 = 0x40048000 as *mut u32;

pub const STARTUP_CORE_CLOCK_FREQ_KHZ: u32 = 12_000;

pub const CRP: *mut u32 = 0x0000_02fc as *mut u32;

pub const CHIP_NAME: &str = "lpc111x";

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
            // Update Main Clock Source by toggling register
            // Needs to see a 0 -> 1 transition
            MAINCLKUEN.write_volatile(0);
            MAINCLKUEN.write_volatile(1);
            // Wait until clock change complete
            while (MAINCLKUEN.read_volatile() & 1) == 0 {}
            MAINCLKDIV.write_volatile(1);
            // Map flash to address 0
            MEMMAP.write_volatile(0x02);
        }
    }

    fn addr_to_sector(&self, addr: u32) -> u32 {
        addr >> 10
    }
}
