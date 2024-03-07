use super::statements::{
    AddressExpression, BranchLocation, BranchOnIndicatorsOpCode, BranchOnRegistersOpCode,
    DataRegister, Mnemonic, ShortValueImmediateOpCode, Statement,
};
use crate::{assembler::statements::StatementKind, logging::AssemblerErrorKind};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case, take_while1},
    character::complete::{digit1, hex_digit1, space0},
    combinator::{map, map_res, opt, value},
    error::{ErrorKind, ParseError},
    multi::separated_list0,
    sequence::{delimited, preceded, terminated, tuple},
    Err, IResult,
};

#[derive(Debug)]
pub struct AssemblerParseError<'a> {
    _input: &'a str,
    pub kind: AssemblerErrorKind,
}

impl<'a> ParseError<&'a str> for AssemblerParseError<'a> {
    fn from_error_kind(input: &'a str, kind: ErrorKind) -> Self {
        Self {
            _input: input,
            kind: AssemblerErrorKind::Nom(kind),
        }
    }

    fn append(_input: &'a str, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

pub fn parse_label(input: &str) -> IResult<&str, String> {
    map(terminated(parse_label_identifier, tag(":")), |label| {
        label.to_uppercase()
    })(input)
}

fn parse_label_identifier(input: &str) -> IResult<&str, String> {
    map(
        tuple((
            take_while1(|chr: char| -> bool { chr.is_alphabetic() || chr == '_' }),
            opt(take_while1(|chr: char| -> bool {
                chr.is_alphanumeric() || chr == '_'
            })),
        )),
        |(a, b): (&str, Option<&str>)| match b {
            Some(b) => a.to_owned() + b,
            None => a.to_owned(),
        },
    )(input)
}

pub fn parse_statement(input: &str) -> IResult<&str, Statement, AssemblerParseError> {
    // Extract mnemonic and args
    let (input, (mnemonic, args)) = parse_mnemonic_and_args(input)?;

    // Encapsulate statement
    let statement = match encapsulate_statement(&mnemonic, &args) {
        Ok(statement) => statement,
        Err(kind) => {
            return Err(Err::Failure(AssemblerParseError {
                _input: input,
                kind,
            }))
        }
    };

    // println!(
    //     "Mnemonic: {}, args: {:?}, statement: {:?}",
    //     mnemonic, args, statement
    // );

    Ok(("", statement))
}

fn parse_mnemonic_and_args(input: &str) -> IResult<&str, (&str, Vec<&str>), AssemblerParseError> {
    // Store intermediary result for type annotations
    let mnemo_res: IResult<&str, &str> = is_not(" ,")(input);

    // Get mnemonic
    let (input, mnemonic) = match mnemo_res {
        Ok(res) => res,
        Err(_) => {
            return Err(Err::Failure(AssemblerParseError {
                _input: input,
                kind: AssemblerErrorKind::MnemonicRequired,
            }))
        }
    };

    // Get arguments
    let (input, arguments) =
        separated_list0(tag(","), delimited(space0, is_not(","), space0))(input)?;

    Ok((input, (mnemonic, arguments)))
}

fn encapsulate_statement(
    mnemonic_str: &str,
    args: &[&str],
) -> Result<Statement, AssemblerErrorKind> {
    // Match mnemonic to list of mnemonics
    let mnemo = match match_mnemonic(&mnemonic_str.to_uppercase()) {
        Ok(mnemonic) => mnemonic,
        Err(()) => return Err(AssemblerErrorKind::UnkownMnemonic(mnemonic_str.to_owned())),
    };

    match mnemo.get_kind() {
        StatementKind::Org => encapsulate_org_statement(args),
        StatementKind::BranchOnIndicators => {
            encapsulate_branch_on_indicators_statement(mnemo, args)
        }
        StatementKind::BranchOnRegisters => encapsulate_branch_on_registers_statement(mnemo, args),
        StatementKind::ShortValueImmediate => {
            encapsulate_short_value_immediate_statement(mnemo, args)
        }
    }
}

/// Matches a mnemonic string to its enum type
fn match_mnemonic(input: &str) -> Result<Mnemonic, ()> {
    match input {
        // Assembler directives
        ".ORG" => Ok(Mnemonic::DotORG),

        // Branch on Indicators instructions
        "BL" => Ok(Mnemonic::BL),
        "BGE" => Ok(Mnemonic::BGE),
        "BG" => Ok(Mnemonic::BG),
        "BLE" => Ok(Mnemonic::BLE),
        "BOV" => Ok(Mnemonic::BOV),
        "BNOV" => Ok(Mnemonic::BNOV),
        "BBT" => Ok(Mnemonic::BBT),
        "BBF" => Ok(Mnemonic::BBF),
        "BCT" => Ok(Mnemonic::BCT),
        "BCF" => Ok(Mnemonic::BCF),
        "BIOT" => Ok(Mnemonic::BIOT),
        "BIOF" => Ok(Mnemonic::BIOF),
        "BAL" => Ok(Mnemonic::BAL),
        "BAGE" => Ok(Mnemonic::BAGE),
        "BE" => Ok(Mnemonic::BE),
        "BNE" => Ok(Mnemonic::BNE),
        "BAG" => Ok(Mnemonic::BAG),
        "BALE" => Ok(Mnemonic::BALE),
        "BSU" => Ok(Mnemonic::BSU),
        "BSE" => Ok(Mnemonic::BSE),
        "B" => Ok(Mnemonic::B),

        // Branch on Registers instructions
        "BLZ" => Ok(Mnemonic::BLZ),
        "BGEZ" => Ok(Mnemonic::BGEZ),
        "BEZ" => Ok(Mnemonic::BEZ),
        "BNEZ" => Ok(Mnemonic::BNEZ),
        "BGZ" => Ok(Mnemonic::BGZ),
        "BLEZ" => Ok(Mnemonic::BLEZ),
        "BODD" => Ok(Mnemonic::BODD),
        "BEVN" => Ok(Mnemonic::BEVN),
        "BINC" => Ok(Mnemonic::BINC),
        "BDEC" => Ok(Mnemonic::BDEC),

        // Short Value Immediate instructions
        "LDV" => Ok(Mnemonic::LDV),
        "CMV" => Ok(Mnemonic::CMV),
        "ADV" => Ok(Mnemonic::ADV),
        "MLV" => Ok(Mnemonic::MLV),
        _ => Err(()),
    }
}

fn encapsulate_org_statement(args: &[&str]) -> Result<Statement, AssemblerErrorKind> {
    // Check number of arguments
    if args.len() != 1 {
        return Err(AssemblerErrorKind::WrongNumberOfArguments(
            Mnemonic::DotORG,
            1,
            args.len(),
        ));
    }

    // Parse address
    let address = parse_hex_address_arg(&args[0])?;

    Ok(Statement::Org(address))
}

fn encapsulate_branch_on_indicators_statement(
    mnemo: Mnemonic,
    args: &[&str],
) -> Result<Statement, AssemblerErrorKind> {
    // Check number of arguments
    if args.len() != 1 {
        return Err(AssemblerErrorKind::WrongNumberOfArguments(
            mnemo,
            1,
            args.len(),
        ));
    }

    // Get op code
    let op = match mnemo {
        Mnemonic::BL => BranchOnIndicatorsOpCode::BL,
        Mnemonic::BGE => BranchOnIndicatorsOpCode::BGE,
        Mnemonic::BG => BranchOnIndicatorsOpCode::BG,
        Mnemonic::BLE => BranchOnIndicatorsOpCode::BLE,
        Mnemonic::BOV => BranchOnIndicatorsOpCode::BOV,
        Mnemonic::BNOV => BranchOnIndicatorsOpCode::BNOV,
        Mnemonic::BBT => BranchOnIndicatorsOpCode::BBT,
        Mnemonic::BBF => BranchOnIndicatorsOpCode::BBF,
        Mnemonic::BCT => BranchOnIndicatorsOpCode::BCT,
        Mnemonic::BCF => BranchOnIndicatorsOpCode::BCF,
        Mnemonic::BIOT => BranchOnIndicatorsOpCode::BIOT,
        Mnemonic::BIOF => BranchOnIndicatorsOpCode::BIOF,
        Mnemonic::BAL => BranchOnIndicatorsOpCode::BAL,
        Mnemonic::BAGE => BranchOnIndicatorsOpCode::BAGE,
        Mnemonic::BE => BranchOnIndicatorsOpCode::BE,
        Mnemonic::BNE => BranchOnIndicatorsOpCode::BNE,
        Mnemonic::BAG => BranchOnIndicatorsOpCode::BAG,
        Mnemonic::BALE => BranchOnIndicatorsOpCode::BALE,
        Mnemonic::BSU => BranchOnIndicatorsOpCode::BSU,
        Mnemonic::BSE => BranchOnIndicatorsOpCode::BSE,
        Mnemonic::B => BranchOnIndicatorsOpCode::B,
        _ => panic!("invalid OpCode for BranchOnIndicators"),
    };

    // Parse branch location
    let branchloc = parse_branch_location_arg(&args[0])?;

    Ok(Statement::BranchOnIndicators(op, branchloc))
}

fn encapsulate_branch_on_registers_statement(
    mnemo: Mnemonic,
    args: &[&str],
) -> Result<Statement, AssemblerErrorKind> {
    // Check number of arguments
    if args.len() != 2 {
        return Err(AssemblerErrorKind::WrongNumberOfArguments(
            mnemo,
            2,
            args.len(),
        ));
    }

    // Get op code
    let op = match mnemo {
        Mnemonic::BLZ => BranchOnRegistersOpCode::BLZ,
        Mnemonic::BGEZ => BranchOnRegistersOpCode::BGEZ,
        Mnemonic::BEZ => BranchOnRegistersOpCode::BEZ,
        Mnemonic::BNEZ => BranchOnRegistersOpCode::BNEZ,
        Mnemonic::BGZ => BranchOnRegistersOpCode::BGZ,
        Mnemonic::BLEZ => BranchOnRegistersOpCode::BLEZ,
        Mnemonic::BODD => BranchOnRegistersOpCode::BODD,
        Mnemonic::BEVN => BranchOnRegistersOpCode::BEVN,
        Mnemonic::BINC => BranchOnRegistersOpCode::BINC,
        Mnemonic::BDEC => BranchOnRegistersOpCode::BDEC,
        _ => panic!("invalid OpCode for BranchOnRegisters"),
    };

    // Parse data register
    let reg = parse_data_register_arg(args[0])?;

    // Parse branch location
    let branchloc = parse_branch_location_arg(&args[1])?;

    Ok(Statement::BranchOnRegisters(op, reg, branchloc))
}

fn encapsulate_short_value_immediate_statement(
    mnemo: Mnemonic,
    args: &[&str],
) -> Result<Statement, AssemblerErrorKind> {
    // Check number of arguments
    if args.len() != 2 {
        return Err(AssemblerErrorKind::WrongNumberOfArguments(
            mnemo,
            2,
            args.len(),
        ));
    }

    // Get op code
    let op = match mnemo {
        Mnemonic::LDV => ShortValueImmediateOpCode::LDV,
        Mnemonic::CMV => ShortValueImmediateOpCode::CMV,
        Mnemonic::ADV => ShortValueImmediateOpCode::ADV,
        Mnemonic::MLV => ShortValueImmediateOpCode::MLV,
        _ => panic!("invalid OpCode for ShortValueImmediate"),
    };

    // Parse data register
    let reg = parse_data_register_arg(args[0])?;

    // Parse branch location
    let val = parse_immediate_value_arg(&args[1])?;

    Ok(Statement::ShortValueImmediate(op, reg, val))
}

fn parse_hex_address_arg(input: &str) -> Result<u64, AssemblerErrorKind> {
    // Parse address
    let (input, address) = match parse_hex_u64(input) {
        Ok(address) => address,
        Err(_) => return Err(AssemblerErrorKind::InvalidAddress(input.to_owned())),
    };

    // Check for extra characters
    if input.len() > 0 {
        return Err(AssemblerErrorKind::UnexpectedCharactersAtEndOfArgument(
            input.to_owned(),
        ));
    }

    Ok(address)
}

fn parse_branch_location_arg(input: &str) -> Result<BranchLocation, AssemblerErrorKind> {
    // Parse address
    let (input, branchloc) = match alt((
        parse_branch_location_absolute,
        parse_branch_location_long_relative,
        parse_branch_location_short_relative,
    ))(input)
    {
        Ok(address) => address,
        Err(_) => return Err(AssemblerErrorKind::InvalidBranchLocation(input.to_owned())),
    };

    // Check for extra characters
    if input.len() > 0 {
        return Err(AssemblerErrorKind::UnexpectedCharactersAtEndOfArgument(
            input.to_owned(),
        ));
    }

    Ok(branchloc)
}

fn parse_branch_location_absolute(input: &str) -> IResult<&str, BranchLocation> {
    map(preceded(tag("<"), parse_address_expression), |addr_exp| {
        BranchLocation::Absolute(addr_exp)
    })(input)
}

fn parse_branch_location_long_relative(input: &str) -> IResult<&str, BranchLocation> {
    map(parse_address_expression, |addr_exp| {
        BranchLocation::LongDisplacement(addr_exp)
    })(input)
}

fn parse_branch_location_short_relative(input: &str) -> IResult<&str, BranchLocation> {
    map(preceded(tag(">"), parse_address_expression), |addr_exp| {
        BranchLocation::ShortDisplacement(addr_exp)
    })(input)
}

fn parse_data_register_arg(input: &str) -> Result<DataRegister, AssemblerErrorKind> {
    // Parse address
    let (input, branchloc) = match preceded(tag("$"), parse_data_register)(input) {
        Ok(address) => address,
        Err(_) => return Err(AssemblerErrorKind::InvalidDataRegister(input.to_owned())),
    };

    // Check for extra characters
    if input.len() > 0 {
        return Err(AssemblerErrorKind::UnexpectedCharactersAtEndOfArgument(
            input.to_owned(),
        ));
    }

    Ok(branchloc)
}

fn parse_immediate_value_arg(input: &str) -> Result<i128, AssemblerErrorKind> {
    // Parse address
    let (input, value) = match parse_immediate_value(input) {
        Ok(address) => address,
        Err(_) => return Err(AssemblerErrorKind::InvalidImmediateValue(input.to_owned())),
    };

    // Check for extra characters
    if input.len() > 0 {
        return Err(AssemblerErrorKind::UnexpectedCharactersAtEndOfArgument(
            input.to_owned(),
        ));
    }

    Ok(value)
}

fn parse_data_register(input: &str) -> IResult<&str, DataRegister> {
    alt((
        value(DataRegister::R1, tag_no_case("R1")),
        value(DataRegister::R2, tag_no_case("R2")),
        value(DataRegister::R3, tag_no_case("R3")),
        value(DataRegister::R4, tag_no_case("R4")),
        value(DataRegister::R5, tag_no_case("R5")),
        value(DataRegister::R6, tag_no_case("R6")),
        value(DataRegister::R7, tag_no_case("R7")),
    ))(input)
}

fn parse_address_expression(input: &str) -> IResult<&str, AddressExpression> {
    alt((
        parse_immediate_address_expression,
        parse_label_address_expression,
        parse_displacement_address_expression,
    ))(input)
}

fn parse_immediate_address_expression(input: &str) -> IResult<&str, AddressExpression> {
    map(parse_hex_u64, |addr| AddressExpression::Immediate(addr))(input)
}

fn parse_label_address_expression(input: &str) -> IResult<&str, AddressExpression> {
    map(parse_label_identifier, |label| {
        AddressExpression::Label(label.to_uppercase())
    })(input)
}

fn parse_displacement_address_expression(input: &str) -> IResult<&str, AddressExpression> {
    // Get displacement sign
    let (input, is_positive) = alt((value(true, tag("+")), value(false, tag("-"))))(input)?;

    // Get displacement number
    let (input, disp) = map_res(digit1, |input| u64::from_str_radix(input, 10))(input)?;

    Ok((
        input,
        AddressExpression::WordDisplacement(match is_positive {
            true => disp as i128,
            false => -(disp as i128),
        }),
    ))
}

fn parse_immediate_value(input: &str) -> IResult<&str, i128> {
    preceded(tag("="), parse_immediate_value_contents)(input)
}

fn parse_immediate_value_contents(input: &str) -> IResult<&str, i128> {
    alt((map(parse_hex_u64, |val| val as i128), parse_dec_i128))(input)
}

pub fn parse_hex_u64(input: &str) -> IResult<&str, u64> {
    preceded(
        tag_no_case("0x"),
        map_res(hex_digit1, |digits| u64::from_str_radix(digits, 16)),
    )(input)
}

pub fn parse_dec_i128(input: &str) -> IResult<&str, i128> {
    alt((
        map(preceded(opt(tag("+")), parse_dec_u64), |val| val as i128),
        map(preceded(tag("-"), parse_dec_u64), |val| -(val as i128)),
    ))(input)
}

pub fn parse_dec_u64(input: &str) -> IResult<&str, u64> {
    map_res(digit1, |val| u64::from_str_radix(val, 10))(input)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::assembler::statements::{
        AddressExpression, BranchLocation, BranchOnIndicatorsOpCode,
    };

    use super::*;
    use nom::Err;

    #[test]
    fn parse_label_succ() {
        let tests = [
            (
                "loop: ldr, something, something",
                "LOOP",
                " ldr, something, something",
            ),
            ("also_a_valid_label76:", "ALSO_A_VALID_LABEL76", ""),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_label(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_label_err() {
        let tests = [("", false), ("abcde", false), ("lab&el:", false)];
        for (input, exp_failure) in tests {
            let err = parse_label(input).unwrap_err();

            match err {
                Err::Incomplete(_) => panic!(),
                Err::Error(_) => assert!(!exp_failure),
                Err::Failure(_) => assert!(exp_failure),
            }
        }
    }

    #[test]
    fn parse_hex_u64_succ() {
        let tests = [("0x00 ciaone", 0, " ciaone"), ("0x11", 17, "")];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_hex_u64(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_hex_u64_err() {
        let tests = [
            ("", false),
            ("abcde", false),
            ("   0x123", false),
            ("0x999999999999999999999999999999999", false),
        ];
        for (input, exp_failure) in tests {
            let err = parse_hex_u64(input).unwrap_err();

            match err {
                Err::Incomplete(_) => panic!(),
                Err::Error(_) => assert!(!exp_failure),
                Err::Failure(_) => assert!(exp_failure),
            }
        }
    }

    #[test]
    fn parse_mnemonic_and_args_succ() {
        let tests = [
            (
                "mnemo arg1, arg2, arg3",
                ("mnemo", vec!["arg1", "arg2", "arg3"]),
                "",
            ),
            (".ORG", (".ORG", vec![]), ""),
            // (".org 0x0", Statement::Org(0x0), ""),
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_mnemonic_and_args(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_mnemonic_and_args_err() {
        let tests = [(",", true)];
        for (input, exp_failure) in tests {
            let err = parse_mnemonic_and_args(input).unwrap_err();

            match err {
                Err::Incomplete(_) => panic!(),
                Err::Error(_) => assert!(!exp_failure),
                Err::Failure(_) => assert!(exp_failure),
            }
        }
    }

    #[test]
    fn parse_statement_succ() {
        let tests = [
            // Org
            (".org 0x100", Statement::Org(0x100), ""),
            // BranchOnIndicators
            (
                "BL <0x1234",
                Statement::BranchOnIndicators(
                    BranchOnIndicatorsOpCode::BL,
                    BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
                ),
                "",
            ),
            (
                "BG <0x1234",
                Statement::BranchOnIndicators(
                    BranchOnIndicatorsOpCode::BG,
                    BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
                ),
                "",
            ),
            (
                "BLE <0x1234",
                Statement::BranchOnIndicators(
                    BranchOnIndicatorsOpCode::BLE,
                    BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
                ),
                "",
            ),
            (
                "BOV <0x1234",
                Statement::BranchOnIndicators(
                    BranchOnIndicatorsOpCode::BOV,
                    BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
                ),
                "",
            ),
            (
                "BNOV <0x1234",
                Statement::BranchOnIndicators(
                    BranchOnIndicatorsOpCode::BNOV,
                    BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
                ),
                "",
            ),
            (
                "BBT <0x1234",
                Statement::BranchOnIndicators(
                    BranchOnIndicatorsOpCode::BBT,
                    BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
                ),
                "",
            ),
            (
                "BBF <0x1234",
                Statement::BranchOnIndicators(
                    BranchOnIndicatorsOpCode::BBF,
                    BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
                ),
                "",
            ),
            (
                "BCT <0x1234",
                Statement::BranchOnIndicators(
                    BranchOnIndicatorsOpCode::BCT,
                    BranchLocation::Absolute(AddressExpression::Immediate(0x1234)),
                ),
                "",
            ),
            // TODO finish testing BranchOnIndicators instructions
        ];
        for (input, exp_output, exp_remaining) in tests {
            let (remaining, output) = parse_statement(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(remaining, exp_remaining);
        }
    }

    #[test]
    fn parse_branch_location_arg_succ() {
        let tests = [
            (
                "<0x10",
                BranchLocation::Absolute(AddressExpression::Immediate(0x10)),
            ),
            (
                "<LABEL",
                BranchLocation::Absolute(AddressExpression::Label("LABEL".to_owned())),
            ),
            (
                "<+5",
                BranchLocation::Absolute(AddressExpression::WordDisplacement(5)),
            ),
            (
                "0x99",
                BranchLocation::LongDisplacement(AddressExpression::Immediate(0x99)),
            ),
            (
                "loop",
                BranchLocation::LongDisplacement(AddressExpression::Label("LOOP".to_owned())),
            ),
            (
                "-10",
                BranchLocation::LongDisplacement(AddressExpression::WordDisplacement(-10)),
            ),
            (
                ">0x1234",
                BranchLocation::ShortDisplacement(AddressExpression::Immediate(0x1234)),
            ),
            (
                ">DO_THING",
                BranchLocation::ShortDisplacement(AddressExpression::Label("DO_THING".to_owned())),
            ),
            (
                ">+9999",
                BranchLocation::ShortDisplacement(AddressExpression::WordDisplacement(9999)),
            ),
        ];
        for (input, exp_output) in tests {
            let output = parse_branch_location_arg(input).unwrap();
            assert_eq!(output, exp_output);
        }
    }

    #[test]
    fn parse_data_register_arg_succ() {
        let tests = [
            ("$R1", DataRegister::R1),
            ("$R2", DataRegister::R2),
            ("$R3", DataRegister::R3),
            ("$R4", DataRegister::R4),
            ("$r5", DataRegister::R5),
            ("$r6", DataRegister::R6),
            ("$r7", DataRegister::R7),
        ];
        for (input, exp_output) in tests {
            let output = parse_data_register_arg(input).unwrap();
            assert_eq!(output, exp_output);
        }
    }

    #[test]
    fn parse_data_register_arg_err() {
        let tests = ["notaregister", "", "$R1 ciao"];
        for input in tests {
            parse_data_register_arg(input).unwrap_err();
        }
    }

    #[test]
    fn parse_immediate_value_arg_err() {
        let tests = ["notimmediate", "", "=123 ciao"];
        for input in tests {
            parse_immediate_value_arg(input).unwrap_err();
        }
    }

    #[test]
    fn parse_immediate_value_succ() {
        let tests = [("=10", "", 10), ("=0x1234notnumber", "notnumber", 0x1234)];
        for (input, exp_rem, exp_output) in tests {
            let (input, output) = parse_immediate_value(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(input, exp_rem);
        }
    }

    #[test]
    fn parse_immediate_value_err() {
        let tests = ["1234", "0x123"];
        for input in tests {
            parse_immediate_value(input).unwrap_err();
        }
    }

    #[test]
    fn parse_immediate_value_contents_succ() {
        let tests = [
            ("1", "", 1),
            ("-1", "", -1),
            ("+9999", "", 9999),
            ("0x100", "", 256),
            ("1234ciao", "ciao", 1234),
            ("-1234  ", "  ", -1234),
            ("0x00000000000,test", ",test", 0),
        ];
        for (input, exp_rem, exp_output) in tests {
            let (input, output) = parse_immediate_value_contents(input).unwrap();
            assert_eq!(output, exp_output);
            assert_eq!(input, exp_rem);
        }
    }

    #[test]
    fn parse_immediate_value_contents_err() {
        let tests = ["notimmediate", ""];
        for input in tests {
            parse_immediate_value_contents(input).unwrap_err();
        }
    }
}
