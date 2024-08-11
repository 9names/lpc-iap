// commands
#[allow(non_camel_case_types, dead_code)]
#[repr(u8)]
pub enum IapCommandNum {
    IAP_INIT = 49,                  // Initialise IAP (some parts)
    IAP_PREPARE = 50,               // Prepare sector(s) for write operation
    IAP_COPY_RAM2FLASH = 51,        // Copy RAM to Flash
    IAP_ERASE = 52,                 // Erase sector(s)
    IAP_BLANK_CHECK = 53,           // Blank check sector(s)
    IAP_READ_PART_ID = 54,          // Read chip part ID
    IAP_READ_BOOT_VER = 55,         // Read chip boot code version
    IAP_COMPARE = 56,               // Compare memory areas
    IAP_REINVOKE_ISP = 57,          // Jump into ISP bootloader
    IAP_READ_UID = 58,              // Read unique ID
    IAP_ERASE_PAGE = 59,            // Erase page(s)
    IAP_SET_ACTIVE_FLASH_BANK = 60, // Set which flash bank is active (some parts)
}

#[cfg(feature = "defmt")]
use defmt::Format;

use num_derive::FromPrimitive;

// return codes
#[derive(FromPrimitive, Debug, PartialEq)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IapRetcode {
    CmdSuccess = 0x0,
    InvalidCommand = 0x01,
    SrcAddrError = 0x02,
    DstAddrError = 0x03,
    SrcAddrNotMapped = 0x04,
    DstAddrNotMapped = 0x05,
    CountError = 0x06,
    InvalidSectorOrPage = 0x07,
    SectorNotBlank = 0x08,
    SectorNotPreparedForWriteOperation = 0x09,
    CompareError = 0x0a,
    Busy = 0x0b,
    ParamError = 0x0c,
    AddrError = 0x0d,
    AddrNotMapped = 0x0e,
    CmdLocked = 0x0f,
    InvalidCode = 0x10,
    InvalidBaudRate = 0x11,
    InvalidStopBit = 0x12,
    CodeReadProtectionEnabled = 0x13,
    UserCodeChecksum = 0x15,
    EfroNoPower = 0x17,
    FlashNoPower = 0x18,
    FlashEraseProgram = 0x20,
    InvalidPage = 0x21,
    FlashNoClock = 0x1b,
    ReinvokeIspConfig = 0x1c,
    NoValidImage = 0x1d,
    FailedToDecode = 0xff,
}

impl Default for IapRetcode {
    fn default() -> Self {
        Self::FailedToDecode
    }
}

pub mod iap_retcode_raw {
    pub const CMD_SUCCESS: u32 = 0x0;
    pub const INVALID_COMMAND: u32 = 0x01;
    pub const SRC_ADDR_ERROR: u32 = 0x02;
    pub const DST_ADDR_ERROR: u32 = 0x03;
    pub const SRC_ADDR_NOT_MAPPED: u32 = 0x04;
    pub const DST_ADDR_NOT_MAPPED: u32 = 0x05;
    pub const COUNT_ERROR: u32 = 0x06;
    pub const INVALID_SECTOR_OR_PAGE: u32 = 0x07;
    pub const SECTOR_NOT_BLANK: u32 = 0x08;
    pub const SECTOR_NOT_PREPARED_FOR_WRITE_OPERATION: u32 = 0x09;
    pub const COMPARE_ERROR: u32 = 0x0a;
    pub const BUSY: u32 = 0x0b;
    pub const PARAM_ERROR: u32 = 0x0c;
    pub const ADDR_ERROR: u32 = 0x0d;
    pub const ADDR_NOT_MAPPED: u32 = 0x0e;
    pub const CMD_LOCKED: u32 = 0x0f;
    pub const INVALID_CODE: u32 = 0x10;
    pub const INVALID_BAUD_RATE: u32 = 0x11;
    pub const INVALID_STOP_BIT: u32 = 0x12;
    pub const CODE_READ_PROTECTION_ENABLED: u32 = 0x13;
    pub const USER_CODE_CHECKSUM: u32 = 0x15;
    pub const EFRO_NO_POWER: u32 = 0x17;
    pub const FLASH_NO_POWER: u32 = 0x18;
    pub const FLASH_ERASE_PROGRAM: u32 = 0x20;
    pub const INVALID_PAGE: u32 = 0x21;
    pub const FLASH_NO_CLOCK: u32 = 0x1b;
    pub const REINVOKE_ISP_CONFIG: u32 = 0x1c;
    pub const NO_VALID_IMAGE: u32 = 0x1d;
}

type IapEntry = unsafe extern "C" fn(*mut u32, *mut u32);

type IapCommand = [u32; 5];
// results are typically only 4 long. the spec allows for up to 5 u32 cmd/response entries
type IapResult = [u32; 5];
type Status = u32;

const K_STATUS_SUCCESS: u32 = 0;

pub struct BootcodeVersion {
    pub major: u32,
    pub minor: u32,
}

/// Build up a command+response pair
const fn command(command: IapCommandNum) -> (IapCommand, IapResult) {
    let cmd: IapCommand = [command as u32, 0, 0, 0, 0];
    let result: IapResult = [0; 5];
    (cmd, result)
}

pub fn err_decode(e: u32) -> IapRetcode {
    use num_traits::FromPrimitive;
    IapRetcode::from_u32(e).unwrap_or_default()
}

pub trait Iap {
    fn new() -> Self;
    fn iap_address(&self) -> usize;

    fn chip_init(&self);

    fn addr_to_sector(&self, addr: u32) -> u32;

    /// Execute IAP command without disabling interrupts. Risky!
    fn iap_entry_no_disable_irq(
        &self,
        cmd: &mut IapCommand,
        result: &mut IapResult,
    ) -> Result<(), Status> {
        unsafe {
            // convert IAP address pointer into function pointer
            let iap_fn = core::mem::transmute::<usize, IapEntry>(self.iap_address());
            // convert our &mut u32's to *mut u32s and pass to fn
            iap_fn(cmd as *mut u32, result as *mut u32);
            // Return an error if we had one
            if result[0] == K_STATUS_SUCCESS {
                Ok(())
            } else {
                Err(result[0])
            }
        }
    }

    /// Execute IAP command with interrupts disabled
    fn iap_entry(&self, cmd: &mut IapCommand, result: &mut IapResult) -> Result<(), Status> {
        cortex_m::interrupt::free(|_| self.iap_entry_no_disable_irq(cmd, result))
    }

    /// Read unique identification.
    fn read_part_id(&self) -> Result<u32, Status> {
        let (mut cmd, mut result) = command(IapCommandNum::IAP_READ_PART_ID);
        self.iap_entry(&mut cmd, &mut result)?;

        Ok(result[1])
    }

    /// Read boot code version number.
    /// Note: Boot code version is two 32-bit words. Word 0 is the major version, word 1 is the minor version.
    fn read_boot_code_version(&self) -> Result<BootcodeVersion, Status> {
        let (mut cmd, mut result) = command(IapCommandNum::IAP_READ_BOOT_VER);
        self.iap_entry(&mut cmd, &mut result)?;

        Ok(BootcodeVersion {
            major: result[1],
            minor: result[2],
        })
    }

    /// Read unique identification.
    fn read_unique_id(&self) -> Result<(u32, u32, u32, u32), Status> {
        let (mut cmd, mut result) = command(IapCommandNum::IAP_READ_UID);
        self.iap_entry(&mut cmd, &mut result)?;

        Ok((result[1], result[2], result[3], result[4]))
    }

    /// Prepare sector for write operation.
    ///
    /// This function prepares sector(s) for write/erase operation. This function must be called before calling the
    /// IAP_CopyRamToFlash() or IAP_EraseSector() or IAP_ErasePage() function. The end sector number must be greater than or
    /// equal to the start sector number.
    fn prepare_sector_for_write(&self, start_sector: u32, end_sector: u32) -> Result<(), Status> {
        let (mut cmd, mut result) = command(IapCommandNum::IAP_PREPARE);
        cmd[1] = start_sector;
        cmd[2] = end_sector;
        self.iap_entry(&mut cmd, &mut result)?;

        Ok(())
    }

    /// Copy RAM to flash.
    ///
    /// This function programs the flash memory. Corresponding sectors must be prepared via IAP_PrepareSectorForWrite before
    /// calling this function
    /// All user code must bte written in such a way that no master accesses the flash
    /// while this command is executed and the flash is programmed.
    /// Note: system_core_clock_khz is in kilohertz (pass 1000 for a 1mhz clock)
    ///
    /// On lpc1114, dst_address must be on 256 byte boundary, and src_buf.len() must be 256/512/1024/4096
    fn copy_ram_to_flash(
        &self,
        dst_addr: u32,
        src_buf: &[u8],
        system_core_clock_khz: u32,
    ) -> Result<(), Status> {
        let (mut cmd, mut result) = command(IapCommandNum::IAP_COPY_RAM2FLASH);
        cmd[1] = dst_addr;
        cmd[2] = src_buf.as_ptr() as u32;
        cmd[3] = src_buf.len() as u32;
        // When flash controller has a fixed reference clock this is ignored
        cmd[4] = system_core_clock_khz;

        self.iap_entry(&mut cmd, &mut result)?;

        Ok(())
    }

    /// Compare memory contents of flash with ram.
    ///
    /// This function compares the contents of flash and ram. It can be used to verify the flash memory contents after
    /// IAP_CopyRamToFlash call.
    /// Note: system_core_clock_khz is in kilohertz (pass 1000 for a 1mhz clock)
    fn compare(
        &self,
        dst_addr: u32,
        src_buf: &[u8],
        system_core_clock_khz: u32,
    ) -> Result<(), Status> {
        let (mut cmd, mut result) = command(IapCommandNum::IAP_COMPARE);
        cmd[1] = dst_addr;
        cmd[2] = src_buf.as_ptr() as u32;
        cmd[3] = src_buf.len() as u32;
        // When flash controller has a fixed reference clock this is ignored
        cmd[4] = system_core_clock_khz;

        self.iap_entry(&mut cmd, &mut result)?;

        Ok(())
    }

    /// Erase sector.
    ///
    /// This function erases sector(s). The end sector number must be greater than or equal to the start sector number.
    /// Note: system_core_clock is in kilohertz (pass 1000 for a 1mhz clock)
    fn erase_sector(
        &self,
        start_sector: u32,
        end_sector: u32,
        system_core_clock_khz: u32,
    ) -> Result<(), Status> {
        let (mut cmd, mut result) = command(IapCommandNum::IAP_ERASE);
        cmd[1] = start_sector;
        cmd[2] = end_sector;
        // When flash controller has a fixed reference clock this is ignored
        cmd[3] = system_core_clock_khz;

        self.iap_entry(&mut cmd, &mut result)?;

        Ok(())
    }

    /// Erase page.
    ///
    /// This function erases page(s). The end page number must be greater than or equal to the start page number.
    /// Note: system_core_clock_khz is in kilohertz (pass 1000 for a 1mhz clock)
    fn erase_page(
        &self,
        start_page: u32,
        end_page: u32,
        system_core_clock_khz: u32,
    ) -> Result<(), Status> {
        let (mut cmd, mut result) = command(IapCommandNum::IAP_ERASE_PAGE);
        cmd[1] = start_page;
        cmd[2] = end_page;
        // system_core_clock is in hz, command wants khz
        // When flash controller has a fixed reference clock this is ignored
        cmd[3] = system_core_clock_khz;

        self.iap_entry(&mut cmd, &mut result)?;

        Ok(())
    }

    /// Blank check sector(s)
    ///
    /// Blank check single or multiples sectors of flash memory. The end sector number must be greater than or equal to the
    /// start sector number. It can be used to verify the sector erasure after IAP_EraseSector call.
    fn blank_check_sector(&self, start_sector: u32, end_sector: u32) -> Result<(), Status> {
        /*
         * @retval kStatus_IAP_Success One or more sectors are in erased state.
         * @retval kStatus_IAP_NoPower Flash memory block is powered down.
         * @retval kStatus_IAP_NoClock Flash memory block or controller is not clocked.
         * @retval kStatus_IAP_SectorNotblank One or more sectors are not blank.
         */
        let (mut cmd, mut result) = command(IapCommandNum::IAP_BLANK_CHECK);
        cmd[1] = start_sector;
        cmd[2] = end_sector;

        self.iap_entry(&mut cmd, &mut result)?;

        Ok(())
    }

    /// Initialise IAP ROM interface
    ///
    /// This is not really documented and not present for most micros.
    fn init(&self) -> Result<(), Status> {
        let (mut cmd, mut result) = command(IapCommandNum::IAP_INIT);
        self.iap_entry(&mut cmd, &mut result)?;

        Ok(())
    }
}
