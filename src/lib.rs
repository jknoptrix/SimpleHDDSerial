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
    serial_numbers: Vec<String>,
    error_message: String,
}

impl HardDriveSerial {
    pub fn new() -> Self {
        HardDriveSerial {
            serial_numbers: Vec::new(),
            error_message: String::new(),
        }
    }

    pub fn get_serial_numbers(&mut self) -> Result<Vec<String>, String> {
        self.read_physical_drive_in_nt_with_admin_rights();
        self.read_physical_drive_in_nt_using_smart();
        self.read_physical_drive_in_nt_with_zero_rights();

        if self.serial_numbers.is_empty() {
            Err(self.error_message.clone())
        } else {
            Ok(self.serial_numbers.clone())
        }
    }


    fn read_physical_drive_in_nt_with_admin_rights(&mut self) {
        for i_drive in 0..MAX_IDE_DRIVES {
            let drive_name = format!("\\\\.\\PhysicalDrive{}", i_drive);
            let h_physical_drive_ioctl = create_file_with_admin_rights(&drive_name);

            if h_physical_drive_ioctl == INVALID_HANDLE_VALUE {
                continue;
            }

            if let Some(serial) = get_drive_serial_with_admin_rights(h_physical_drive_ioctl, i_drive as BYTE) {
                self.serial_numbers.push(serial);
            }

            unsafe { CloseHandle(h_physical_drive_ioctl); }
        }
    }

    fn read_physical_drive_in_nt_using_smart(&mut self) {
        for i_drive in 0..MAX_IDE_DRIVES {
            let drive_name = format!("\\\\.\\PhysicalDrive{}", i_drive);
            let h_physical_drive_ioctl = create_file_with_smart_rights(&drive_name);

            if h_physical_drive_ioctl == INVALID_HANDLE_VALUE {
                continue;
            }

            if let Some(serial) = get_drive_serial_with_smart(h_physical_drive_ioctl, i_drive as BYTE) {
                self.serial_numbers.push(serial);
            }

            unsafe { CloseHandle(h_physical_drive_ioctl); }
        }
    }

    fn read_physical_drive_in_nt_with_zero_rights(&mut self) {
        for i_drive in 0..MAX_IDE_DRIVES {
            let drive_name = format!("\\\\.\\PhysicalDrive{}", i_drive);
            let h_physical_drive_ioctl = create_file_with_zero_rights(&drive_name);

            if h_physical_drive_ioctl == INVALID_HANDLE_VALUE {
                continue; 
            }

            if let Some(serial) = get_drive_serial_with_zero_rights(h_physical_drive_ioctl) {
                self.serial_numbers.push(serial);
            }

            unsafe { CloseHandle(h_physical_drive_ioctl); }
        }
    }
}