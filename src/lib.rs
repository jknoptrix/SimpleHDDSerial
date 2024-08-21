#![warn(dead_code)]
mod structs;
mod utils;
mod ioctl_operations;

use structs::*;
use ioctl_operations::*;
use winapi::{
    shared::minwindef::BYTE,
    um::handleapi::CloseHandle,
};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct HardDriveSerial {
    serial_numbers: Vec<String>,
    errors: Vec<String>,
    timings: Vec<(usize, String, Duration)>, // (drive_index, timing_type, duration)
}

impl HardDriveSerial {
    pub fn new() -> Self {
        HardDriveSerial {
            serial_numbers: Vec::new(),
            errors: Vec::new(),
            timings: Vec::new(),
        }
    }

    // attempts to retrieve hard drive serial numbers using different methods
    // returns a Result containing a vector of serial numbers if successful, or a vector of error messages if not
    pub fn get_serial_numbers(&mut self) -> Result<Vec<String>, Vec<String>> {
        // create a vector of threads, one for each possible drive index
        let threads: Vec<_> = (0..MAX_IDE_DRIVES)
            .map(|i_drive| {
                let mut hd_serial = HardDriveSerial::new();
                // spawn a thread to read drive information for a specific drive index
                thread::spawn(move || {
                    hd_serial.read_physical_drive_in_nt_with_admin_rights(i_drive as BYTE);
                    hd_serial.read_physical_drive_in_nt_using_smart(i_drive as BYTE);
                    hd_serial.read_physical_drive_in_nt_with_zero_rights(i_drive as BYTE);
                    hd_serial
                })
            })
            .collect();

        // collect results from each thread, aggregating serial numbers, errors and timings
        for thread in threads {
            let mut hd_serial = thread.join().unwrap();
            self.serial_numbers.append(&mut hd_serial.serial_numbers);
            self.errors.append(&mut hd_serial.errors);
            self.timings.append(&mut hd_serial.timings);
        }

        // print execution timings for each drive and method
        for (drive_index, timing_type, duration) in &self.timings {
            println!("Drive {:#?}, Method {}: [{:#?}]", drive_index, timing_type, duration);
        }

        // return serial numbers if any were found, otherwise return the collected errors
        if self.serial_numbers.is_empty() {
            Err(self.errors.clone())
        } else {
            Ok(self.serial_numbers.clone())
        }
    }

    // attempts to retrieve the hard drive serial number using admin rights
    fn read_physical_drive_in_nt_with_admin_rights(&mut self, b_drive_num: BYTE) {
        let start_time = std::time::Instant::now(); // record the start time for performance measurement
        let drive_name = format!("\\\\.\\PhysicalDrive{}", b_drive_num);
        // attempt to open the drive with admin rights
        let h_physical_drive_ioctl = create_file_with_admin_rights(&drive_name);

        // check if the handle is valid
        if h_physical_drive_ioctl == INVALID_HANDLE_VALUE {
            return; // return early if the handle is invalid
        }

        // attempt to retrieve the serial number
        if let Some(serial) = get_drive_serial_with_admin_rights(h_physical_drive_ioctl, b_drive_num) {
            // validate the serial number
            if self.is_valid_serial_number(&serial) {
                self.serial_numbers.push(serial);
            } else {
                // log an error if the serial number is invalid
                let error_message = format!("Drive {}: Invalid serial number received with admin rights", b_drive_num);
                self.errors.push(error_message);
            }
        } else {
            // handle errors and log them
            let error_code = ioctl_operations::get_last_error();
            if error_code == 50 { // ERROR_NOT_SUPPORTED
                println!("Drive {}: Admin rights method not supported. Skipping.", b_drive_num);
            } else {
                let error_message = format!("Drive {}: Failed to get serial number with admin rights, error code: {}", b_drive_num, error_code);
                self.errors.push(error_message);
            }
        }

        // close the handle to the drive
        unsafe { CloseHandle(h_physical_drive_ioctl); }

        // record the end time and store the timing information
        let end_time = std::time::Instant::now();
        self.timings.push((b_drive_num as usize, format!("hd{}Trd#A", b_drive_num), end_time.duration_since(start_time)));
    }

    // attempts to retrieve the hard drive serial number using SMART
    fn read_physical_drive_in_nt_using_smart(&mut self, b_drive_num: BYTE) {
        let start_time = std::time::Instant::now(); // record the start time for performance measurement
        let drive_name = format!("\\\\.\\PhysicalDrive{}", b_drive_num);
        // attempt to open the drive with SMART rights
        let h_physical_drive_ioctl = create_file_with_smart_rights(&drive_name);

        // check if the handle is valid
        if h_physical_drive_ioctl == INVALID_HANDLE_VALUE {
            return; // return early if the handle is invalid
        }

        // attempt to retrieve the serial number using SMART
        if let Some(serial) = get_drive_serial_with_smart(h_physical_drive_ioctl, b_drive_num) {
            // validate the serial number
            if self.is_valid_serial_number(&serial) {
                self.serial_numbers.push(serial);
            } else {
                // log an error if the serial number is invalid
                let error_message = format!("Drive {}: Invalid serial number received using SMART", b_drive_num);
                self.errors.push(error_message);
            }
        } 

        // close the handle to the drive
        unsafe { CloseHandle(h_physical_drive_ioctl); }

        // record the end time and store the timing information
        let end_time = std::time::Instant::now();
        self.timings.push((b_drive_num as usize, format!("hd{}Trd#SM", b_drive_num), end_time.duration_since(start_time)));
    }

    // attempts to retrieve the hard drive serial number with zero rights
    fn read_physical_drive_in_nt_with_zero_rights(&mut self, b_drive_num: BYTE) {
        let start_time = std::time::Instant::now(); // record the start time for performance measurement
        let drive_name = format!("\\\\.\\PhysicalDrive{}", b_drive_num);
        // attempt to open the drive with zero rights
        let h_physical_drive_ioctl = create_file_with_zero_rights(&drive_name);

        // check if the handle is valid
        if h_physical_drive_ioctl == INVALID_HANDLE_VALUE {
            return; // return early if the handle is invalid
        }

        // attempt to retrieve the serial number with zero rights
        if let Some(serial) = get_drive_serial_with_zero_rights(h_physical_drive_ioctl) {
            // validate the serial number
            if self.is_valid_serial_number(&serial) {
                self.serial_numbers.push(serial);
            } else {
                // log an error if the serial number is invalid
                let error_message = format!("Drive {}: Invalid serial number received with zero rights", b_drive_num);
                self.errors.push(error_message);
            }
        }

        // close the handle to the drive
        unsafe { CloseHandle(h_physical_drive_ioctl); }

        // record the end time and store the timing information
        let end_time = std::time::Instant::now();
        self.timings.push((b_drive_num as usize, format!("hd{}Trd#Z", b_drive_num), end_time.duration_since(start_time)));
    }

    // validates the format of the serial number
    fn is_valid_serial_number(&self, serial: &str) -> bool {
        !serial.is_empty() && serial.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }
}