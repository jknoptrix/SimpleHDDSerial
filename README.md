# hard-drive-serial

A Rust library for retrieving hard drive serial numbers on Windows. This library provides functionality to read the serial numbers of physical drives connected to the system. It employs multiple methods to access drive information, ensuring compatibility with different levels of user privileges.

## Usage

```rust
use hard_drive_serial::HardDriveSerial;

fn main() -> Result<(), String> {
    let mut hard_drive_serial = HardDriveSerial::new();
    let serial_numbers = hard_drive_serial.get_serial_numbers()?;
    
    for serial in serial_numbers {
        println!("Serial Number: {}", serial);
    }

    Ok(())
}
```

## Features

- **Multiple Access Methods:**  Utilizes different techniques to access drive information:
    - **Admin Rights:** Attempts to read drive information with administrator privileges.
    - **SMART:** Uses the SMART (Self-Monitoring, Analysis and Reporting Technology) interface.
    - **Zero Rights:** Tries to read information with minimal access rights.
- **Error Handling:** Returns a `Result` indicating success or failure with an error message if necessary.
- **Cross-Platform Compatibility:** Designed for Windows.

## Dependencies

This library uses the `winapi` crate for interacting with the Windows API and `lazy_static` for efficient initialization of static variables.


## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hard-drive-serial = "0.1.0" // Replace with the actual version
```

## Example

```rust
use hard_drive_serial::HardDriveSerial;

fn main() -> Result<(), String> {
    let mut hard_drive_serial = HardDriveSerial::new();
    let serial_numbers = hard_drive_serial.get_serial_numbers()?;
    
    println!("{:?}", serial_numbers); // Output: ["SERIAL1", "SERIAL2", ...]

    Ok(())
}
```