use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;

use super::symbolic_alu::Node as AluNode;
pub use crate::symbolic_alu::BinaryOp;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Node {
    Const(i64),
    Inp(usize),
    Ref(usize),
    BinaryOp {
        op: BinaryOp,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}

impl Node {
    pub fn inputs(&self) -> Vec<usize> {
        match self {
            Node::Inp(x) => vec![*x],
            Node::BinaryOp { op: _, lhs, rhs } => {
                let mut result = lhs.inputs();
                result.append(&mut rhs.inputs());
                result
            }
            _ => vec![],
        }
    }

    pub fn dependencies(&self) -> Vec<usize> {
        match self {
            Node::Ref(x) => vec![*x],
            Node::BinaryOp { op: _, lhs, rhs } => {
                let mut dependencies = lhs.dependencies();
                dependencies.append(&mut rhs.dependencies());
                dependencies
            }
            _ => vec![],
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Const(value) => f.write_fmt(format_args!("{}", value)),
            Node::Inp(index) => f.write_fmt(format_args!("Inp[{}]", index)),
            Node::Ref(index) => f.write_fmt(format_args!("Ref[{}]", index)),
            Node::BinaryOp { op, lhs, rhs } => match op {
                BinaryOp::Add => f.write_fmt(format_args!("({} {} {})", lhs, op, rhs)),
                BinaryOp::Eql => f.write_fmt(format_args!("({} {} {})", lhs, op, rhs)),
                op => f.write_fmt(format_args!("{} {} {}", lhs, op, rhs)),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeduplicatedAst {
    nodes: Vec<Node>,
}

impl DeduplicatedAst {
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn num_inputs(&self) -> usize {
        self.max_input().map(|x| x + 1).unwrap_or(0)
    }

    fn max_input(&self) -> Option<usize> {
        self.nodes.iter().flat_map(Node::inputs).max()
    }

    fn process_alu_node(
        &mut self,
        node: &Rc<AluNode>,
        mapping: &mut HashMap<Rc<AluNode>, usize>,
    ) -> Node {
        if let Some(index) = mapping.get(node) {
            return Node::Ref(*index);
        }

        let converted = match node.deref() {
            AluNode::Const(x) => Node::Const(*x),
            AluNode::Inp(x) => Node::Inp(*x),
            AluNode::BinaryOp { op, lhs, rhs } => Node::BinaryOp {
                op: *op,
                lhs: Box::new(self.process_alu_node(lhs, mapping)),
                rhs: Box::new(self.process_alu_node(rhs, mapping)),
            },
        };

        if Rc::strong_count(node) > 1 {
            mapping.insert(Rc::clone(node), self.nodes.len());
            self.nodes.push(converted);
            Node::Ref(self.nodes.len() - 1)
        } else {
            converted
        }
    }
}

impl From<&Rc<AluNode>> for DeduplicatedAst {
    fn from(source: &Rc<AluNode>) -> Self {
        let mut compact_ast = Self { nodes: vec![] };
        let root = compact_ast.process_alu_node(source, &mut HashMap::new());
        compact_ast.nodes.push(root);
        compact_ast
    }
}

impl Display for DeduplicatedAst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, node) in self.nodes.iter().enumerate() {
            f.write_fmt(format_args!("Node[{}]: {}\n", i, node))?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Evaluator<'a> {
    ast: &'a DeduplicatedAst,
    inputs: Vec<i64>,
    input_to_node: Vec<usize>,
    cached_values: Vec<i64>,
}

impl<'a> Evaluator<'a> {
    pub fn new(ast: &'a DeduplicatedAst) -> Self {
        let num_inputs = ast.num_inputs();
        let mut input_to_node = vec![ast.nodes().len() - 1; num_inputs];
        for (i, node) in ast.nodes().iter().enumerate() {
            for input in node.inputs() {
                input_to_node[input] = i;
            }
        }

        Self {
            ast,
            inputs: vec![],
            input_to_node,
            cached_values: vec![],
        }
    }

    pub fn num_nodes_evaluated(&self) -> usize {
        self.cached_values.len()
    }

    pub fn get_cached(&self, index: usize) -> Option<i64> {
        self.cached_values.get(index).copied()
    }

    pub fn result(&self) -> Option<i64> {
        if self.cached_values.len() == self.ast.nodes().len() {
            self.cached_values.last().copied()
        } else {
            None
        }
    }

    pub fn inputs(&self) -> &[i64] {
        &self.inputs
    }

    pub fn push_input(&mut self, value: i64) {
        self.inputs.push(value);
        let eval_to_node = self
            .input_to_node
            .get(self.inputs.len())
            .copied()
            .unwrap_or(self.ast.nodes().len());
        for i in self.cached_values.len()..eval_to_node {
            let value = self.eval_node_index(i);
            self.cached_values.push(value);
        }
    }

    pub fn pop_input(&mut self) {
        self.inputs.pop();
        let from_node = self
            .input_to_node
            .get(self.inputs.len())
            .copied()
            .unwrap_or(self.ast.nodes().len());
        self.cached_values.truncate(from_node);
    }

    fn eval_node(&mut self, node: &Node) -> i64 {
        match node {
            Node::Const(x) => *x,
            Node::Inp(index) => self.inputs[*index],
            Node::Ref(index) => self.eval_node_index(*index),
            Node::BinaryOp { op, lhs, rhs } => match op {
                BinaryOp::Add => self.eval_node(lhs) + self.eval_node(rhs),
                BinaryOp::Mul => self.eval_node(lhs) * self.eval_node(rhs),
                BinaryOp::Div => self.eval_node(lhs) / self.eval_node(rhs),
                BinaryOp::Mod => self.eval_node(lhs) % self.eval_node(rhs),
                BinaryOp::Eql => {
                    if self.eval_node(lhs) == self.eval_node(rhs) {
                        1
                    } else {
                        0
                    }
                }
            },
        }
    }

    fn eval_node_index(&mut self, index: usize) -> i64 {
        if index < self.cached_values.len() {
            return self.cached_values[index];
        }

        self.eval_node(&self.ast.nodes()[index])
    }
}
