use clap::Parser;
use num_format::{Locale, ToFormattedString};
use polycube::{puzzle::Puzzle, solver::Solver};
use std::{io, path::PathBuf};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Puzzle file
    puzzle: PathBuf,

    /// Returns solution to sudoku
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let puzzle = Puzzle::from_csv(args.puzzle)?;

    let mut solver = Solver::build(args.verbose);
    solver.begin(&puzzle);

    // if args.verbose {
    let duration = solver.start_time.unwrap().elapsed();
    let rate = duration / solver.solutions.len() as u32;

    println!(
        "Solutions: {} Explored: {} Time: {:.2?} [rate {:.2?} per solution]",
        solver.solutions.len().to_formatted_string(&Locale::en),
        solver.explored.to_formatted_string(&Locale::en),
        duration,
        rate
    );
    // }

    Ok(())
}
