use super::parsers::{parse_label, parse_statement};
use super::statements::Statement;
use crate::assembler::codegen::codegen;
use crate::assembler::size::statement_size;
use crate::logging::{print_assembler_error, AssemblerError, AssemblerErrorKind};
use crate::preprocessor::{CodeLine, LineLocation};
use nom::Err;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
struct AbstractBinaryLine {
    address: u64,
    statement: Statement,
    location: LineLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssembledLine {
    pub address: u64,
    pub data: Vec<u16>,
    pub location: LineLocation,
}

/// Assembles a list of `CodeLine`s to a list of `AssembledLine`s, containing the raw machine code
pub fn assemble(input: &[CodeLine]) -> Result<Vec<AssembledLine>, Vec<AssembledLine>> {
    let mut error_occurred = false;
    let mut current_address: u64 = 0;

    // Create abstract binary list
    let mut abstract_binary_list: Vec<AbstractBinaryLine> = vec![];
    let mut label_table: HashMap<String, u64> = HashMap::new();
    for line in input {
        // Parse code line
        let (label, statement) = match parse_code_line(&line.body, &line.location) {
            Ok(res) => res,
            Err(res) => {
                error_occurred = true;
                res
            }
        };

        // Handle inserting label into label table
        if let Some(label) = label {
            // Check if label is already defined
            if !label_table.contains_key(&label) {
                // Insert label into label table
                label_table.insert(label, current_address);
            } else {
                // Double label definition
                print_assembler_error(AssemblerError {
                    kind: AssemblerErrorKind::LabelDoubleDefinition(label),
                    location: Some(line.location.clone()),
                });
                error_occurred = true;
            }
        }

        // Handle adding statements to abstract binary list
        if let Some(statement) = statement {
            // If statement is Org, change current address
            if let Statement::Org(address) = statement {
                current_address = address;
                continue;
            }

            // Calculate statement size in words
            let size = statement_size(&statement, current_address);

            // Add statement to Abstract Binary List
            abstract_binary_list.push(AbstractBinaryLine {
                address: current_address,
                statement,
                location: line.location.clone(),
            });

            // Update current address with size of just processed statement
            current_address += size;
        }
    }

    println!("{:#?}", label_table);
    println!("{:#?}", abstract_binary_list);

    // Generate machine code
    let mut result: Vec<AssembledLine> = vec![];
    for line in abstract_binary_list {
        // Generate binary for this statement
        let data: Vec<u16> = match codegen(&line.statement, line.address, &label_table) {
            Ok(res) => res,
            Err(err) => {
                error_occurred = true;
                print_assembler_error(AssemblerError {
                    kind: err,
                    location: Some(line.location),
                });
                continue;
            }
        };

        result.push(AssembledLine {
            address: line.address,
            data,
            location: line.location,
        })
    }

    // Return result based on whether an error occurred or not
    match error_occurred {
        false => Ok(result),
        true => Err(result),
    }
}

// Parse code line
fn parse_code_line(
    input: &str,
    location: &LineLocation,
) -> Result<(Option<String>, Option<Statement>), (Option<String>, Option<Statement>)> {
    // Parse label
    let (input, label) = match parse_label(input) {
        Ok((input, res)) => (input, Some(res.to_owned())),
        Err(_) => (input, None),
    };

    // Check if there is a statement
    if input.len() > 0 {
        let (_input, statement) = match parse_statement(input.trim()) {
            Ok((input, res)) => (input, Some(res)),
            Err(err) => {
                match err {
                    Err::Failure(err) => print_assembler_error(AssemblerError {
                        kind: err.kind,
                        location: Some(location.clone()),
                    }),
                    Err::Error(err) => print_assembler_error(AssemblerError {
                        kind: err.kind,
                        location: Some(location.clone()),
                    }),
                    Err::Incomplete(_) => {}
                }
                return Err((label, None));
            }
        };
        Ok((label, statement))
    } else {
        Ok((label, None))
    }
}
