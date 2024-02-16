use nom::Err;
use std::path::PathBuf;
use std::{collections::HashMap, vec};

use crate::file::{FileInclusionCoordinator, FileInclusionError};

use crate::logging::{
    print_preprocessor_error, print_preprocessor_warning, PreprocessorError, PreprocessorErrorKind,
    PreprocessorWarning, PreprocessorWarningKind,
};

use super::parsers::{parse_definitions_chunks, parse_source_line};

#[derive(Debug, PartialEq, Clone)]
pub enum SourceLineBody {
    Empty,
    Define(String, String),
    Include(PathBuf),
    Code(String),
}
#[derive(Debug, PartialEq, Clone)]
pub struct LineLocation {
    pub line_n: usize,
    pub file_name: PathBuf,
    pub raw_content: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SourceLine {
    body: SourceLineBody,
    comment: String,
    location: LineLocation,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CodeLine {
    body: String,
    comment: String,
    location: LineLocation,
}

/// Preprocess a program
pub fn preprocess(
    file_path: &PathBuf,
    fi_coord: &mut FileInclusionCoordinator,
) -> Result<Vec<CodeLine>, Vec<CodeLine>> {
    let mut error_encountered = false;

    // Parse the source file (resolving all includes)
    let source_lines = match parse_source_file(file_path, fi_coord, &None) {
        Ok(lines) => lines,
        Err(lines) => {
            error_encountered = true;
            lines
        }
    };

    // Process %defines
    let source_lines = match process_defines(&source_lines) {
        Ok(lines) => lines,
        Err(lines) => {
            error_encountered = true;
            lines
        }
    };

    // Flatten all code lines to string
    let res = source_lines_to_code_lines(&source_lines);

    // Return results
    match error_encountered {
        false => Ok(res),
        true => Err(res),
    }
}

/// Parses input source file to a vector of `SourceLine`s, resolving includes
fn parse_source_file(
    file_path: &PathBuf,
    fi_coord: &mut FileInclusionCoordinator,
    include_location: &Option<LineLocation>,
) -> Result<Vec<SourceLine>, Vec<SourceLine>> {
    let mut error_encountered = false;

    // Read file
    let (abs_path, code) = match fi_coord.read_file(&file_path) {
        Ok(res) => res,
        Err(err) => {
            // Log error
            match err {
                FileInclusionError::FileNotFound(file_path) => {
                    print_preprocessor_error(PreprocessorError {
                        kind: PreprocessorErrorKind::CannotOpenSourceFile(file_path),
                        location: include_location.clone(),
                    });
                }
                FileInclusionError::DoubleInclusion(file_path) => {
                    print_preprocessor_error(PreprocessorError {
                        kind: PreprocessorErrorKind::DobleInclusion(file_path),
                        location: include_location.clone(),
                    });
                }
            }

            return Err(vec![]);
        }
    };

    // Parse source lines
    let lines = match parse_source_string(&code, &abs_path) {
        Ok(lines) => lines,
        Err(lines) => {
            error_encountered = true;
            lines
        }
    };

    // Process %includes
    let lines = match process_includes(&lines, fi_coord) {
        Ok(res) => res,
        Err(lines) => {
            error_encountered = true;
            lines
        }
    };

    // Return results
    match error_encountered {
        false => Ok(lines),
        true => Err(lines),
    }
}

/// Converts a Vec of SourceLines (containing only Code) to strings,
/// fails if vec contains other types of lines
fn source_lines_to_code_lines(input: &[SourceLine]) -> Vec<CodeLine> {
    let mut strings: Vec<CodeLine> = vec![];

    // Go over all lines
    for line in input {
        if let SourceLineBody::Code(content) = &line.body {
            strings.push(CodeLine {
                body: content.to_owned(),
                comment: line.comment.to_owned(),
                location: line.location.to_owned(),
            });
        } else {
            panic!();
        }
    }

    strings
}

/*
 * Parsing and %include processing
 */

/// Parses input source code string to a vector of `SourceLine`s
fn parse_source_string(
    input: &str,
    file_name: &PathBuf,
) -> Result<Vec<SourceLine>, Vec<SourceLine>> {
    let mut lines: Vec<SourceLine> = vec![];

    // Keep track of if an error has been encountered
    let mut error_encountered = false;

    // Get all lines from file
    for (line_n, raw_line) in input.lines().enumerate() {
        // Parse line
        let (_, (body, comment, garbage)) = match parse_source_line(raw_line) {
            Ok(line) => line,
            Err(err) => {
                match err {
                    Err::Failure(err) => print_preprocessor_error(PreprocessorError {
                        kind: err.kind,
                        location: Some(LineLocation {
                            line_n: line_n + 1,
                            file_name: file_name.clone(),
                            raw_content: raw_line.to_owned(),
                        }),
                    }),
                    Err::Error(err) => print_preprocessor_error(PreprocessorError {
                        kind: err.kind,
                        location: Some(LineLocation {
                            line_n: line_n + 1,
                            file_name: file_name.clone(),
                            raw_content: raw_line.to_owned(),
                        }),
                    }),

                    Err::Incomplete(_) => {}
                }

                // An error has been encountered
                error_encountered = true;

                continue;
            }
        };

        // Check if there is still unparsed stuff
        if garbage.len() > 0 {
            print_preprocessor_warning(PreprocessorWarning {
                line_n: line_n + 1,
                file_name: file_name.clone(),
                line: raw_line.to_owned(),
                kind: PreprocessorWarningKind::GarbageAtEndOfLine(garbage.to_owned()),
            });
        }

        // If the line is empty, ignore it
        if let SourceLineBody::Empty = body {
            // continue;
        }

        // Add line to list
        lines.push(SourceLine {
            body,
            comment,
            location: LineLocation {
                line_n: line_n + 1,
                file_name: file_name.clone(),
                raw_content: raw_line.to_owned(),
            },
        });
    }

    // Return results
    match error_encountered {
        false => Ok(lines),
        true => Err(lines),
    }
    // Ok(lines)
}

fn process_includes(
    input: &Vec<SourceLine>,
    fi_coord: &mut FileInclusionCoordinator,
) -> Result<Vec<SourceLine>, Vec<SourceLine>> {
    let mut output: Vec<SourceLine> = vec![];
    let mut error = false;

    for line in input {
        // If line is include, resolve it. Otherwise copy line
        if let SourceLineBody::Include(file_path) = &line.body {
            // Process new file
            let mut included_lines =
                match parse_source_file(&file_path, fi_coord, &Some(line.location.clone())) {
                    Ok(lines) => lines,
                    Err(lines) => {
                        error = true;
                        lines
                    }
                };
            output.append(&mut included_lines)
        } else {
            output.push(line.clone())
        }
    }

    // Return results
    match error {
        false => Ok(output),
        true => Err(output),
    }
}

/*
 * Nom parsers
 */

/*
 * Definition processing
 */

fn process_defines(input: &[SourceLine]) -> Result<Vec<SourceLine>, Vec<SourceLine>> {
    let mut res: Vec<SourceLine> = vec![];
    let mut error_encountered = false;

    // Definition table
    let mut definition_table: HashMap<String, String> = HashMap::new();

    // Construct definition table
    for line in input {
        if let SourceLineBody::Define(identifier, value) = &line.body {
            match definition_table.get(identifier) {
                // Check if this identifier was already defined
                Some(_) => {
                    print_preprocessor_error(PreprocessorError {
                        kind: PreprocessorErrorKind::DefineMultipleDefinition(identifier.clone()),
                        location: Some(line.location.clone()),
                    });
                    error_encountered = true;
                }
                None => {
                    definition_table.insert(identifier.clone().to_lowercase(), value.clone());
                }
            }
        }
    }

    // Resolve definitions
    for line in input {
        if let SourceLineBody::Code(code) = &line.body {
            match resolve_defines(&code, &definition_table, &line.location) {
                Ok(code) => res.push(SourceLine {
                    body: SourceLineBody::Code(code),
                    comment: line.comment.clone(),
                    location: line.location.clone(),
                }),
                Err(_) => error_encountered = true,
            }
        }
    }

    match error_encountered {
        false => Ok(res),
        true => Err(res),
    }
}

#[derive(Debug)]
pub enum DefinitionChunk {
    Code(String),
    DefinitionReference(String),
}

fn resolve_defines(
    code: &str,
    def_table: &HashMap<String, String>,
    location: &LineLocation,
) -> Result<String, ()> {
    let (_, chunks) = match parse_definitions_chunks(code) {
        Ok(chunks) => chunks,
        Err(_) => return Err(()),
    };

    let mut result: String = "".to_owned();

    for chunk in chunks {
        match chunk {
            DefinitionChunk::Code(code) => result.push_str(&code),
            DefinitionChunk::DefinitionReference(identifier) => {
                // Look for definition in table
                match def_table.get(&identifier.to_lowercase()) {
                    Some(value) => result.push_str(&value),
                    None => {
                        print_preprocessor_error(PreprocessorError {
                            kind: PreprocessorErrorKind::DefineUndefined(identifier),
                            location: Some(location.clone()),
                        });
                        return Err(());
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Converts the preprocessor output to an string to be written to a file
pub fn convert_preprocessor_output(input: &[CodeLine]) -> String {
    let mut string = String::new();

    for code_line in input {
        string.push_str(&code_line.body);
        string.push_str(" ");
        string.push_str(&code_line.comment);
        string.push_str("\r\n");
    }

    string
}
