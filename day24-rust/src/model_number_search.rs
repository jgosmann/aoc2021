use std::collections::HashSet;

use crate::ast::{DeduplicatedAst, Evaluator, Node};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct MemoKey {
    num_nodes_evaluated: usize,
    dependencies_of_unevaluated_nodes: Vec<i64>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SearchMode {
    Largest,
    Smallest,
}

impl SearchMode {
    fn search_range(&self) -> Box<dyn Iterator<Item = i64>> {
        match self {
            SearchMode::Largest => Box::new((1..=9).rev()),
            SearchMode::Smallest => Box::new(1..=9),
        }
    }
}

pub struct ModelNumberSearch<'ast> {
    evaluator: Evaluator<'ast>,
    node_dependencies: Vec<Vec<usize>>,
    exhausted_branches: HashSet<MemoKey>,
}

impl<'ast> ModelNumberSearch<'ast> {
    pub fn new(ast: &'ast DeduplicatedAst) -> Self {
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

        Self {
            evaluator: Evaluator::new(ast),
            node_dependencies,
            exhausted_branches: HashSet::with_capacity(20_000_000),
        }
    }

    fn memo_key(&self) -> MemoKey {
        let num_nodes_evaluated = self.evaluator.num_nodes_evaluated();
        let dependencies_of_unevaluated_nodes = self.node_dependencies[num_nodes_evaluated]
            .iter()
            .map(|&index| self.evaluator.cached_values()[index])
            .collect();
        MemoKey {
            num_nodes_evaluated,
            dependencies_of_unevaluated_nodes,
        }
    }

    pub fn find_model_number(&mut self, search_mode: SearchMode) -> Option<String> {
        let memo_key = self.memo_key();
        if self.exhausted_branches.contains(&memo_key) {
            return None;
        }

        let mut result = None;
        for input in search_mode.search_range() {
            self.evaluator.push_input(input);

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
