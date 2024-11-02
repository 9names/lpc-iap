#![no_std]
#![no_main]

use flash_algorithm::*;
use lpc1549::Chip;
use rtt_target::{rprintln, rtt_init_print};

use lpc_iap::iap::err_decode;
use lpc_iap::iap::Iap;
use lpc_iap::*;

use lpc_iap::lpc1549::PAGE_SIZE;
use lpc_iap::lpc1549::SECTOR_SIZE;
use lpc_iap::lpc1549::SYSMEMREMAP;
struct Algorithm;

algorithm!(Algorithm, {
    flash_address: 0x0,
    flash_size: 0x00040000,
    page_size: PAGE_SIZE,
    empty_value: 0xFF,
    sectors: [{
        size: SECTOR_SIZE as u32,
        address: 0x0,
    }]
});

// Leave processor at default clock speed
const DEFAULT_SYSTEM_CLK: u32 = 12_000_000;

fn addr_to_sector(addr: u32) -> u32 {
    // let's assume that probe-rs will never ask for an invalid address
    addr / SECTOR_SIZE as u32
}

fn erase_sectors(sector_start: u32, sector_end: u32) -> Result<(), ()> {
    let chip = Chip::new();
    // Check if the sectors need erasing first
    if let Err(e) = chip.blank_check_sector(sector_start, sector_end) {
        let e = err_decode(e);
        if e == lpc_iap::IapRetcode::SectorNotBlank {
            rprintln!("Sec!empty");
            if let Ok(()) = chip.prepare_sector_for_write(sector_start, sector_end) {
                rprintln!("FlUnl");
            } else {
                return Err(());
            }

            if let Ok(()) = chip.erase_sector(sector_start, sector_end, DEFAULT_SYSTEM_CLK) {
                rprintln!("ErOpSuc");
            } else {
                return Err(());
            }

            if let Ok(()) = chip.blank_check_sector(sector_start, sector_end) {
                rprintln!("ChkOk");
            } else {
                return Err(());
            }
        }
    }
    Ok(())
}

impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        rtt_init_print!();
        rprintln!("Init");
        unsafe {
            SYSMEMREMAP.write_volatile(0x02);
        }

        Ok(Self)
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        rprintln!("Erase All");
        let result = erase_sectors(0, 63);
        if result.is_ok() {
            Ok(())
        } else {
            Err(ErrorCode::new(0x70d1).unwrap())
        }
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        rprintln!("Erase sector addr:{}", addr);
        let sector = addr_to_sector(addr);
        rprintln!("Erase sector num: {}", sector);
        if let Ok(()) = erase_sectors(sector, sector) {
            rprintln!("ErSuc");
            Ok(())
        } else {
            Err(ErrorCode::new(0x70d3).unwrap())
        }
    }

    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        let chip = Chip::new();
        rprintln!("Program Page addr:{} size:{}", addr, data.len());
        let datalen = data.len() as u32;
        let sector_start = addr_to_sector(addr);
        let sector_end = addr_to_sector(addr + datalen);
        // unlock sectors first
        if let Ok(()) = chip.prepare_sector_for_write(sector_start, sector_end) {
            rprintln!("FlUnl");
        } else {
            return Err(ErrorCode::new(0x70d4).unwrap());
        }
        match chip.copy_ram_to_flash(addr, data, DEFAULT_SYSTEM_CLK) {
            Ok(_) => rprintln!("WrOpSuc"),
            Err(_e) => {
                rprintln!("WrOpEr");
                return Err(ErrorCode::new(0x70d5).unwrap());
            }
        }
        match chip.compare(addr, data, DEFAULT_SYSTEM_CLK) {
            Ok(_) => rprintln!("VerSuc"),
            Err(e) => {
                if e == 10 {
                    rprintln!("Src+Dest!equal");
                    return Err(ErrorCode::new(0x70d6).unwrap());
                } else {
                    rprintln!("VerEr");
                    return Err(ErrorCode::new(0x70d7).unwrap());
                }
            }
        }
        Ok(())
    }
}

impl Drop for Algorithm {
    fn drop(&mut self) {}
}
