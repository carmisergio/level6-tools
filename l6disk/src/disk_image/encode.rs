use std::vec;

// In-module imports
use super::convert::{Cylinder, Sector, Track};
pub use super::disk_parameters::{DiskParameters, DiskTrackFormat};
use super::level6::encode_track_level6;

// Encode all cylinders in the disk
pub fn encode_disk(
    sectors: &Vec<Sector>,
    disk_parameters: &DiskParameters,
) -> Result<Vec<Cylinder>, String> {
    let mut cylinders: Vec<Cylinder> = vec![];

    // println!("Encoding disk: n_sectors = {}", sectors.len());

    // Encode each cylinder of the disk
    for cyl_n in 0..disk_parameters.n_cylinders {
        // Encode cylinder
        let cylinder = match encode_cylinder(sectors, disk_parameters, cyl_n) {
            Ok(cylinder) => cylinder,
            Err(msg) => return Err(msg),
        };

        cylinders.push(cylinder);
    }

    Ok(cylinders)
}

// Encode one cylinder in the disk
fn encode_cylinder(
    sectors: &Vec<Sector>,
    disk_parameters: &DiskParameters,
    cyl_n: u16,
) -> Result<Cylinder, String> {
    let mut cylinder: Cylinder = vec![];
    // println!("Encoding cylinder: {}", cyl_n);

    // Encode each side of this cylinder
    for side_n in 0..disk_parameters.n_sides {
        // Encode this side
        let track = match encode_track(&sectors, &disk_parameters, cyl_n, side_n) {
            Ok(track) => track,
            Err(msg) => return Err(msg),
        };

        // Add track to cylinder
        cylinder.push(track);
    }

    Ok(cylinder)
}

// Encode one track
fn encode_track(
    sectors: &Vec<Sector>,
    disk_parameters: &DiskParameters,
    cyl_n: u16,
    side_n: u16,
) -> Result<Track, String> {
    // Compute start  and end sector of this track
    let start_sector: usize =
        (disk_parameters.sectors_per_track * (side_n + disk_parameters.n_sides * cyl_n)) as usize;
    let end_sector: usize = start_sector + disk_parameters.sectors_per_track as usize;

    // println!(
    //     "Encoding track cyl_n={}, side_n={}: start_sector={}",
    //     cyl_n, side_n, start_sector
    // );

    // Encode track to appropriate format
    match disk_parameters.track_format {
        DiskTrackFormat::Level6 => encode_track_level6(
            &sectors[start_sector..end_sector],
            &disk_parameters,
            cyl_n,
            side_n,
        ),
    }
}

pub fn calc_interleave_map(n_sectors: u16, interleave: u16) -> Vec<u16> {
    let mut res: Vec<u16> = vec![0; n_sectors as usize];
    let mut used: Vec<bool> = vec![false; n_sectors as usize];

    // Place all the physical sectors

    let mut curs: usize = 0;
    let mut skip: u16 = interleave; // Place first logical sector in first physical sector
    for log_sec in 0..n_sectors {
        // Find non used sector and skip correct number of sectors
        while used[curs] || skip < interleave {
            skip += 1;

            // Move cursor to next sector
            curs += 1;
            if curs >= res.len() {
                curs = 0;
            }
        }

        // Place logical sector in interleave map
        res[curs] = log_sec;
        used[curs] = true;
        skip = 1;

        // Move cursor to next sector
        curs += 1;
        if curs >= res.len() {
            curs = 0;
        }
    }

    res
}

// TODO fix interleave function again
