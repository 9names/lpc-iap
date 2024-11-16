#![no_std]

// IAP trait
pub mod iap;

// chip specific defines and IAP impls
pub mod lpc11xx;
pub mod lpc13xx;
pub mod lpc15xx;
pub mod lpc178x;
pub mod lpc43xx;
pub mod lpc80x;
pub mod lpc81x;
pub mod lpc82x;

// exported types
pub use iap::iap_retcode_raw;
pub use iap::IapRetcode;

// TODO: work out if CRP stuff should live here

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CrpLevel {
    NoCrp,
    NoIsp,
    Crp1,
    Crp2,
    Crp3,
}

pub const CRP: *mut u32 = 0x0000_02fc as *mut u32;
pub fn read_crp_setting() -> CrpLevel {
    let crp = unsafe { CRP.read_volatile() };
    match crp {
        0x4e69_7370 => CrpLevel::NoIsp,
        0x1234_5678 => CrpLevel::Crp1,
        0x8765_4321 => CrpLevel::Crp2,
        0x4321_8765 => CrpLevel::Crp3,
        _ => CrpLevel::NoCrp,
    }
}
