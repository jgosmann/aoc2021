use crate::instruction_set::{self, Register};
use std::{
    borrow::Borrow,
    fmt::Display,
    ops::{Index, IndexMut},
    rc::Rc,
};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Mul,
    Div,
    Mod,
    Eql,
    Neq,
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BinaryOp::*;
        let op = match self {
            Add => "+",
            Mul => "*",
            Div => "/",
            Mod => "%",
            Eql => "==",
            Neq => "!=",
        };
        f.write_str(op)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Node {
    Const(i64),
    Inp(usize),
    BinaryOp {
        op: BinaryOp,
        lhs: Rc<Node>,
        rhs: Rc<Node>,
    },
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Const(value) => f.write_fmt(format_args!("{}", value)),
            Node::Inp(index) => f.write_fmt(format_args!("Inp[{}]", index)),
            Node::BinaryOp { op, lhs, rhs } => match op {
                BinaryOp::Add => f.write_fmt(format_args!("({} {} {})", lhs, op, rhs)),
                BinaryOp::Eql => f.write_fmt(format_args!("({} {} {})", lhs, op, rhs)),
                op => f.write_fmt(format_args!("{} {} {}", lhs, op, rhs)),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolicAlu {
    pub w: Rc<Node>,
    pub x: Rc<Node>,
    pub y: Rc<Node>,
    pub z: Rc<Node>,
    input_count: usize,
}

impl Index<&Register> for SymbolicAlu {
    type Output = Rc<Node>;

    fn index(&self, index: &Register) -> &Self::Output {
        match index {
            Register::W => &self.w,
            Register::X => &self.x,
            Register::Y => &self.y,
            Register::Z => &self.z,
        }
    }
}

impl IndexMut<&Register> for SymbolicAlu {
    fn index_mut(&mut self, index: &Register) -> &mut Self::Output {
        match index {
            Register::W => &mut self.w,
            Register::X => &mut self.x,
            Register::Y => &mut self.y,
            Register::Z => &mut self.z,
        }
    }
}

impl Default for SymbolicAlu {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolicAlu {
    pub fn new() -> Self {
        Self {
            w: Rc::new(Node::Const(0)),
            x: Rc::new(Node::Const(0)),
            y: Rc::new(Node::Const(0)),
            z: Rc::new(Node::Const(0)),
            input_count: 0,
        }
    }

    pub fn extract_z(self) -> Rc<Node> {
        self.z
    }

    pub fn execute(&mut self, op: &instruction_set::Op) {
        use instruction_set::{Op, Operand};

        let node_from_operand = |operand: &Operand| match operand {
            Operand::Const(value) => Rc::new(Node::Const(*value)),
            Operand::Register(register) => Rc::clone(self.index(register)),
        };

        let construct_binary_op = |op, lhs, rhs: &Operand| {
            Rc::new(Node::BinaryOp {
                op,
                lhs: Rc::clone(self.index(lhs)),
                rhs: node_from_operand(rhs),
            })
        };

        match op {
            Op::Inp(register) => {
                *self.index_mut(register) = Rc::new(Node::Inp(self.input_count));
                self.input_count += 1;
            }
            Op::Add(_, Operand::Const(0)) => (),
            Op::Add(lhs, rhs) => {
                *self.index_mut(lhs) = match self.index(lhs).borrow() {
                    Node::Const(0) => node_from_operand(rhs),
                    _ => construct_binary_op(BinaryOp::Add, lhs, rhs),
                }
            }
            Op::Mul(lhs, Operand::Const(0)) => *self.index_mut(lhs) = Rc::new(Node::Const(0)),
            Op::Mul(_, Operand::Const(1)) => (),
            Op::Mul(lhs, Operand::Const(rhs)) => {
                *self.index_mut(lhs) =
                    construct_binary_op(BinaryOp::Mul, lhs, &Operand::Const(*rhs))
            }
            Op::Mul(lhs, Operand::Register(rhs)) => {
                *self.index_mut(lhs) = match (self.index(lhs).borrow(), self.index(rhs).borrow()) {
                    (_, Node::Const(0)) => Rc::new(Node::Const(0)),
                    (Node::Const(0), _) => Rc::new(Node::Const(0)),
                    (_, Node::Const(1)) => Rc::clone(self.index(lhs)),
                    (Node::Const(1), _) => Rc::clone(self.index(rhs)),
                    _ => construct_binary_op(BinaryOp::Mul, lhs, &Operand::Register(*rhs)),
                }
            }
            Op::Div(_, Operand::Const(1)) => (),
            Op::Div(lhs, rhs) => {
                *self.index_mut(lhs) = construct_binary_op(BinaryOp::Div, lhs, rhs)
            }
            Op::Mod(lhs, rhs) => {
                *self.index_mut(lhs) = match self.index(lhs).borrow() {
                    Node::Const(0) => Rc::new(Node::Const(0)),
                    _ => construct_binary_op(BinaryOp::Mod, lhs, rhs),
                }
            }
            Op::Eql(lhs, Operand::Register(rhs)) => {
                *self.index_mut(lhs) = match (self.index(lhs).borrow(), self.index(rhs).borrow()) {
                    (Node::Const(x), Node::Inp(_)) if *x < 1 || *x > 9 => Rc::new(Node::Const(0)),
                    (Node::Const(x), Node::Const(y)) => {
                        Rc::new(Node::Const(if x == y { 1 } else { 0 }))
                    }
                    _ => construct_binary_op(BinaryOp::Eql, lhs, &Operand::Register(*rhs)),
                }
            }
            Op::Eql(lhs, Operand::Const(rhs)) => {
                *self.index_mut(lhs) = match (self.index(lhs).borrow(), rhs) {
                    (
                        Node::BinaryOp {
                            op: BinaryOp::Eql,
                            lhs: inner_lhs,
                            rhs: inner_rhs,
                        },
                        0,
                    ) => Rc::new(Node::BinaryOp {
                        op: BinaryOp::Neq,
                        lhs: Rc::clone(inner_lhs),
                        rhs: Rc::clone(inner_rhs),
                    }),
                    (Node::Const(x), _) => Rc::new(Node::Const(if x == rhs { 1 } else { 0 })),
                    _ => construct_binary_op(BinaryOp::Eql, lhs, &Operand::Const(*rhs)),
                }
            }
        }
    }
}
