use std::path::PathBuf;

use crate::{
    assembler::AssembledLine, file::write_file, logging::print_write_file_error_msg,
    preprocessor::CodeLine,
};

/// Write preprocessor output to file
pub fn write_preprocessor_output(file_path: &PathBuf, lines: &[CodeLine]) -> Result<(), ()> {
    let mut string = String::new();

    for code_line in lines {
        string.push_str(&code_line.body);
        string.push_str("\r\n");
    }

    // Write output to file
    match write_file(file_path, &string.as_bytes()) {
        Ok(()) => Ok(()),
        Err(err) => {
            print_write_file_error_msg(err);
            Err(())
        }
    }
}

/// Write assembler output to a binary file
pub fn write_assembler_binary_output(
    file_path: &PathBuf,
    lines: &[AssembledLine],
) -> Result<(), ()> {
    // Convert output to string
    let mut output: Vec<u8> = vec![];
    for line in lines {
        for word in line.data.clone() {
            output.extend_from_slice(&word.to_be_bytes());
        }
    }

    // Write output to file
    match write_file(file_path, &output) {
        Ok(()) => Ok(()),
        Err(err) => {
            print_write_file_error_msg(err);
            Err(())
        }
    }
}

/// Write assembler output to a listing file
pub fn write_assembler_listing_output(
    file_path: &PathBuf,
    lines: &[AssembledLine],
) -> Result<(), ()> {
    let mut string = String::new();

    for line in lines {
        string.push_str(&generate_line_listing(line));
    }

    // Write output to file
    match write_file(file_path, &string.as_bytes()) {
        Ok(()) => Ok(()),
        Err(err) => {
            print_write_file_error_msg(err);
            Err(())
        }
    }
}

// Generate listing for a single AssembledLine
fn generate_line_listing(line: &AssembledLine) -> String {
    let mut words_written: usize = 0;
    let mut output: String = "".to_owned();

    while words_written < line.data.len() {
        // Compute address column
        let (address_column, code_column) = if words_written == 0 {
            (
                format!("{:0>4X}", line.address),
                line.location.raw_content.clone(),
            )
        } else {
            ("    ".to_owned(), "".to_owned())
        };

        // Calculate instruction words field
        let mut words_column = "".to_owned();
        for i in 0..2 {
            words_column.push_str(&if words_written + i < line.data.len() {
                format!("{:0>4X} ", line.data[words_written + i])
            } else {
                "     ".to_owned()
            });
        }

        words_written += 2;

        output.push_str(&format!(
            "{}:  {}   {}\r\n",
            address_column, words_column, code_column
        ))
    }

    output
}
