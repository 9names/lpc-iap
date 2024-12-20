pub const PAGE_SIZE: u32 = 1024;
pub const SECTOR_SIZE: u32 = 4096;
pub const EMPTY_VAL: u8 = 0xFF;
pub const FLASH_SIZE: u32 = 0x40000;

pub const IAP_ENTRY_ADDRESS: usize = 0x03000205;

pub const SYSMEMREMAP: *mut u32 = 0x40074000 as *mut u32;
pub const MAINCLKSELA: *mut u32 = 0x40074080 as *mut u32;
pub const MAINCLKSELB: *mut u32 = 0x40074084 as *mut u32;
pub const SYSAHBCLKDIV: *mut u32 = 0x400740C0 as *mut u32;
pub const SYSAHBCLKCTRL0: *mut u32 = 0x400740C4 as *mut u32;

pub const STARTUP_CORE_CLOCK_FREQ_KHZ: u32 = 12_000;

pub const CHIP_NAME: &str = "lpc154x";

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
            SYSMEMREMAP.write_volatile(0x02);
        }
    }

    fn addr_to_sector(&self, addr: u32) -> u32 {
        // let's assume that probe-rs will never ask for an invalid address
        addr / SECTOR_SIZE as u32
    }
}
