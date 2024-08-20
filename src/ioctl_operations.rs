use crate::structs::*;
use crate::utils::*;
use std::mem::zeroed;
use std::ptr::null_mut;
use std::mem;
use winapi::{
        shared::{
            minwindef::{ DWORD, BYTE },
            ntdef::ULONG,
        },
        um::{
            winioctl::{
                METHOD_BUFFERED, CTL_CODE, IOCTL_DISK_BASE, FILE_READ_ACCESS, FILE_WRITE_ACCESS, IOCTL_STORAGE_QUERY_PROPERTY,
                StorageDeviceProperty, PropertyStandardQuery,
                STORAGE_PROPERTY_QUERY, IOCTL_DISK_GET_DRIVE_GEOMETRY_EX
            },
            winnt::{
                GENERIC_READ, GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE,
                HANDLE, FILE_SHARE_DELETE
            },
            fileapi::{ CreateFileA, OPEN_EXISTING },
            ioapiset::DeviceIoControl
        },
};

use lazy_static::lazy_static;

lazy_static! {
    static ref SMART_GET_VERSION: DWORD = CTL_CODE(IOCTL_DISK_BASE, 0x0020, METHOD_BUFFERED, FILE_READ_ACCESS);
    static ref SMART_RCV_DRIVE_DATA: DWORD = CTL_CODE(IOCTL_DISK_BASE, 0x0022, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);
}


pub fn create_file_with_admin_rights(drive_name: &str) -> HANDLE {
    unsafe {
        CreateFileA(
            drive_name.as_ptr() as *const i8,
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        )
    }
}

pub fn create_file_with_smart_rights(drive_name: &str) -> HANDLE {
    unsafe {
        CreateFileA(
            drive_name.as_ptr() as *const i8,
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_DELETE | FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        )
    }
}

pub fn create_file_with_zero_rights(drive_name: &str) -> HANDLE {
    unsafe {
        CreateFileA(
            drive_name.as_ptr() as *const i8,
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        )
    }
}

pub fn get_drive_serial_with_admin_rights(h_physical_drive_ioctl: HANDLE, b_drive_num: BYTE) -> Option<String> {
    let mut version_params: GETVERSIONOUTPARAMS = unsafe { zeroed() };
    let mut dw_bytes_returned: DWORD = 0;

    let result = unsafe {
        DeviceIoControl(
            h_physical_drive_ioctl,
            DFP_GET_VERSION,
            null_mut(),
            0,
            &mut version_params as *mut _ as *mut winapi::ctypes::c_void,
            mem::size_of::<GETVERSIONOUTPARAMS>() as DWORD,
            &mut dw_bytes_returned,
            null_mut(),
        )
    };

    if result == 0 {
        return None;
    }

    if version_params.b_ide_device_map <= 0 {
        return None;
    }

    let mut scip: SENDCMDINPARAMS = unsafe { zeroed() };
    let mut by_id_out_cmd: SENDCMDOUTPARAMS = unsafe { zeroed() };

    if do_identify(
        h_physical_drive_ioctl,
        &mut scip,
        &mut by_id_out_cmd,
        IDE_ATA_IDENTIFY,
        b_drive_num,
        &mut dw_bytes_returned,
    ) {
        return match extract_serial_from_identify_data(&by_id_out_cmd.b_buffer) {
            Ok(serial) => Some(serial),
            Err(_) => None,
        };
    }

    None
}

pub fn get_drive_serial_with_smart(h_physical_drive_ioctl: HANDLE, _b_drive_num: BYTE) -> Option<String> {
    let mut get_version_params: GETVERSIONINPARAMS = unsafe { zeroed() };
    let mut dw_bytes_returned: DWORD = 0;

    let result = unsafe {
        DeviceIoControl(
            h_physical_drive_ioctl,
            *SMART_GET_VERSION,
            null_mut(),
            0,
            &mut get_version_params as *mut _ as *mut winapi::ctypes::c_void,
            mem::size_of::<GETVERSIONINPARAMS>() as DWORD,
            &mut dw_bytes_returned,
            null_mut(),
        )
    };

    if result == 0 {
        return None;
    }

    let command_size: ULONG =
        (mem::size_of::<SENDCMDINPARAMS>() + IDENTIFY_BUFFER_SIZE) as ULONG;
    let mut command: SENDCMDINPARAMS = unsafe { zeroed() };
    command.ir_drive_regs.b_command_reg = IDE_ATA_IDENTIFY;

    let result = unsafe {
        DeviceIoControl(
            h_physical_drive_ioctl,
            *SMART_RCV_DRIVE_DATA,
            &mut command as *mut _ as *mut winapi::ctypes::c_void,
            mem::size_of::<SENDCMDINPARAMS>() as DWORD,
            &mut command as *mut _ as *mut winapi::ctypes::c_void,
            command_size as DWORD,
            &mut dw_bytes_returned,
            null_mut(),
        )
    };

    if result == 0 {
        return None;
    } else {
        return match extract_serial_from_identify_data(&command.c_buffer) {
            Ok(serial) => Some(serial),
            Err(_) => None,
        };
    }
}

pub fn get_drive_serial_with_zero_rights(h_physical_drive_ioctl: HANDLE) -> Option<String> {
    let mut query: STORAGE_PROPERTY_QUERY = unsafe { zeroed() };
    query.PropertyId = StorageDeviceProperty;
    query.QueryType = PropertyStandardQuery;

    let mut dw_bytes_returned: DWORD = 0;
    let mut csz_buffer: [BYTE; 10000] = [0; 10000];

    let result = unsafe {
        DeviceIoControl(
            h_physical_drive_ioctl,
            IOCTL_STORAGE_QUERY_PROPERTY,
            &mut query as *mut _ as *mut winapi::ctypes::c_void,
            mem::size_of::<STORAGE_PROPERTY_QUERY>() as DWORD,
            &mut csz_buffer as *mut _ as *mut winapi::ctypes::c_void,
            mem::size_of_val(&csz_buffer) as DWORD,
            &mut dw_bytes_returned,
            null_mut(),
        )
    };

    if result != 0 {
        let descrip: *const STORAGE_DEVICE_DESCRIPTOR = csz_buffer.as_ptr() as *const STORAGE_DEVICE_DESCRIPTOR;
        let mut csz_serial_number: [BYTE; 1000] = [0; 1000];

        flip_and_code_bytes(
            unsafe { (*descrip).serial_number_offset } as usize,
            0,
            &csz_buffer,
            &mut csz_serial_number,
        );

        let serial_number = String::from_utf8_lossy(&csz_serial_number)
            .trim_end_matches(char::from(0))
            .to_string();

        if csz_serial_number[0].is_ascii_alphanumeric()
            || csz_serial_number[19].is_ascii_alphanumeric()
        {
            return Some(serial_number);
        }

        let mut csz_buffer: [BYTE; 10000] = [0; 10000];
        let result = unsafe {
            DeviceIoControl(
                h_physical_drive_ioctl,
                IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
                null_mut(),
                0,
                &mut csz_buffer as *mut _ as *mut winapi::ctypes::c_void,
                mem::size_of_val(&csz_buffer) as DWORD,
                &mut dw_bytes_returned,
                null_mut(),
            )
        };

        if result == 0 {
            return None;
        }
    } else {
        return None;
    }

    None
}