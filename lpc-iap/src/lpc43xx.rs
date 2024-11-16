pub const PAGE_SIZE: u32 = 512;
pub const SECTOR_SIZE: u32 = 65536;
pub const SECTOR_SIZE_SMALL: u32 = 8192;
pub const SECTOR_SIZE_LARGE: u32 = 0x10000;
pub const BANK_A_START_ADDR: u32 = 0x1A000000;
pub const BANK_A_8K_ADDR: u32 = 0x1A000000;
pub const BANK_A_64K_ADDR: u32 = 0x1A010000;
pub const EMPTY_VAL: u8 = 0xFF;
pub const FLASH_SIZE: u32 = 0x00080000;

// NOTE: unlike other LPC chips this address stores the location of the
// IAP function, not the function itself.
pub const IAP_ENTRY_ADDRESS: *mut u32 = 0x10400100 as *mut u32;

// reset value = 0x1040_0000
pub const M4MEMMAP: *mut u32 = 0x40043100 as *mut u32;

pub const CCLK: u32 = 12000;

pub const STARTUP_CORE_CLOCK_FREQ_KHZ: u32 = 120_000;

pub const CRP: *mut u32 = 0x0000_02fc as *mut u32;

pub const BASE_MX_CLK: *mut u32 = 0x4005006C as *mut u32;

pub const CHIP_NAME: &str = "lpc43xx";

#[cfg(feature = "defmt")]
use defmt::Format;

pub struct Chip {
    iap_address: usize,
}

impl crate::iap::Iap for Chip {
    fn new() -> Self {
        Chip {
            iap_address: unsafe { IAP_ENTRY_ADDRESS.read_volatile() as usize },
        }
    }

    fn iap_address(&self) -> usize {
        self.iap_address
    }

    fn chip_init(&self) {
        unsafe {
            // Autoblock En, Set clock source to IRC
            BASE_MX_CLK.write_volatile((0x1 << 11) | (0x1 << 24));
            M4MEMMAP.write_volatile(BANK_A_START_ADDR);
        }
        let _ = self.init();
    }

    fn addr_to_sector(&self, addr: u32) -> u32 {
        //  8kB Sector
        let n = (addr & 0x000FF000) >> 13;
        if n >= 0x07 {
            // 64kB Sector
            0x07 + (n >> 3)
        } else {
            n
        }
    }
}
