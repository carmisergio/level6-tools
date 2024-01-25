pub use super::disk_parameters::DiskParameters;
pub use super::errors::{EncodeError, EncodeErrorType};

// Encoding options
pub struct EncodeOpts {
    pub ignore_errors: bool,
    pub format: DiskParameters,
}

// Image encoding result
pub type EncodeResult = Result<Vec<u8>, EncodeError>;

// Convert data disk image to raw disk image
pub fn encode_disk_image(data_img: Vec<u8>, opts: EncodeOpts) -> EncodeResult {
    let generated_data: Vec<u8> = vec![b'a'; 100];

    // Divide disk image into sectors
    let sectors =
        match divide_data_image_sectors(data_img, opts.format.sector_size, opts.ignore_errors) {
            Ok(sectors) => sectors,
            Err(()) => return Err(EncodeError::new(EncodeErrorType::SectorDivision)),
        };

    println!("{:?}", sectors);

    Ok(generated_data)
}

// Divide data image into vector of sectors with given size
pub fn divide_data_image_sectors(
    data_img: Vec<u8>,
    sector_size: u16,
    ignore_errors: bool,
) -> Result<Vec<Vec<u8>>, ()> {
    let mut sectors: Vec<Vec<u8>> = vec![];

    // Convert image to sectors
    let mut bytes_consumed: usize = 0;
    while bytes_consumed < data_img.len() {
        let remaining_bytes = data_img.len() - bytes_consumed;

        // Create new sector
        if remaining_bytes >= sector_size as usize {
            // Valid sector boundary
            sectors.push(data_img[bytes_consumed..bytes_consumed + sector_size as usize].to_vec());
        } else if ignore_errors {
            // Invalid sector boundary but we ignore errors

            // Fill remaining bytes with zeroes
            let mut this_sector: Vec<u8> = vec![0; sector_size as usize];
            this_sector[0..remaining_bytes]
                .copy_from_slice(&data_img[bytes_consumed..data_img.len()]);
            sectors.push(this_sector);
        } else {
            // Sector boundary error
            return Err(());
        }

        bytes_consumed += sector_size as usize;
    }

    Ok(sectors)
}
