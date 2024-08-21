#![warn(dead_code)]
use crate::structs::*;
use std::io;
use std::mem;
use winapi::shared::minwindef::{DWORD, BYTE, ULONG};
use winapi::um::winnt::HANDLE;
use winapi::um::ioapiset::DeviceIoControl;
use byteorder::{ReadBytesExt, LittleEndian};
use std::ptr::null_mut;

// converts a slice of DWORDs to a string, handling potential whitespace and null characters
pub fn convert_to_string(
    dw_disk_data: &[DWORD],
    i_first_index: usize,
    i_last_index: usize,
    pcsz_buf: &mut [BYTE],
) {
    let mut i_position = 0;

    // iterate over the DWORD slice, extracting characters and building the string
    for i_index in i_first_index..=i_last_index {
        let c_temp = (dw_disk_data[i_index] / 256) as BYTE;
        if c_temp == 0 { break; } // exit the loop if a null character is encountered
        if c_temp != b' ' {
            pcsz_buf[i_position] = c_temp;
            i_position += 1;
        }

        let c_temp1 = (dw_disk_data[i_index] % 256) as BYTE;
        if c_temp1 == 0 { break; } // exit the loop if a null character is encountered
        if c_temp1 != b' ' {
            pcsz_buf[i_position] = c_temp1;
            i_position += 1;
        }
    }

    pcsz_buf[i_position] = 0; // null-terminate the string

    // remove trailing whitespace from the string
    for i_index in (0..i_position).rev() {
        if pcsz_buf[i_index].is_ascii_whitespace() {
            pcsz_buf[i_index] = 0;
        } else {
            break;
        }
    }
}


// flips and decodes bytes in a string, handling different encoding possibilities
pub fn flip_and_code_bytes(
    i_pos: usize,
    i_flip: i32,
    pcsz_str: &[BYTE],
    pcsz_buf: &mut [BYTE],
) {
    if i_pos <= 0 {
        return;
    }

    let mut i_j = 1;
    let mut i_k = 0;

    // attempt to decode the string assuming hexadecimal encoding
    for i_i in i_pos..pcsz_str.len() {
        if pcsz_str[i_i] == 0 {
            break;
        }

        let c_c = pcsz_str[i_i].to_ascii_lowercase();
        let c_c = if c_c.is_ascii_whitespace() { b'0' } else { c_c };

        pcsz_buf[i_k] <<= 4;

        if c_c >= b'0' && c_c <= b'9' {
            pcsz_buf[i_k] |= c_c - b'0';
        } else if c_c >= b'a' && c_c <= b'f' {
            pcsz_buf[i_k] |= c_c - b'a' + 10;
        } else {
            i_j = 0;
            break;
        }

        if i_j == 2 {
            if pcsz_buf[i_k] != 0 && !pcsz_buf[i_k].is_ascii_graphic() {
                i_j = 0;
                break;
            }
            i_k += 1;
            i_j = 0;
        }
    }

    // if hexadecimal decoding failed, attempt to decode as ASCII
    if i_j == 0 {
        i_j = 1;
        i_k = 0;
        for i_i in i_pos..pcsz_str.len() {
            if pcsz_str[i_i] == 0 {
                break;
            }

            let c_c = pcsz_str[i_i];

            if !c_c.is_ascii_graphic() {
                i_j = 0;
                break;
            }

            pcsz_buf[i_k] = c_c;
            i_k += 1;
        }
    }

    // flip bytes if requested
    if i_flip != 0 {
        for i_j in (0..i_k).step_by(2) {
            pcsz_buf.swap(i_j, i_j + 1);
        }
    }

    // remove leading and trailing whitespace
    let mut i_i = -1;
    let mut i_j = -1;

    for i_k in 0..pcsz_buf.len() {
        if pcsz_buf[i_k] != 0 && !pcsz_buf[i_k].is_ascii_whitespace() {
            if i_i == -1 {
                i_i = i_k as i32;
            }
            i_j = i_k as i32;
        }
    }

    if i_i >= 0 && i_j >= 0 {
        for i_k in i_i..=i_j {
            if pcsz_buf[i_k as usize] == 0 {
                break;
            }
            pcsz_buf[(i_k - i_i) as usize] = pcsz_buf[i_k as usize];
        }
        pcsz_buf[(i_j - i_i + 1) as usize] = 0;
    }
}


// sends an IDENTIFY DEVICE command to the drive
pub fn do_identify(
    h_physical_drive_ioctl: HANDLE,
    p_scip: &mut SENDCMDINPARAMS,
    p_scop: &mut SENDCMDOUTPARAMS,
    b_id_cmd: BYTE,
    b_drive_num: BYTE,
    lpcb_bytes_returned: &mut DWORD,
) -> bool {
    // set up the command parameters
    p_scip.c_buffer_size = IDENTIFY_BUFFER_SIZE as ULONG;
    p_scip.ir_drive_regs.b_features_reg = 0;
    p_scip.ir_drive_regs.b_sector_count_reg = 1;
    p_scip.ir_drive_regs.b_cyl_low_reg = 0;
    p_scip.ir_drive_regs.b_cyl_high_reg = 0;

    p_scip.ir_drive_regs.b_drive_head_reg = 0xA0 | ((b_drive_num & 1) << 4);

    p_scip.ir_drive_regs.b_command_reg = b_id_cmd;
    p_scip.b_drive_number = b_drive_num;
    p_scip.c_buffer_size = IDENTIFY_BUFFER_SIZE as ULONG;

    // send the command to the drive using DeviceIoControl
    unsafe {
        DeviceIoControl(
            h_physical_drive_ioctl,
            DFP_RECEIVE_DRIVE_DATA,
            p_scip as *mut _ as *mut winapi::ctypes::c_void,
            (mem::size_of::<SENDCMDINPARAMS>() - 1) as DWORD,
            p_scop as *mut _ as *mut winapi::ctypes::c_void,
            (mem::size_of::<SENDCMDOUTPARAMS>() + IDENTIFY_BUFFER_SIZE - 1) as DWORD,
            lpcb_bytes_returned,
            null_mut(),
        ) != 0
    }
}

// extracts the serial number from the IDENTIFY DEVICE data
pub fn extract_serial_from_identify_data(identify_data: &[BYTE]) -> io::Result<String> {
    // create a reader for the identify data
    let mut reader = io::Cursor::new(identify_data);

    // read the data into a DWORD array
    let mut dw_disk_data: [DWORD; 256] = [0; 256];
    for i_ijk in 0..256 {
        dw_disk_data[i_ijk] = reader.read_u16::<LittleEndian>()?.into();
    }

    // convert the relevant portion of the DWORD array to a string
    let mut csz_serial_number: [BYTE; 1024] = [0; 1024];
    convert_to_string(&dw_disk_data, 10, 19, &mut csz_serial_number);

    // create a string from the resulting byte array, removing trailing null characters
    let serial_number = String::from_utf8_lossy(&csz_serial_number)
        .trim_end_matches(char::from(0))
        .to_string();

    // return an error if the serial number is empty
    if serial_number.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to extract serial number"));
    }

    Ok(serial_number) // return the extracted serial number
}