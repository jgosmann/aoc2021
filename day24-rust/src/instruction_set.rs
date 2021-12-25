use std::{error::Error, fmt::Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    W,
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operand {
    Const(i64),
    Register(Register),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Inp(Register),
    Add(Register, Operand),
    Mul(Register, Operand),
    Div(Register, Operand),
    Mod(Register, Operand),
    Eql(Register, Operand),
}

#[derive(Debug)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("parse error")
    }
}

impl Error for ParseError {}

impl TryFrom<&str> for Register {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "w" => Ok(Self::W),
            "x" => Ok(Self::X),
            "y" => Ok(Self::Y),
            "z" => Ok(Self::Z),
            _ => Err(ParseError),
        }
    }
}

impl TryFrom<&str> for Operand {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse().map_or_else(
            |_| Ok(Self::Register(Register::try_from(value)?)),
            |parsed_value| Ok(Self::Const(parsed_value)),
        )
    }
}

impl TryFrom<&str> for Op {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (op_token, remainder) = value.split_once(' ').ok_or(ParseError)?;
        match op_token.trim() {
            "inp" => Ok(Self::Inp(Register::try_from(remainder)?)),
            op_token => {
                let (op0, op1) = remainder.trim().split_once(' ').ok_or(ParseError)?;
                let op0 = Register::try_from(op0.trim())?;
                let op1 = Operand::try_from(op1.trim())?;
                match op_token {
                    "add" => Ok(Self::Add(op0, op1)),
                    "mul" => Ok(Self::Mul(op0, op1)),
                    "div" => Ok(Self::Div(op0, op1)),
                    "mod" => Ok(Self::Mod(op0, op1)),
                    "eql" => Ok(Self::Eql(op0, op1)),
                    _ => Err(ParseError),
                }
            }
        }
    }
}
