use winapi::shared::minwindef::{DWORD, BYTE, USHORT};
use winapi::shared::ntdef::ULONG;


pub const IDENTIFY_BUFFER_SIZE: usize = 512;
pub const DFP_GET_VERSION: DWORD = 0x00074080;
pub const DFP_RECEIVE_DRIVE_DATA: DWORD = 0x0007c088;
pub const IDE_ATA_IDENTIFY: BYTE = 0xEC;
pub const MAX_IDE_DRIVES: usize = 16;
pub const INVALID_HANDLE_VALUE: winapi::shared::ntdef::HANDLE = usize::MAX as winapi::shared::ntdef::HANDLE;

#[repr(C)]
pub struct GETVERSIONOUTPARAMS {
    pub b_version: BYTE,
    pub b_revision: BYTE,
    pub b_reserved: BYTE,
    pub b_ide_device_map: BYTE,
    pub f_capabilities: DWORD,
    pub dw_reserved: [DWORD; 4],
}

#[repr(C)]
pub struct SENDCMDINPARAMS {
    pub c_buffer_size: ULONG,
    pub ir_drive_regs: IDEREGS,
    pub b_drive_number: BYTE,
    pub c_buffer: [BYTE; IDENTIFY_BUFFER_SIZE],
}

#[repr(C)]
pub struct IDEREGS {
    pub b_features_reg: BYTE,
    pub b_sector_count_reg: BYTE,
    pub b_sector_number_reg: BYTE,
    pub b_cyl_low_reg: BYTE,
    pub b_cyl_high_reg: BYTE,
    pub b_drive_head_reg: BYTE,
    pub b_command_reg: BYTE,
    pub b_reserved: BYTE,
}

#[repr(C)]
pub struct SENDCMDOUTPARAMS {
    pub c_buffer_size: ULONG,
    pub ir_drive_regs: IDEREGS,
    pub b_drive_number: BYTE,
    pub b_status_reg: BYTE,
    pub b_error_reg: BYTE,
    pub b_interrupt_reason_reg: BYTE,
    pub b_reserved: [BYTE; 3],
    pub b_buffer: [BYTE; IDENTIFY_BUFFER_SIZE],
}

#[repr(C)]
pub struct GETVERSIONINPARAMS {
    pub b_version: BYTE,
    pub b_revision: BYTE,
    pub b_reserved: BYTE,
    pub b_ide_device_map: BYTE,
    pub f_capabilities: DWORD,
    pub dw_reserved: [DWORD; 4],
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct IDENTIFY_DATA {
    pub GeneralConfiguration: USHORT,
    pub NumberOfCylinders: USHORT,
    pub Reserved1: USHORT,
    pub NumberOfHeads: USHORT,
    pub UnformattedBytesPerTrack: USHORT,
    pub UnformattedBytesPerSector: USHORT,
    pub SectorsPerTrack: USHORT,
    pub VendorUnique1: [USHORT; 3],
    pub SerialNumber: [USHORT; 10],
    pub BufferType: USHORT,
    pub BufferSectorSize: USHORT,
    pub NumberOfEccBytes: USHORT,
    pub FirmwareRevision: [USHORT; 4],
    pub ModelNumber: [USHORT; 20],
    pub MaximumBlockTransfer: BYTE,
    pub VendorUnique2: BYTE,
    pub DoubleWordIo: USHORT,
    pub Capabilities: USHORT,
    pub Reserved2: USHORT,
    pub VendorUnique3: BYTE,
    pub PioCycleTimingMode: BYTE,
    pub VendorUnique4: BYTE,
    pub DmaCycleTimingMode: BYTE,
    pub TranslationFieldsValid: USHORT,
    pub NumberOfCurrentCylinders: USHORT,
    pub NumberOfCurrentHeads: USHORT,
    pub CurrentSectorsPerTrack: USHORT,
    pub CurrentSectorCapacity: ULONG,
    pub CurrentMultiSectorSetting: USHORT,
    pub UserAddressableSectors: ULONG,
    pub SingleWordDMASupport: USHORT,
    pub MultiWordDMASupport: USHORT,
    pub AdvancedPIOModes: USHORT,
    pub MinimumMWXferCycleTime: USHORT,
    pub RecommendedMWXferCycleTime: USHORT,
    pub MinimumPIOCycleTime: USHORT,
    pub MinimumPIOCycleTimeIORDY: USHORT,
    pub Reserved5: [USHORT; 2],
    pub ReleaseTimeOverlapped: USHORT,
    pub ReleaseTimeServiceCommand: USHORT,
    pub MajorRevision: USHORT,
    pub MinorRevision: USHORT,
    pub Reserved6: [USHORT; 50],
    pub SpecialFunctionsEnabled: USHORT,
    pub Reserved7: [USHORT; 128],
}

#[repr(C)]
pub struct STORAGE_DEVICE_DESCRIPTOR {
    pub version: DWORD,
    pub size: DWORD,
    pub device_type: BYTE,
    pub vendor_id_offset: DWORD,
    pub product_id_offset: DWORD,
    pub product_revision_offset: DWORD,
    pub serial_number_offset: DWORD,
    pub bus_type: BYTE,
    pub raw_properties_length: DWORD,
    pub raw_device_properties: [BYTE; 1],
}

#[repr(C)]
pub struct DISK_GEOMETRY {
    pub cylinders: ULONG,
    pub media_type: BYTE,
    pub tracks_per_cylinder: USHORT,
    pub sectors_per_track: USHORT,
    pub bytes_per_sector: USHORT,
}

#[repr(C)]
pub struct DISK_GEOMETRY_EX {
    pub geometry: DISK_GEOMETRY,
    pub disk_size: i64,
    pub data: [BYTE; 1],
}


#[repr(C)]
pub struct STORAGE_PROPERTY_QUERY {
    pub PropertyId: DWORD,
    pub QueryType: DWORD,
    pub AdditionalParameters: [BYTE; 1],
}