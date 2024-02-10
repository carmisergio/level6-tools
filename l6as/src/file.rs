use std::env::current_dir;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct FileInclusionCoordinator {
    include_dirs: Vec<PathBuf>,
    already_included: Vec<PathBuf>,
}

impl FileInclusionCoordinator {
    pub fn new() -> Self {
        Self {
            include_dirs: vec![],
            already_included: vec![],
        }
    }

    fn add_include_dir(&mut self, dir: PathBuf) -> io::Result<()> {
        let mut absolute_path = current_dir()?;
        absolute_path.push(dir);
        self.include_dirs.push(absolute_path);
        Ok(())
    }

    pub fn add_include_dirs(&mut self, dirs: &[PathBuf]) -> io::Result<()> {
        for dir in dirs {
            self.add_include_dir(dir.into())?;
        }
        Ok(())
    }

    pub fn add_current_dir(&mut self) -> io::Result<()> {
        let absolute_path = current_dir()?;
        self.include_dirs.push(absolute_path);
        Ok(())
    }

    pub fn read_file(
        &mut self,
        file_path: &PathBuf,
    ) -> Result<(PathBuf, String), FileInclusionError> {
        // Try every include directory
        for dir_path in &self.include_dirs {
            let mut abs_path = dir_path.clone();

            abs_path.push(&file_path);

            // Read this file
            let contents = match fs::read_to_string(&abs_path) {
                Ok(cont) => cont,
                Err(_err) => continue, // Next include directory
            };

            // Check if this file was already included
            if self.already_included.contains(&abs_path) {
                return Err(FileInclusionError::DoubleInclusion(abs_path.clone()));
            }

            self.already_included.push(abs_path.clone());

            return Ok((abs_path, contents));
        }

        Err(FileInclusionError::FileNotFound(file_path.clone()))
    }
}

#[derive(Debug)]
pub enum FileInclusionError {
    FileNotFound(PathBuf),
    DoubleInclusion(PathBuf),
}
