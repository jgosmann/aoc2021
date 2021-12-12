use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::error::Error;
use std::fmt::Display;
use std::io::{self, BufRead};

type Node = String;

#[derive(Clone, Debug, PartialEq, Eq)]
struct UndirectedGraph {
    edges: HashMap<Node, HashSet<Node>>,
    empty_neighbors: HashSet<Node>,
}

impl UndirectedGraph {
    fn new() -> Self {
        Self {
            edges: HashMap::new(),
            empty_neighbors: HashSet::new(),
        }
    }

    fn insert(&mut self, from: Node, to: Node) {
        self.edges
            .entry(from.clone())
            .or_insert(HashSet::new())
            .insert(to.clone());
        self.edges.entry(to).or_insert(HashSet::new()).insert(from);
    }

    fn get(&self, node: &Node) -> &HashSet<Node> {
        self.edges.get(node).unwrap_or(&self.empty_neighbors)
    }
}

fn count_cave_paths<'a>(
    graph: &'a UndirectedGraph,
    from: &'a Node,
    excluded: &mut HashSet<&'a Node>,
    may_visit_single_excluded: bool,
) -> u64 {
    if from == "end" {
        return 1;
    }

    let is_small_cave = from.chars().all(char::is_lowercase);
    let remove_from_excluded = if is_small_cave {
        excluded.insert(&from)
    } else {
        false
    };

    let count = graph
        .get(&from)
        .iter()
        .map(|neighbor| {
            if neighbor == "start" {
                return 0;
            }

            if excluded.contains(&neighbor) && !may_visit_single_excluded {
                return 0;
            }

            let count = count_cave_paths(
                graph,
                &neighbor,
                excluded,
                may_visit_single_excluded && !excluded.contains(&neighbor),
            );
            return count;
        })
        .sum();

    if remove_from_excluded {
        excluded.remove(from);
    }

    count
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Parse error")
    }
}

impl Error for ParseError {}

fn read_undirected_graph<R: BufRead>(reader: &mut R) -> Result<UndirectedGraph, Box<dyn Error>> {
    let mut graph = UndirectedGraph::new();
    for line in reader.lines() {
        let parts: Vec<String> = line?.split('-').map(&str::trim).map(String::from).collect();
        let [from, to]: [String; 2] = parts.try_into().map_err(|_| ParseError)?;
        graph.insert(from, to);
    }
    Ok(graph)
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let graph = read_undirected_graph(&mut stdin.lock())?;
    println!(
        "Path count (revisiting small caves disallowed): {}",
        count_cave_paths(&graph, &"start".to_string(), &mut HashSet::new(), false)
    );
    println!(
        "Path count (may visit single small cave twice): {}",
        count_cave_paths(&graph, &"start".to_string(), &mut HashSet::new(), true)
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_graph() -> UndirectedGraph {
        let mut graph = UndirectedGraph::new();
        graph.insert("start".into(), "A".into());
        graph.insert("start".into(), "b".into());
        graph.insert("A".into(), "c".into());
        graph.insert("A".into(), "b".into());
        graph.insert("A".into(), "b".into());
        graph.insert("b".into(), "d".into());
        graph.insert("A".into(), "end".into());
        graph.insert("b".into(), "end".into());
        graph
    }

    #[test]
    fn test_count_cave_paths() {
        assert_eq!(
            count_cave_paths(
                &test_graph(),
                &"start".to_string(),
                &mut HashSet::new(),
                false
            ),
            10
        );
    }

    #[test]
    fn test_count_cave_paths_visiting_single_small_cave_twice() {
        assert_eq!(
            count_cave_paths(
                &test_graph(),
                &"start".to_string(),
                &mut HashSet::new(),
                true
            ),
            36
        );
    }

    #[test]
    fn test_read_undirected_graph() {
        let mut buf = "start-A
            start-b
            A-c
            A-b
            b-d
            A-end
            b-end"
            .as_bytes();
        assert_eq!(read_undirected_graph(&mut buf).unwrap(), test_graph());
    }
}
