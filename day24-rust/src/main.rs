use std::{
    error::Error,
    io::{self, BufRead},
};

use day24_rust::{
    ast::DeduplicatedAst,
    instruction_set::Op,
    model_number_search::{ModelNumberSearch, SearchMode},
    symbolic_alu::SymbolicAlu,
};

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();

    let mut alu = SymbolicAlu::new();
    for line in stdin.lock().lines() {
        let line = line?;
        let instruction = Op::try_from(line.as_str())?;
        alu.execute(&instruction);
    }

    let z = alu.extract_z();
    let compact_ast = DeduplicatedAst::from(&z);
    println!("AST: \n{}\n", compact_ast);

    let mut search = ModelNumberSearch::new(&compact_ast);
    println!(
        "Largest accepted model number: {}",
        search
            .find_model_number(SearchMode::Largest)
            .unwrap_or_else(|| "None found.".into())
    );
    println!(
        "Smallest accepted model number: {}",
        search
            .find_model_number(SearchMode::Smallest)
            .unwrap_or_else(|| "None found.".into())
    );
    Ok(())
}
