pub const PAGE_SIZE: u32 = 64;
pub const SECTOR_SIZE: usize = 1024;
pub const FLASH_SIZE: usize = 0x8000;
pub const EMPTY_VAL: u8 = 0x00;

pub const IAP_ENTRY_ADDRESS: usize = 0x0F00_1FF1;

pub const MEMMAP: *mut u32 = 0x40048000 as *mut u32;

pub const MAINCLKSEL: *mut u32 = 0x40048050 as *mut u32;
pub const MAINCLKUEN: *mut u32 = 0x40048054 as *mut u32;
pub const SYSAHBCLKDIV: *mut u32 = 0x40048058 as *mut u32;

pub const STARTUP_CORE_CLOCK_FREQ_KHZ: u32 = 12_000;

pub const CRP: *mut u32 = 0x0000_02fc as *mut u32;

pub struct Chip;

impl crate::iap::Iap for Chip {
    fn new() -> Self {
        Self
    }

    fn iap_address(&self) -> usize {
        IAP_ENTRY_ADDRESS
    }

    fn chip_init(&self) {
        let _ = self.init();
        unsafe {
            // Select FRO
            MAINCLKSEL.write_volatile(0);
            // Update Main Clock Source by toggling register
            // Needs to see a 0 -> 1 transition
            MAINCLKUEN.write_volatile(0);
            MAINCLKUEN.write_volatile(1);
            // Wait until clock change complete
            while (MAINCLKUEN.read_volatile() & 1) == 0 {}
            SYSAHBCLKDIV.write_volatile(1);
            // Map flash to address 0
            MEMMAP.write_volatile(0x02);
        }
    }

    fn addr_to_sector(&self, addr: u32) -> u32 {
        // addr / SECTOR_SIZE as u32
        addr >> 10 as u32
    }
}
