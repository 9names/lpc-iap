#![no_std]
#![no_main]

use flash_algorithm::*;
use rtt_target::{rprint, rprintln, rtt_init_print};

use lpc_iap::lpc1788::{
    Chip, PAGE_SIZE, SECTOR_SIZE_LARGE, SECTOR_SIZE_SMALL, STARTUP_CORE_CLOCK_FREQ_KHZ,
};
struct Algorithm;

use lpc_iap::iap::{err_decode, Iap};

// 0.4 algo macro

// algorithm!(Algorithm, {
//     flash_address: 0x0,
//     flash_size: 0x80000,
//     page_size: PAGE_SIZE,
//     empty_value: 0xff,
//     sectors: [{
//         size: SECTOR_SIZE,
//         address: 0x0,
//     }]
// });

// 0.5 algo macro

fn print_nibble(val: u8) {
    let a = val & 0x0f;
    let a = if a < 10 { a + b'0' } else { a + (b'a' - 10) };
    let c = a as char;
    let mut my_buf: [u8; 4] = [0; 4];
    let my_str: &str = c.encode_utf8(&mut my_buf);
    rprint!(my_str);
}

fn print_u8(val: u8) {
    print_nibble(val >> 4);
    print_nibble(val & 0xf);
}

fn println_u8(val: u8) {
    rprint!("0x");
    print_u8(val);
    rprint!("\n");
}

fn print_u32(val: u32) {
    rprint!("0x");
    print_u8((val >> 24) as u8);
    print_u8((val >> 16) as u8);
    print_u8((val >> 8) as u8);
    print_u8((val) as u8);
}

fn println_u32(val: u32) {
    print_u32(val);
    rprint!("\n");
}

algorithm!(Algorithm, {
    device_name: "lpc1788",
    device_type: DeviceType::Onchip,
    flash_address: 0x0,
    flash_size: 0x80000,
    page_size: PAGE_SIZE,
    empty_value: 0xff,
    program_time_out: 1500,
    erase_time_out: 1500,
    sectors: [{
        size: SECTOR_SIZE_SMALL,
        address: 0x0,
    },{
        size: SECTOR_SIZE_LARGE,
        address: 0x10000,
    }]
});

fn erase_sectors(sector_start: u32, sector_end: u32) -> Result<(), ()> {
    let chip = Chip::new();
    // Check if the sectors need erasing first
    if let Err(e) = chip.blank_check_sector(sector_start, sector_end) {
        let e = err_decode(e);
        if e == lpc_iap::IapRetcode::SectorNotBlank {
            // unlock sectors
            if let Ok(()) = chip.prepare_sector_for_write(sector_start, sector_end) {
            } else {
                return Err(());
            }

            // erase them
            if let Ok(()) = chip.erase_sector(sector_start, sector_end, STARTUP_CORE_CLOCK_FREQ_KHZ)
            {
            } else {
                return Err(());
            }

            // // verify that they were erased
            // if let Ok(()) = chip.blank_check_sector(sector_start, sector_end) {
            // } else {
            //     return Err(());
            // }
        }
    }
    Ok(())
}

#[cfg(feature = "lpc-boot-sig")]
fn generate_boot_sig(data: &mut [u8], addr: u32) {
    if addr == 0 {
        let mut checksum: u32 = 0;
        for i in 0..7 {
            let i = i << 2;
            let val: u32 = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
            checksum = checksum.wrapping_add(val);
        }
        let checksum = 0u32.wrapping_sub(checksum);
        let checksum = checksum.to_le_bytes();
        data[0x1c] = checksum[0];
        data[0x1c + 1] = checksum[1];
        data[0x1c + 2] = checksum[2];
        data[0x1c + 3] = checksum[3];
    }
}

#[cfg(not(feature = "lpc-boot-sig"))]
fn generate_boot_sig(_data: &mut [u8], _addr: u32) {}

impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        rtt_init_print!();
        rprintln!("Init");
        let chip = Chip::new();
        chip.chip_init();
        Ok(Self)
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        rprintln!("erase_sector");
        let chip = Chip::new();
        rprint!("address: ");
        println_u32(addr);
        let sector = chip.addr_to_sector(addr);
        rprint!("sector: ");
        println_u8(sector as u8);
        if let Ok(()) = erase_sectors(sector, sector) {
            Ok(())
        } else {
            Err(ErrorCode::new(2).unwrap())
        }
    }

    fn program_page(&mut self, addr: u32, data: &mut [u8]) -> Result<(), ErrorCode> {
        rprintln!("program_page");
        let chip = Chip::new();
        let datalen = data.len() as u32;

        let sector_start = chip.addr_to_sector(addr);
        let sector_end = chip.addr_to_sector(addr + datalen);

        generate_boot_sig(data, addr);

        // unlock sectors first
        if let Ok(()) = chip.prepare_sector_for_write(sector_start, sector_end) {
        } else {
            return Err(ErrorCode::new(3).unwrap());
        }
        // write to flash
        match chip.copy_ram_to_flash(addr, data, STARTUP_CORE_CLOCK_FREQ_KHZ) {
            Ok(_) => {}
            Err(_e) => {
                return Err(ErrorCode::new(4).unwrap());
            }
        }
        // verify that write was successful
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
        // TODO: Add code here to uninitialize the flash algorithm.
    }
}
