use std::{collections::HashSet, error::Error, fmt::Display};

use crate::ast::{DeduplicatedAst, Evaluator, Node};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct MemoKey<const N: usize> {
    num_nodes_evaluated: usize,
    dependencies_of_unevaluated_nodes: [i64; N],
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SearchMode {
    Largest,
    Smallest,
}

impl SearchMode {
    fn search_range(&self) -> &[i64] {
        match self {
            SearchMode::Largest => &[9, 8, 7, 6, 5, 4, 3, 2, 1],
            SearchMode::Smallest => &[1, 2, 3, 4, 5, 6, 7, 8, 9],
        }
    }
}

#[derive(Debug)]
pub struct InsufficientMemoKeySize;

impl Display for InsufficientMemoKeySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("insufficient memo key size")
    }
}

impl Error for InsufficientMemoKeySize {}

pub struct ModelNumberSearch<'ast, const N: usize> {
    evaluator: Evaluator<'ast>,
    node_dependencies: Vec<Vec<usize>>,
    exhausted_branches: HashSet<MemoKey<N>>,
}

impl<'ast, const N: usize> ModelNumberSearch<'ast, N> {
    pub fn new(ast: &'ast DeduplicatedAst) -> Result<Self, InsufficientMemoKeySize> {
        let node_dependencies: Vec<Vec<usize>> = ast
            .nodes()
            .iter()
            .map(Node::dependencies)
            .enumerate()
            .rev()
            .fold(
                vec![vec![]; ast.nodes().len()],
                |mut accum, (i, dependencies)| {
                    let mut dependency_set: HashSet<usize> = if i + 1 < accum.len() {
                        accum[i + 1]
                            .iter()
                            .copied()
                            .filter(|&dependency| dependency < i)
                            .collect()
                    } else {
                        HashSet::new()
                    };
                    dependency_set.extend(dependencies.iter());
                    accum[i] = dependency_set.into_iter().collect();
                    accum
                },
            );

        if node_dependencies.iter().map(Vec::len).any(|l| l > N) {
            return Err(InsufficientMemoKeySize);
        }

        Ok(Self {
            evaluator: Evaluator::new(ast),
            node_dependencies,
            exhausted_branches: HashSet::with_capacity(20_000_000),
        })
    }

    fn memo_key(&self) -> MemoKey<N> {
        let num_nodes_evaluated = self.evaluator.num_nodes_evaluated();
        let mut dependencies_of_unevaluated_nodes: Vec<i64> = Vec::with_capacity(N);
        dependencies_of_unevaluated_nodes.extend(
            self.node_dependencies[num_nodes_evaluated]
                .iter()
                .map(|&index| self.evaluator.cached_values()[index]),
        );
        dependencies_of_unevaluated_nodes.resize(N, 0);
        MemoKey {
            num_nodes_evaluated,
            dependencies_of_unevaluated_nodes: dependencies_of_unevaluated_nodes
                .try_into()
                .unwrap(),
        }
    }

    pub fn find_model_number(&mut self, search_mode: SearchMode) -> Option<String> {
        let memo_key = self.memo_key();
        if self.exhausted_branches.contains(&memo_key) {
            return None;
        }

        let mut result = None;
        for input in search_mode.search_range() {
            self.evaluator.push_input(*input);

            if let Some(eval_result) = self.evaluator.result() {
                if eval_result == 0 {
                    result = Some(
                        self.evaluator
                            .inputs()
                            .iter()
                            .map(i64::to_string)
                            .collect::<Vec<_>>()
                            .join(""),
                    );
                }
            } else {
                result = self.find_model_number(search_mode);
            }

            self.evaluator.pop_input();

            if result.is_some() {
                return result;
            }
        }

        self.exhausted_branches.insert(memo_key);
        None
    }
}
