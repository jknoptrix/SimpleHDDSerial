use crate::structs::*;
use crate::utils::*;
use std::mem::zeroed;
use std::ptr::null_mut;
use std::mem;
use winapi::{
    shared::{
        minwindef::{DWORD, BYTE},
        ntdef::ULONG,
    },
    um::{
        winioctl::{
            METHOD_BUFFERED, CTL_CODE, IOCTL_DISK_BASE, FILE_READ_ACCESS, FILE_WRITE_ACCESS,
            IOCTL_STORAGE_QUERY_PROPERTY, StorageDeviceProperty, PropertyStandardQuery,
            STORAGE_PROPERTY_QUERY, IOCTL_DISK_GET_DRIVE_GEOMETRY_EX, IOCTL_DISK_GET_DRIVE_GEOMETRY,
        },
        winnt::{
            GENERIC_READ, GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE,
            HANDLE, FILE_SHARE_DELETE,
        },
        fileapi::{CreateFileA, OPEN_EXISTING},
        ioapiset::DeviceIoControl,
        errhandlingapi::GetLastError,
        synchapi::{WaitForSingleObject, CreateEventA},
        minwinbase::OVERLAPPED,
        winbase::{INFINITE, FILE_FLAG_OVERLAPPED, WAIT_OBJECT_0},
    },
};

use winapi::shared::winerror::ERROR_IO_PENDING;

use lazy_static::lazy_static;

lazy_static! {
    static ref SMART_GET_VERSION: DWORD = CTL_CODE(IOCTL_DISK_BASE, 0x0020, METHOD_BUFFERED, FILE_READ_ACCESS);                        // control code for getting SMART version
    static ref SMART_RCV_DRIVE_DATA: DWORD = CTL_CODE(IOCTL_DISK_BASE, 0x0022, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS); // control code for receiving SMART data
}


pub fn create_file_with_admin_rights(drive_name: &str) -> HANDLE {
    unsafe {
        CreateFileA(
            drive_name.as_ptr() as *const i8,      // drive name to open
            GENERIC_READ | GENERIC_WRITE,     // desired access (read and write)
            FILE_SHARE_READ | FILE_SHARE_WRITE,   // share mode (read and write)
            null_mut(),                  // security attributes (none)
            OPEN_EXISTING,              // open only if it exists
            FILE_FLAG_OVERLAPPED,        // overlapped operation flag
            null_mut(),                         // template file (none)
        )
    }
}

pub fn create_file_with_smart_rights(drive_name: &str) -> HANDLE {
    unsafe {
        CreateFileA(
            drive_name.as_ptr() as *const i8,                        // drive name
            GENERIC_READ | GENERIC_WRITE,                       // read and write access
            FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE, // sharing permissions
            null_mut(),                                    // no security attributes
            OPEN_EXISTING,                                // open only existing
            FILE_FLAG_OVERLAPPED,                          // overlapped i/o
            null_mut(),                                           // no template file
        )
    }
}

pub fn create_file_with_zero_rights(drive_name: &str) -> HANDLE {
    unsafe {
        CreateFileA(
            drive_name.as_ptr() as *const i8,    // name of the drive
            0,                              // no access rights
            FILE_SHARE_READ | FILE_SHARE_WRITE, // share read and write
            null_mut(),                // default security
            OPEN_EXISTING,            // open only if it exists
            FILE_FLAG_OVERLAPPED,      // use overlapped i/o
            null_mut(),                       // no template file
        )
    }
}

pub fn get_drive_serial_with_admin_rights(h_physical_drive_ioctl: HANDLE, b_drive_num: BYTE) -> Option<String> {
    let mut version_params: GETVERSIONOUTPARAMS = unsafe { zeroed() };                   // initialize the struct with zeros
    let mut dw_bytes_returned: DWORD = 0;                                                // initialize the number of bytes returned
    let mut overlapped: OVERLAPPED = unsafe { zeroed() };                                // initialize the overlapped struct with zeros
    overlapped.hEvent = unsafe { 
        CreateEventA(
            null_mut(), 1, 0, null_mut()    // create an event for overlapped operation
        ) 
    };

    let result = unsafe {
        DeviceIoControl(
            h_physical_drive_ioctl,                                           // handle to the drive
            DFP_GET_VERSION,                                          // control code for getting the version
            null_mut(),                                                    // no input buffer
            0,                                                          // size of input buffer
            &mut version_params as *mut _ as *mut winapi::ctypes::c_void, // output buffer
            mem::size_of::<GETVERSIONOUTPARAMS>() as DWORD,            // size of output buffer
            &mut dw_bytes_returned,                                   // number of bytes returned
            &mut overlapped,                                             // overlapped structure
        )
    };

    if result == 0 {
        let error = unsafe { GetLastError() };
        if error == ERROR_IO_PENDING {
            if unsafe { WaitForSingleObject(overlapped.hEvent, INFINITE) } != WAIT_OBJECT_0 { // wait for the overlapped operation to complete
                return None;
            }
        } else {
            return None;
        }
    }

    if version_params.b_ide_device_map <= 0 {
        return None;
    }

    let mut scip: SENDCMDINPARAMS = unsafe { zeroed() }; // input parameters for sending a command
    let mut by_id_out_cmd: SENDCMDOUTPARAMS = unsafe { zeroed() }; // output parameters for sending a command

    if do_identify(
        h_physical_drive_ioctl,    // handle to the drive
        &mut scip,          // input parameters
        &mut by_id_out_cmd, // output parameters
        IDE_ATA_IDENTIFY, // command to identify the drive
        b_drive_num,                // drive number
        &mut dw_bytes_returned, // number of bytes returned
    ) {
        return match extract_serial_from_identify_data(&by_id_out_cmd.b_buffer) { // extract the serial from the identify data
            Ok(serial) => Some(serial),
            Err(_) => None,
        };
    }

    None
}

pub fn get_drive_serial_with_smart(h_physical_drive_ioctl: HANDLE, _b_drive_num: BYTE) -> Option<String> {
    let mut get_version_params: GETVERSIONINPARAMS = unsafe { zeroed() }; // initialize with zeros
    let mut dw_bytes_returned: DWORD = 0; // initialize bytes returned
    let mut overlapped: OVERLAPPED = unsafe { zeroed() }; // initialize overlapped struct
    overlapped.hEvent = unsafe { CreateEventA(null_mut(), 1, 0, null_mut()) }; // create an event

    let result = unsafe {
        DeviceIoControl(
            h_physical_drive_ioctl, // handle to drive
            *SMART_GET_VERSION, // control code for SMART get version
            null_mut(), // no input data
            0, // input data size
            &mut get_version_params as *mut _ as *mut winapi::ctypes::c_void, // output buffer
            mem::size_of::<GETVERSIONINPARAMS>() as DWORD, // output buffer size
            &mut dw_bytes_returned, // bytes returned
            &mut overlapped, // overlapped struct
        )
    };

    if result == 0 {
        let error = unsafe { GetLastError() }; // get last error if operation failed
        if error == ERROR_IO_PENDING {
            if unsafe { WaitForSingleObject(overlapped.hEvent, INFINITE) } != WAIT_OBJECT_0 { // wait for overlapped operation to complete
                return None;
            }
        } else {
            return None; // return none if operation failed
        }
    }

    let command_size: ULONG =
        (mem::size_of::<SENDCMDINPARAMS>() + IDENTIFY_BUFFER_SIZE) as ULONG; // calculate command size
    let mut command: SENDCMDINPARAMS = unsafe { zeroed() }; // initialize command struct with zeros
    command.ir_drive_regs.b_command_reg = IDE_ATA_IDENTIFY; // set the command to identify

    let result = unsafe {
        DeviceIoControl(
            h_physical_drive_ioctl, // handle to the drive
            *SMART_RCV_DRIVE_DATA, // control code for receiving SMART data
            &mut command as *mut _ as *mut winapi::ctypes::c_void, // input buffer (command)
            mem::size_of::<SENDCMDINPARAMS>() as DWORD, // size of input buffer
            &mut command as *mut _ as *mut winapi::ctypes::c_void, // output buffer (command)
            command_size as DWORD, // size of output buffer
            &mut dw_bytes_returned, // bytes returned
            &mut overlapped, // overlapped struct
        )
    };

    if result == 0 {
        let error = unsafe { GetLastError() }; // check for error
        if error == ERROR_IO_PENDING { // check if pending
            if unsafe { WaitForSingleObject(overlapped.hEvent, INFINITE) } != WAIT_OBJECT_0 { // wait if pending
                return None; // return None if wait fails
            }
        } else {
            return None; // return None if other errors occur
        }
    } 

    return match extract_serial_from_identify_data(&command.c_buffer) { // try extracting serial from identify data
        Ok(serial) => Some(serial),
        Err(_) => None,
    };
}

pub fn get_drive_serial_with_zero_rights(h_physical_drive_ioctl: HANDLE) -> Option<String> {
    let mut dw_bytes_returned: DWORD = 0; // initialize bytes returned to 0
    let mut overlapped: OVERLAPPED = unsafe { zeroed() }; // zero the overlapped structure
    overlapped.hEvent = unsafe { CreateEventA(null_mut(), 1, 0, null_mut()) }; // create event for overlapped operation

    // First try to get serial using STORAGE_PROPERTY_QUERY
    let mut query: STORAGE_PROPERTY_QUERY = unsafe { zeroed() }; // initialize the query structure
    query.PropertyId = StorageDeviceProperty; // we want the device property
    query.QueryType = PropertyStandardQuery; // we'll do a standard query

    let mut buffer: [BYTE; 10000] = [0; 10000]; // buffer to receive the query results

    let result = unsafe {
        DeviceIoControl(
            h_physical_drive_ioctl, // handle to the drive
            IOCTL_STORAGE_QUERY_PROPERTY, // control code to query properties
            &mut query as *mut _ as *mut winapi::ctypes::c_void, // input buffer (our query)
            mem::size_of::<STORAGE_PROPERTY_QUERY>() as DWORD, // size of input buffer
            &mut buffer as *mut _ as *mut winapi::ctypes::c_void, // output buffer
            mem::size_of_val(&buffer) as DWORD, // size of output buffer
            &mut dw_bytes_returned, // number of bytes returned
            &mut overlapped, // overlapped struct
        )
    };

    if result != 0  || unsafe{ GetLastError() } == ERROR_IO_PENDING {
        if result == 0 {
            if unsafe { WaitForSingleObject(overlapped.hEvent, INFINITE) } != WAIT_OBJECT_0 { // wait for operation to complete
                return None;
            }
        }

        let descrip: *const STORAGE_DEVICE_DESCRIPTOR = buffer.as_ptr() as *const STORAGE_DEVICE_DESCRIPTOR; // cast buffer to descriptor pointer
        let mut serial_buffer: [BYTE; 1000] = [0; 1000]; // create buffer for serial number

        flip_and_code_bytes(
            unsafe { (*descrip).serial_number_offset } as usize, // get offset of serial number
            0, // no flipping needed
            &buffer, // buffer with the query results
            &mut serial_buffer, // buffer for serial number
        );

        let serial_number = String::from_utf8_lossy(&serial_buffer) // create a string from the serial buffer
            .trim_end_matches(char::from(0))
            .to_string();

        if serial_number.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') { // check if serial number contains only valid characters
            return Some(serial_number); // return the serial number if valid
        }
    }

    // If STORAGE_PROPERTY_QUERY fails, try using IOCTL_DISK_GET_DRIVE_GEOMETRY_EX
    let mut geometry_ex: DISK_GEOMETRY_EX = unsafe { zeroed() }; // initialize the geometry structure
    let result = unsafe {
        DeviceIoControl(
            h_physical_drive_ioctl, // handle to the drive
            IOCTL_DISK_GET_DRIVE_GEOMETRY_EX, // control code to get geometry
            null_mut(), // no input data
            0, // input data size
            &mut geometry_ex as *mut _ as *mut winapi::ctypes::c_void, // output buffer
            mem::size_of::<DISK_GEOMETRY_EX>() as DWORD, // size of output buffer
            &mut dw_bytes_returned, // number of bytes returned
            &mut overlapped, // overlapped structure
        )
    };

    if result == 0 {
        let error = unsafe { GetLastError() }; // get last error
        if error == ERROR_IO_PENDING {
            if unsafe { WaitForSingleObject(overlapped.hEvent, INFINITE) } != WAIT_OBJECT_0 { // wait for operation to finish
                return None;
            }
        } else {
            return None; // return None if other errors occured
        }
    }

    // If IOCTL_DISK_GET_DRIVE_GEOMETRY_EX fails, try using IOCTL_DISK_GET_DRIVE_GEOMETRY
    let mut geometry: DISK_GEOMETRY = unsafe { zeroed() }; // initialize the geometry struct
    let result = unsafe {
        DeviceIoControl(
            h_physical_drive_ioctl, // handle to drive
            IOCTL_DISK_GET_DRIVE_GEOMETRY, // ioctl to get drive geometry
            null_mut(), // no input data
            0, // no input data size
            &mut geometry as *mut _ as *mut winapi::ctypes::c_void, // output buffer
            mem::size_of::<DISK_GEOMETRY>() as DWORD, // output buffer size
            &mut dw_bytes_returned, // bytes returned
            &mut overlapped, // overlapped struct
        )
    };

    if result == 0 {
        let error = unsafe { GetLastError() }; // get the last error
        if error == ERROR_IO_PENDING {
            if unsafe { WaitForSingleObject(overlapped.hEvent, INFINITE) } != WAIT_OBJECT_0 { // wait for the operation to complete
                return None;
            }
        } else {
            return None; // return None if another error happened
        }
    }

    None
}

pub fn get_last_error() -> DWORD {
    unsafe { GetLastError() } // just return the last error code
}