#![warn(dead_code)]
use winapi::shared::minwindef::{DWORD, BYTE, USHORT};
use winapi::shared::ntdef::ULONG;

// define constants used for interacting with the hard drive
pub const IDENTIFY_BUFFER_SIZE: usize = 512; // size of the buffer used for IDENTIFY commands
pub const DFP_GET_VERSION: DWORD = 0x00074080; // IOCTL code for getting the driver version
pub const DFP_RECEIVE_DRIVE_DATA: DWORD = 0x0007c088; // IOCTL code for receiving drive data
pub const IDE_ATA_IDENTIFY: BYTE = 0xEC; // command code for ATA IDENTIFY DEVICE
pub const MAX_IDE_DRIVES: usize = 16; // maximum number of IDE drives to check
pub const INVALID_HANDLE_VALUE: winapi::shared::ntdef::HANDLE = usize::MAX as winapi::shared::ntdef::HANDLE; // invalid handle value


// structures representing various data structures used for IOCTL communication
// and retrieving hard drive information

// structure for retrieving driver version information
#[repr(C)]
pub struct GETVERSIONOUTPARAMS {
    pub b_version: BYTE, // major version number
    pub b_revision: BYTE, // minor version number
    pub b_reserved: BYTE, // reserved byte
    pub b_ide_device_map: BYTE, // IDE device map
    pub f_capabilities: DWORD, // driver capabilities
    pub dw_reserved: [DWORD; 4], // reserved DWORDs
}

// structure for sending commands to the drive
#[repr(C)]
pub struct SENDCMDINPARAMS {
    pub c_buffer_size: ULONG, // size of the command buffer
    pub ir_drive_regs: IDEREGS, // IDE registers
    pub b_drive_number: BYTE, // drive number
    pub c_buffer: [BYTE; IDENTIFY_BUFFER_SIZE], // command buffer
}

// structure representing IDE registers
#[repr(C)]
pub struct IDEREGS {
    pub b_features_reg: BYTE, // features register
    pub b_sector_count_reg: BYTE, // sector count register
    pub b_sector_number_reg: BYTE, // sector number register
    pub b_cyl_low_reg: BYTE, // low cylinder register
    pub b_cyl_high_reg: BYTE, // high cylinder register
    pub b_drive_head_reg: BYTE, // drive head register
    pub b_command_reg: BYTE, // command register
    pub b_reserved: BYTE, // reserved byte
}

// structure for receiving command output from the drive
#[repr(C)]
pub struct SENDCMDOUTPARAMS {
    pub c_buffer_size: ULONG, // size of the output buffer
    pub ir_drive_regs: IDEREGS, // IDE registers
    pub b_drive_number: BYTE, // drive number
    pub b_status_reg: BYTE, // status register
    pub b_error_reg: BYTE, // error register
    pub b_interrupt_reason_reg: BYTE, // interrupt reason register
    pub b_reserved: [BYTE; 3], // reserved bytes
    pub b_buffer: [BYTE; IDENTIFY_BUFFER_SIZE], // output buffer
}

// structure for retrieving SMART version information
#[repr(C)]
pub struct GETVERSIONINPARAMS {
    pub b_version: BYTE, // major version number
    pub b_revision: BYTE, // minor version number
    pub b_reserved: BYTE, // reserved byte
    pub b_ide_device_map: BYTE, // IDE device map
    pub f_capabilities: DWORD, // driver capabilities
    pub dw_reserved: [DWORD; 4], // reserved DWORDs
}


// structure containing detailed information about the drive, retrieved via IDENTIFY DEVICE command
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

// structure representing a storage device descriptor
#[repr(C)]
pub struct STORAGE_DEVICE_DESCRIPTOR {
    pub version: DWORD, // descriptor version
    pub size: DWORD, // descriptor size
    pub device_type: BYTE, // device type
    pub vendor_id_offset: DWORD, // offset to the vendor ID string
    pub product_id_offset: DWORD, // offset to the product ID string
    pub product_revision_offset: DWORD, // offset to the product revision string
    pub serial_number_offset: DWORD, // offset to the serial number string
    pub bus_type: BYTE, // bus type
    pub raw_properties_length: DWORD, // length of the raw device properties
    pub raw_device_properties: [BYTE; 1], // raw device properties
}

// structure representing disk geometry
#[repr(C)]
pub struct DISK_GEOMETRY {
    pub cylinders: ULONG, // number of cylinders
    pub media_type: BYTE, // media type
    pub tracks_per_cylinder: USHORT, // number of tracks per cylinder
    pub sectors_per_track: USHORT, // number of sectors per track
    pub bytes_per_sector: USHORT, // number of bytes per sector
}

// structure representing extended disk geometry
#[repr(C)]
pub struct DISK_GEOMETRY_EX {
    pub geometry: DISK_GEOMETRY, // disk geometry
    pub disk_size: i64, // disk size in bytes
    pub data: [BYTE; 1], // additional data
}


// structure used for querying storage device properties
#[repr(C)]
pub struct STORAGE_PROPERTY_QUERY {
    pub PropertyId: DWORD, // property ID to query
    pub QueryType: DWORD, // type of query
    pub AdditionalParameters: [BYTE; 1], // additional parameters
}