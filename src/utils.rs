use crate::structs::*;
use std::io;
use std::mem;
use winapi::shared::minwindef::{DWORD, BYTE, ULONG};
use winapi::um::winnt::HANDLE;
use winapi::um::ioapiset::DeviceIoControl;
use byteorder::{ReadBytesExt, LittleEndian};
use std::ptr::null_mut;

pub fn convert_to_string(
    dw_disk_data: &[DWORD],
    i_first_index: usize,
    i_last_index: usize,
    pcsz_buf: &mut [BYTE],
) {
    let mut i_position = 0;

    for i_index in i_first_index..=i_last_index {
        let c_temp = (dw_disk_data[i_index] / 256) as BYTE;
        if c_temp == 0 { break; } // Added exit from the loop when a null character is found
        if c_temp != b' ' {
            pcsz_buf[i_position] = c_temp;
            i_position += 1;
        }

        let c_temp1 = (dw_disk_data[i_index] % 256) as BYTE;
        if c_temp1 == 0 { break; } // Added exit from the loop when a null character is found
        if c_temp1 != b' ' {
            pcsz_buf[i_position] = c_temp1;
            i_position += 1;
        }
    }

    pcsz_buf[i_position] = 0;

    for i_index in (0..i_position).rev() {
        if pcsz_buf[i_index].is_ascii_whitespace() {
            pcsz_buf[i_index] = 0;
        } else {
            break;
        }
    }
}


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

    if i_flip != 0 {
        for i_j in (0..i_k).step_by(2) {
            pcsz_buf.swap(i_j, i_j + 1);
        }
    }

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


pub fn do_identify(
    h_physical_drive_ioctl: HANDLE,
    p_scip: &mut SENDCMDINPARAMS,
    p_scop: &mut SENDCMDOUTPARAMS,
    b_id_cmd: BYTE,
    b_drive_num: BYTE,
    lpcb_bytes_returned: &mut DWORD,
) -> bool {
    p_scip.c_buffer_size = IDENTIFY_BUFFER_SIZE as ULONG;
    p_scip.ir_drive_regs.b_features_reg = 0;
    p_scip.ir_drive_regs.b_sector_count_reg = 1;
    p_scip.ir_drive_regs.b_cyl_low_reg = 0;
    p_scip.ir_drive_regs.b_cyl_high_reg = 0;

    p_scip.ir_drive_regs.b_drive_head_reg = 0xA0 | ((b_drive_num & 1) << 4);

    p_scip.ir_drive_regs.b_command_reg = b_id_cmd;
    p_scip.b_drive_number = b_drive_num;
    p_scip.c_buffer_size = IDENTIFY_BUFFER_SIZE as ULONG;

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

pub fn extract_serial_from_identify_data(identify_data: &[BYTE]) -> io::Result<String> {
    let mut reader = io::Cursor::new(identify_data);
    let mut dw_disk_data: [DWORD; 256] = [0; 256];
    for i_ijk in 0..256 {
        dw_disk_data[i_ijk] = reader.read_u16::<LittleEndian>()? as DWORD;
    }

    let mut csz_serial_number: [BYTE; 1024] = [0; 1024];
    convert_to_string(&dw_disk_data, 10, 19, &mut csz_serial_number);

    let serial_number = String::from_utf8_lossy(&csz_serial_number)
        .trim_end_matches(char::from(0))
        .to_string();

    Ok(serial_number)
}
