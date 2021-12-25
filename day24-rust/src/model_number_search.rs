use std::collections::HashSet;

use crate::ast::{DeduplicatedAst, Evaluator, Node};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
struct Dependency {
    index: usize,
    value: i64,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct MemoKey {
    num_nodes_evaluated: usize,
    dependencies_of_unevaluated_nodes: Vec<Dependency>,
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
        let node_dependencies = ast.nodes().iter().map(Node::dependencies).collect();

        Self {
            evaluator: Evaluator::new(ast),
            node_dependencies,
            exhausted_branches: HashSet::new(),
        }
    }

    fn memo_key(&self) -> MemoKey {
        let num_nodes_evaluated = self.evaluator.num_nodes_evaluated();
        let unevaluated_nodes_iter = self.node_dependencies.iter().skip(num_nodes_evaluated);
        let dependencies: HashSet<Dependency> = unevaluated_nodes_iter
            .flat_map(|node_deps| {
                node_deps.iter().filter_map(|&dep_index| {
                    self.evaluator
                        .get_cached(dep_index)
                        .map(|dep_value| Dependency {
                            index: dep_index,
                            value: dep_value,
                        })
                })
            })
            .collect();
        let mut dependencies = Vec::from_iter(dependencies.into_iter());
        dependencies.sort_by_key(|d| d.index);
        MemoKey {
            num_nodes_evaluated,
            dependencies_of_unevaluated_nodes: dependencies,
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
