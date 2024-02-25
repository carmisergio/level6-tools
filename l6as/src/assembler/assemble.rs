use super::instructions::Statement;
use super::parsers::{parse_label, parse_statement};
use crate::logging::{print_assembler_error, AssemblerError};
use crate::preprocessor::{CodeLine, LineLocation};
use nom::Err;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
struct AbstractBinaryLine {
    address: u128,
    statement: Statement,
    location: LineLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssembledLine {
    data: Vec<u8>,
    location: LineLocation,
}

/// Assembles a list of `CodeLine`s to a list of `AssembledLine`s, containing the raw machine code
pub fn assemble(input: &[CodeLine]) -> Result<Vec<AssembledLine>, Vec<AssembledLine>> {
    let mut error_occurred = false;
    let mut current_address: u128 = 0;

    // Create abstract binary list
    let mut abstract_binary_list: Vec<AbstractBinaryLine> = vec![];
    let mut label_table: HashMap<String, u128> = HashMap::new();
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
            label_table.insert(label, current_address);
        }

        // Handle adding statements to abstract binary list
        if let Some(statement) = statement {
            // If statement is Org, change current address
            if let Statement::Org(address) = statement {
                current_address = address;
                continue;
            }

            abstract_binary_list.push(AbstractBinaryLine {
                address: current_address,
                statement,
                location: line.location.clone(),
            });
        }
    }

    println!("{:#?}", label_table);
    println!("{:#?}", abstract_binary_list);

    // Generate machine code
    let mut result: Vec<AssembledLine> = vec![];
    for line in abstract_binary_list {
        // Generate binary for this statement
        let data: Vec<u8> = vec![0x01, 0x02];

        result.push(AssembledLine {
            data,
            location: line.location,
        })
    }

    let result: Vec<AssembledLine> = vec![];

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

    // // Parse statement
    // let (_input, statement) = match parse_statement(input) {
    //     Ok((input, res)) => (input, Some(res)),
    //     Err(err) => {
    //         match err {
    //             Err::Failure(err) => print_assembler_error(AssemblerError {
    //                 kind: err.kind,
    //                 location: Some(location.clone()),
    //             }),
    //             Err::Error(err) => print_assembler_error(AssemblerError {
    //                 kind: err.kind,
    //                 location: Some(location.clone()),
    //             }),
    //             Err::Incomplete(_) => {}
    //         }
    //         error_occurred = true;
    //         (input, None)
    //     }
    // };

    // match error_occurred {
    //     false => Ok((label, statement)),
    //     true => Err((label, statement)),
    // }
}
