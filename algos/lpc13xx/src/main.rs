#![no_std]
#![no_main]

use flash_algorithm::*;

use lpc_iap::lpc1347::{
    Chip, EMPTY_VAL, FLASH_SIZE, PAGE_SIZE, SECTOR_SIZE, STARTUP_CORE_CLOCK_FREQ_KHZ,
};

use lpc_iap::iap::{err_decode, Iap};

struct Algorithm;

algorithm!(Algorithm, {
    flash_address: 0x0,
    flash_size: FLASH_SIZE,
    page_size: PAGE_SIZE,
    empty_value: EMPTY_VAL,
    sectors: [{
        size: SECTOR_SIZE,
        address: 0x0,
    }]
});

fn erase_sectors(sector_start: u32, sector_end: u32) -> Result<(), ()> {
    let chip = Chip::new();
    // Check if the sectors need erasing first
    if let Err(e) = chip.blank_check_sector(sector_start, sector_end) {
        let e = err_decode(e);
        if e == lpc_iap::IapRetcode::SectorNotBlank {
            if let Ok(()) = chip.prepare_sector_for_write(sector_start, sector_end) {
            } else {
                return Err(());
            }

            if let Ok(()) = chip.erase_sector(sector_start, sector_end, STARTUP_CORE_CLOCK_FREQ_KHZ)
            {
            } else {
                return Err(());
            }

            if let Ok(()) = chip.blank_check_sector(sector_start, sector_end) {
            } else {
                return Err(());
            }
        }
    }
    Ok(())
}

impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        let chip = Chip::new();
        chip.chip_init();
        Ok(Self)
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        let result = erase_sectors(0, 7);
        if result.is_ok() {
            Ok(())
        } else {
            Err(ErrorCode::new(1).unwrap())
        }
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        let chip = Chip::new();
        let sector = chip.addr_to_sector(addr);
        if let Ok(()) = erase_sectors(sector, sector) {
            Ok(())
        } else {
            Err(ErrorCode::new(2).unwrap())
        }
    }

    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        let chip = Chip::new();
        let datalen = data.len() as u32;
        let sector_start = chip.addr_to_sector(addr);
        let sector_end = chip.addr_to_sector(addr + datalen);
        // unlock sectors first
        if let Ok(()) = chip.prepare_sector_for_write(sector_start, sector_end) {
        } else {
            return Err(ErrorCode::new(3).unwrap());
        }
        match chip.copy_ram_to_flash(addr, data, STARTUP_CORE_CLOCK_FREQ_KHZ) {
            Ok(_) => {}
            Err(_e) => {
                return Err(ErrorCode::new(4).unwrap());
            }
        }
        match chip.compare(addr, data, STARTUP_CORE_CLOCK_FREQ_KHZ) {
            Ok(_) => {}
            Err(e) => {
                if e == 10 {
                    return Err(ErrorCode::new(4).unwrap());
                } else {
                    return Err(ErrorCode::new(5).unwrap());
                }
            }
        }
        Ok(())
    }
}

impl Drop for Algorithm {
    fn drop(&mut self) {
        // nothing to do here
    }
}
