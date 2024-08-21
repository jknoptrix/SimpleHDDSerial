# simplehddserial: A (Highly Unstable) Rust Crate for Retrieving Hard Drive Serial Numbers on Windows

**Important Note:** This crate is currently **extremely unstable** and is known to fail in approximately 1/3 of attempts to retrieve serial numbers. Use with caution and be prepared for potential errors.

This crate provides functionality to retrieve hard drive serial numbers on Windows systems. It utilizes various methods, including those requiring administrator privileges, SMART capabilities, and even attempts retrieval with zero rights. Due to the complexities of interacting with hardware and different driver implementations, the stability of this crate is currently limited.

## Features

* **Multiple Retrieval Methods:** Attempts to retrieve serial numbers using different approaches:
    * **Admin Rights:** Utilizes `CreateFileA` with administrator privileges.
    * **SMART:** Leverages SMART (Self-Monitoring, Analysis and Reporting Technology) capabilities.
    * **Zero Rights:** Attempts retrieval using minimal access rights.
* **Parallel Execution:** Employs multi-threading to query multiple drives concurrently, potentially improving performance.
* **Error Handling:** Provides error messages indicating the source of failure (e.g., specific drive or method).
* **Timing Information:** Outputs timing information for each method and drive, allowing for performance analysis and identification of bottlenecks.

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
simplehddserial = "0.1.0" // Replace with the actual version
```

Then, in your code:

```rust
use simplehddserial::HardDriveSerial;

fn main() {
    let mut hd_serial = HardDriveSerial::new();
    match hd_serial.get_serial_numbers() {
        Ok(serial_numbers) => {
            for serial in serial_numbers {
                println!("Serial number: {}", serial);
            }
        }
        Err(errors) => {
            println!("Error: {:?}", errors);
        }
    }
}
```

## Technical Details

The crate interacts with hard drives through Windows APIs, primarily using `DeviceIoControl` for sending IOCTL (Input/Output Control) requests.  It utilizes several IOCTL codes and data structures:

* **DFP_GET_VERSION:** Retrieves the driver version information.
* **DFP_RECEIVE_DRIVE_DATA:** Sends commands to the drive and receives data.
* **SMART_GET_VERSION:** Retrieves SMART version information.
* **SMART_RCV_DRIVE_DATA:** Sends SMART commands to the drive and receives data.
* **IOCTL_STORAGE_QUERY_PROPERTY:** Queries storage device properties, including serial numbers.
* **IOCTL_DISK_GET_DRIVE_GEOMETRY_EX:** Retrieves extended disk geometry information.
* **IOCTL_DISK_GET_DRIVE_GEOMETRY:** Retrieves basic disk geometry information.

The core functionality involves sending commands like `IDE_ATA_IDENTIFY` (Identify Device) to the drive and parsing the returned data to extract the serial number.

Different access levels (admin rights, SMART rights, zero rights) are attempted to maximize the chances of retrieval, as some methods might fail depending on the system configuration and driver implementations.

## Limitations and Instability

As mentioned earlier, this crate is highly unstable and prone to errors. This is due to several factors:

* **Driver Variations:** Different hard drive manufacturers and models might have different driver implementations, leading to inconsistencies in the data returned or even failure to respond to specific IOCTL requests.
* **Operating System Compatibility:** While designed for Windows, the specific behavior might vary across different Windows versions.
* **Access Permissions:**  Retrieving serial numbers often requires elevated privileges, and the lack thereof might hinder certain retrieval methods.

**The observed error rate of approximately 1/3 is a significant limitation, and this crate should not be used in production environments or where reliable retrieval is critical.**

## Future Work

* **Improved Stability:** Investigate the causes of instability and explore more robust methods for retrieving serial numbers across different hardware and driver configurations.
* **Error Handling and Recovery:** Implement better error handling mechanisms and potentially incorporate fallback strategies when a particular method fails.
* **Testing and Validation:**  Expand the testing coverage across various hardware and software environments to identify and address potential issues.

## Contributing

Contributions are welcome!  If you encounter issues or have ideas for improvements, please open an issue or submit a pull request.


## Disclaimer

This crate is provided "as is" and without warranties. The author is not responsible for any data loss or damage caused by the use of this crate.

This detailed README provides a comprehensive overview of the `simplehddserial` crate, including its features, technical details, limitations, and future plans.  It highlights the instability issues and emphasizes the need for caution when using this crate.
