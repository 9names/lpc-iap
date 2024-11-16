#![no_std]
#![no_main]

use flash_algorithm::*;

use lpc_iap::iap::err_decode;
use lpc_iap::iap::Iap;
use lpc_iap::lpc81x::{
    Chip, EMPTY_VAL, FLASH_SIZE, PAGE_SIZE, SECTOR_SIZE, STARTUP_CORE_CLOCK_FREQ_KHZ,
};
struct Algorithm;

algorithm!(Algorithm, {
    device_name: "lpc810",
    device_type: DeviceType::Onchip,
    flash_address: 0x0,
    flash_size: FLASH_SIZE,
    page_size: PAGE_SIZE,
    empty_value: EMPTY_VAL,
    program_time_out: 1500,
    erase_time_out: 1500,
    sectors: [{
        size: SECTOR_SIZE as u32,
        address: 0x0,
    }]
});

// 1KB RAM is not enough room for error handling. this algo ignores all failures.

impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        let chip = Chip::new();
        chip.chip_init();
        Ok(Self)
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        let chip = Chip::new();
        let sector = chip.addr_to_sector(addr);
        let _ = chip.prepare_sector_for_write(sector, sector);
        let _ = chip.erase_sector(sector, sector, STARTUP_CORE_CLOCK_FREQ_KHZ);
        Ok(())
    }

    fn program_page(&mut self, addr: u32, data: &mut [u8]) -> Result<(), ErrorCode> {
        let datalen = data.len() as u32;
        let chip = Chip::new();
        let sector_start = chip.addr_to_sector(addr);

        // unlock sectors first
        let _ = chip.prepare_sector_for_write(sector_start, sector_start);
        // write to flash
        let _ = chip.copy_ram_to_flash(addr, data, STARTUP_CORE_CLOCK_FREQ_KHZ);
        Ok(())
    }
}

impl Drop for Algorithm {
    fn drop(&mut self) {}
}
