pub const PAGE_SIZE: u32 = 1024;
pub const SECTOR_SIZE: u32 = 4096;
pub const EMPTY_VAL: u8 = 0xFF;
pub const FLASH_SIZE: u32 = 0x80000;
pub const SECTOR_SIZE_2: u32 = 32768;
pub const SECTOR_ADDR_2: u32 = 0x10000;

pub const IAP_ENTRY_ADDRESS: usize = 0x1fff1ff1;

pub const SYSCON_BASE: u32 = 0x400FC000;
pub const CCLKSEL: *mut u32 = 0x400FC104 as *mut u32;
pub const CLKSRCSEL: *mut u32 = 0x400FC10C as *mut u32;
pub const MAINCLKSELB: *mut u32 = 0x40074084 as *mut u32;
pub const SYSAHBCLKDIV: *mut u32 = 0x400740C0 as *mut u32;
pub const SYSAHBCLKCTRL0: *mut u32 = 0x400740C4 as *mut u32;

pub const MEMMAP: *mut u32 = 0x400FC040 as *mut u32;

pub const STARTUP_CORE_CLOCK_FREQ_KHZ: u32 = 12_000;

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
            // Map flash to address 0
            MEMMAP.write_volatile(0x01);
        }
    }

    fn addr_to_sector(&self, addr: u32) -> u32 {
        // let's assume that probe-rs will never ask for an invalid address
        if addr < SECTOR_ADDR_2 {
            // first 16 sectors are 4KB
            addr / SECTOR_SIZE as u32
        } else {
            // remaining sectors are 32KB
            16 + ((addr - SECTOR_ADDR_2) / (SECTOR_SIZE_2 as u32))
        }
    }
}
