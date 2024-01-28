// In-module imports
use super::disk_parameters::DiskParameters;
use super::encode::encode_disk;
use super::errors::{ConvertError, ConvertErrorType};

#[derive(Debug)]
pub enum RawImageFormat {
    HFE,
}
#[derive(Debug)]
// Encoding options
pub struct ConvertOpts {
    pub ignore_errors: bool,
    pub disk_parameters: DiskParameters,
    pub out_file_format: RawImageFormat,
}

pub type ConvertResult = Result<Vec<u8>, ConvertError>;

// Floppy Data types
pub type Sector = Vec<u8>;
pub type Track = Vec<u8>;
pub type Cylinder = Vec<Track>;

// Convert data image to raw floppy image
pub fn convert_to_raw(data_img: Vec<u8>, opts: ConvertOpts) -> ConvertResult {
    // Divide disk image into sectors
    let mut sectors = match divide_data_image_sectors(
        data_img,
        opts.disk_parameters.bytes_per_sector,
        opts.ignore_errors,
    ) {
        Ok(sectors) => sectors,
        Err(()) => return Err(ConvertError::new(ConvertErrorType::SectorDivision)),
    };

    // Check number of sectors
    let expected_sectors: usize = (opts.disk_parameters.sectors_per_track
        * opts.disk_parameters.n_sides
        * opts.disk_parameters.n_cylinders) as usize;

    if sectors.len() != expected_sectors {
        // Wrong number of sectors
        if opts.ignore_errors {
            if sectors.len() < expected_sectors {
                // Add sectors to fill remaining tracks
                sectors.append(&mut vec![
                    vec![
                        0;
                        opts.disk_parameters.bytes_per_sector as usize
                    ];
                    expected_sectors - sectors.len()
                ]);
            }
        } else {
            return Err(ConvertError::new(ConvertErrorType::SectorNumber));
        }
    }

    // Encode disk image to correct format
    let _encoded_disk = match encode_disk(&sectors, &opts.disk_parameters) {
        Ok(disk) => disk,
        Err(msg) => return Err(ConvertError::new(ConvertErrorType::DiskEncoding(msg))),
    };

    let out_file_data: Vec<u8> = vec![b'b', 100];

    Ok(out_file_data)
}

// Divide data image into vector of sectors with given size
fn divide_data_image_sectors(
    data_img: Vec<u8>,
    sector_size: u16,
    ignore_errors: bool,
) -> Result<Vec<Sector>, ()> {
    let mut sectors: Vec<Sector> = vec![];

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
            let mut this_sector: Sector = vec![0; sector_size as usize];
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
