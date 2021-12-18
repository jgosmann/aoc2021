use std::borrow::Borrow;
use std::error::Error;
use std::{
    cell::RefCell,
    fmt::Display,
    io::{self, BufRead},
    num::ParseIntError,
    ops::Deref,
    rc::Rc,
};

type Number = u8;

#[derive(Clone, Debug)]
struct Leaf {
    value: RefCell<Number>,
    left: RefCell<Option<Rc<Leaf>>>,
    right: RefCell<Option<Rc<Leaf>>>,
}

impl Leaf {
    fn new(value: Number) -> Self {
        Self {
            value: RefCell::new(value),
            left: RefCell::new(None),
            right: RefCell::new(None),
        }
    }
}

impl Display for Leaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.value.borrow()))
    }
}

impl PartialEq for Leaf {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Leaf {}

#[derive(Debug, PartialEq, Eq)]
enum Node {
    Pair(Box<RefCell<Node>>, Box<RefCell<Node>>),
    RegularNumber(Rc<Leaf>),
}

impl Clone for Node {
    fn clone(&self) -> Self {
        match self {
            Self::Pair(arg0, arg1) => Self::Pair(arg0.clone(), arg1.clone()),
            Self::RegularNumber(arg0) => Self::RegularNumber(Rc::new(arg0.deref().clone())),
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RegularNumber(leaf) => leaf.fmt(f),
            Self::Pair(lhs, rhs) => f.write_fmt(format_args!(
                "[{},{}]",
                lhs.deref().borrow(),
                rhs.deref().borrow()
            )),
        }
    }
}

impl Node {
    fn init_leaf_links(&mut self, left: Option<Rc<Leaf>>) -> Rc<Leaf> {
        match self {
            Self::RegularNumber(leaf) => {
                leaf.left.replace(left.clone());
                if let Some(left) = left {
                    left.right.replace(Some(Rc::clone(leaf)));
                }
                return Rc::clone(leaf);
            }
            Self::Pair(lhs, rhs) => {
                let left = lhs.get_mut().init_leaf_links(left);
                return rhs.get_mut().init_leaf_links(Some(left));
            }
        }
    }

    fn explode(node: &mut RefCell<Self>, nesting_level: usize) -> bool {
        if let Self::Pair(lhs, rhs) = node.get_mut() {
            if nesting_level >= 4 {
                if let (Self::RegularNumber(left_leaf), Self::RegularNumber(right_leaf)) =
                    (lhs.get_mut(), rhs.get_mut())
                {
                    let new_leaf = Rc::new(Leaf::new(0));
                    if let Some(left) = left_leaf.left.borrow_mut().as_mut() {
                        new_leaf.left.replace(Some(Rc::clone(&left)));
                        *left.value.borrow_mut() += *left_leaf.value.borrow();
                        left.right.replace(Some(Rc::clone(&new_leaf)));
                    }
                    if let Some(right) = right_leaf.right.borrow_mut().as_mut() {
                        new_leaf.right.replace(Some(Rc::clone(&right)));
                        *right.value.borrow_mut() += *right_leaf.value.borrow();
                        right.left.replace(Some(Rc::clone(&new_leaf)));
                    }
                    node.replace(Self::RegularNumber(new_leaf));
                    return true;
                }
            }

            return Self::explode(lhs, nesting_level + 1) || Self::explode(rhs, nesting_level + 1);
        } else {
            return false;
        }
    }

    fn split(node: &mut RefCell<Self>) -> bool {
        match node.get_mut() {
            Self::Pair(lhs, rhs) => Self::split(lhs) || Self::split(rhs),
            Self::RegularNumber(leaf) => {
                if *leaf.value.borrow() > 9 {
                    let left_value = *leaf.deref().value.borrow() / 2;
                    let right_value = *leaf.deref().value.borrow() - *left_value.borrow();
                    let lhs = Rc::new(Leaf::new(left_value));
                    let rhs = Rc::new(Leaf::new(right_value));
                    lhs.right.replace(Some(Rc::clone(&rhs)));
                    rhs.left.replace(Some(Rc::clone(&lhs)));
                    if let Some(left) = leaf.left.borrow().as_ref() {
                        left.right.replace(Some(Rc::clone(&lhs)));
                        lhs.left.replace(Some(Rc::clone(&left)));
                    }
                    if let Some(right) = leaf.right.borrow().as_ref() {
                        right.left.replace(Some(Rc::clone(&rhs)));
                        rhs.right.replace(Some(Rc::clone(&right)));
                    }
                    node.replace(Self::Pair(
                        Box::new(RefCell::new(Self::RegularNumber(lhs))),
                        Box::new(RefCell::new(Self::RegularNumber(rhs))),
                    ));
                    true
                } else {
                    false
                }
            }
        }
    }

    fn add(lhs: RefCell<Self>, rhs: RefCell<Self>) -> RefCell<Self> {
        let mut sum = RefCell::new(Self::Pair(Box::new(lhs), Box::new(rhs)));
        sum.borrow_mut().init_leaf_links(None);
        let mut needs_reduction = true;
        while needs_reduction {
            needs_reduction = Self::explode(&mut sum, 0) || Self::split(&mut sum);
        }
        sum
    }

    fn magnitude(&self) -> u64 {
        match self {
            Self::RegularNumber(leaf) => *leaf.deref().value.borrow() as u64,
            Self::Pair(lhs, rhs) => {
                3 * lhs.deref().borrow().magnitude() + 2 * rhs.deref().borrow().magnitude()
            }
        }
    }
}

#[derive(Debug)]
struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("parse error")
    }
}

impl Error for ParseError {}

fn parse_regular_num(input: &str) -> Result<(RefCell<Node>, &str), ParseIntError> {
    let value = u8::from_str_radix(&input[0..1], 10)?;
    Ok((
        RefCell::new(Node::RegularNumber(Rc::new(Leaf::new(value)))),
        &input[1..],
    ))
}

fn parse_pair(input: &str) -> Result<(RefCell<Node>, &str), Box<dyn Error>> {
    if input.as_bytes()[0] != b'[' {
        return Err(Box::new(ParseError));
    }
    let (lhs, input) = parse(&input[1..])?;
    if input.as_bytes()[0] != b',' {
        return Err(Box::new(ParseError));
    }
    let (rhs, input) = parse(&input[1..])?;
    if input.as_bytes()[0] != b']' {
        return Err(Box::new(ParseError));
    }
    Ok((
        RefCell::new(Node::Pair(Box::new(lhs), Box::new(rhs))),
        &input[1..],
    ))
}

fn parse(input: &str) -> Result<(RefCell<Node>, &str), Box<dyn Error>> {
    if input.as_bytes()[0] == b'[' {
        return parse_pair(input);
    } else {
        return Ok(parse_regular_num(input)?);
    }
}

fn read_homework<R: BufRead>(reader: &mut R) -> Result<Vec<RefCell<Node>>, Box<dyn Error>> {
    reader.lines().map(|line| Ok(parse(&line?)?.0)).collect()
}

fn do_homework_part1(numbers: Vec<RefCell<Node>>) -> u64 {
    let sum = numbers.into_iter().reduce(|x, y| Node::add(x, y)).unwrap();
    let sum = sum.borrow();
    sum.magnitude()
}

fn do_homework_part2(numbers: &Vec<RefCell<Node>>) -> u64 {
    let mut max_magnitude = 0;
    for (i, lhs) in numbers.iter().enumerate() {
        for (j, rhs) in numbers.iter().enumerate() {
            if i == j {
                continue;
            }
            let sum = Node::add(lhs.clone(), rhs.clone());
            let sum = sum.borrow();
            max_magnitude = u64::max(max_magnitude, sum.magnitude());
        }
    }
    max_magnitude
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let homework = read_homework(&mut stdin.lock())?;
    println!("Part 1: {}", do_homework_part1(homework.clone()));
    println!("Part 2: {}", do_homework_part2(&homework));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case("[[1,2],[[3,4],5]]", 143)]
    #[case("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]", 1384)]
    #[case("[[[[1,1],[2,2]],[3,3]],[4,4]]", 445)]
    #[case("[[[[3,0],[5,3]],[4,4]],[5,5]]", 791)]
    #[case("[[[[5,0],[7,4]],[5,5]],[6,6]]", 1137)]
    #[case("[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]", 3488)]
    fn test_magnitude(#[case] input: &str, #[case] expected: u64) {
        assert_eq!(parse(input).unwrap().0.borrow().magnitude(), expected);
    }

    #[test]
    fn test_addition() {
        let lhs = parse("[[[[4,3],4],4],[7,[[8,4],9]]]").unwrap().0;
        let rhs = parse("[1,1]").unwrap().0;
        let expected = parse("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]").unwrap().0;
        assert_eq!(Node::add(lhs, rhs), expected);
    }

    static TEST_INPUT: &[u8] = include_bytes!("../test.input");

    #[test]
    fn test_do_homework_part1() {
        let mut input = TEST_INPUT;
        assert_eq!(do_homework_part1(read_homework(&mut input).unwrap()), 4140);
    }

    #[test]
    fn test_do_homework_part2() {
        let mut input = TEST_INPUT;
        assert_eq!(do_homework_part2(&read_homework(&mut input).unwrap()), 3993);
    }
}
