mod structs;
mod utils;
mod ioctl_operations;

use structs::*;
use ioctl_operations::*;
use winapi::{
    shared::minwindef::BYTE,
    um::handleapi::CloseHandle
};

#[derive(Debug)]
pub struct HardDriveSerial {
    serial_number: String,
    error_message: String,
}

impl HardDriveSerial {
    pub fn new() -> Self {
        HardDriveSerial {
            serial_number: String::new(),
            error_message: String::new(),
        }
    }

    pub fn get_serial_number(&mut self) -> Result<String, String> {
        if self.read_physical_drive_in_nt_with_admin_rights() {
            return Ok(self.serial_number.clone());
        }
        if self.read_physical_drive_in_nt_using_smart() {
            return Ok(self.serial_number.clone());
        }
        if self.read_physical_drive_in_nt_with_zero_rights() {
            return Ok(self.serial_number.clone());
        }

        Err(self.error_message.clone())
    }

    // --- Internal Methods (Refactored for clarity) ---

    fn read_physical_drive_in_nt_with_admin_rights(&mut self) -> bool {
        for i_drive in 0..MAX_IDE_DRIVES {
            let drive_name = format!("\\\\.\\PhysicalDrive{}", i_drive);
            let h_physical_drive_ioctl = create_file_with_admin_rights(&drive_name);

            if h_physical_drive_ioctl == INVALID_HANDLE_VALUE {
                continue; // Error handling moved to create_file_with_admin_rights
            }

            if let Some(serial) = get_drive_serial_with_admin_rights(h_physical_drive_ioctl, i_drive as BYTE) {
                self.serial_number = serial;
                unsafe { CloseHandle(h_physical_drive_ioctl); }
                return true;
            }

            unsafe { CloseHandle(h_physical_drive_ioctl); }
        }

        false
    }

    fn read_physical_drive_in_nt_using_smart(&mut self) -> bool {
        for i_drive in 0..MAX_IDE_DRIVES {
            let drive_name = format!("\\\\.\\PhysicalDrive{}", i_drive);
            let h_physical_drive_ioctl = create_file_with_smart_rights(&drive_name);

            if h_physical_drive_ioctl == INVALID_HANDLE_VALUE {
                continue; // Error handling moved to create_file_with_smart_rights
            }

            if let Some(serial) = get_drive_serial_with_smart(h_physical_drive_ioctl, i_drive as BYTE) {
                self.serial_number = serial;
                unsafe { CloseHandle(h_physical_drive_ioctl); }
                return true;
            }

            unsafe { CloseHandle(h_physical_drive_ioctl); }
        }

        false
    }

    fn read_physical_drive_in_nt_with_zero_rights(&mut self) -> bool {
        for i_drive in 0..MAX_IDE_DRIVES {
            let drive_name = format!("\\\\.\\PhysicalDrive{}", i_drive);
            let h_physical_drive_ioctl = create_file_with_zero_rights(&drive_name);

            if h_physical_drive_ioctl == INVALID_HANDLE_VALUE {
                continue; // Error handling moved to create_file_with_zero_rights
            }

            if let Some(serial) = get_drive_serial_with_zero_rights(h_physical_drive_ioctl) {
                self.serial_number = serial;
                unsafe { CloseHandle(h_physical_drive_ioctl); }
                return true;
            }

            unsafe { CloseHandle(h_physical_drive_ioctl); }
        }

        false
    }
}
