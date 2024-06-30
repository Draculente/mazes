use std::{fs::File, io::Write, num::ParseIntError, path::PathBuf};

use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use mazes::{a_star, generate_maze, Block, Map};
use promptly::{prompt, prompt_opt};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Solve(SolveArgs),
    Gen(GenArgs),
}

#[derive(Args)]
struct SolveArgs {
    /// The path of the map on which the agent shall move
    #[arg(long, short)]
    path: Option<PathBuf>,
    /// The x coordinate of the initial position of the agent
    #[arg(long)]
    start_x: Option<usize>,
    /// The y coordinate of the initial position of the agent (origin is in the top left)
    #[arg(long)]
    start_y: Option<usize>,
    /// The x coordinate of the desired destination of the agent
    #[arg(long)]
    dest_x: Option<usize>,
    /// The y coordinate of the desired destination of the agent (origin is in the top left)
    #[arg(long)]
    dest_y: Option<usize>,
    /// The path where to store the solution as txt
    #[arg(long)]
    txt: Option<PathBuf>,
    /// The path where to store the solution as png
    #[arg(long)]
    png: Option<PathBuf>,
}

fn between_0_1(s: &str) -> Result<f64, String> {
    let f: f64 = s.parse().map_err(|_| format!("'{s}' is not a float"))?;
    if f >= 1.0 {
        return Err(format!("{s} is greater than or equal to 1"));
    }
    if f < 0.0 {
        return Err(format!("{s} is smaller than 0"));
    }
    Ok(f)
}

#[derive(Args)]
struct GenArgs {
    /// The width of the generated maze in blocks
    #[arg(long)]
    width: Option<usize>,
    /// The height of the generated maze in blocks
    #[arg(long)]
    height: Option<usize>,
    /// The probability that a loop occurs as decimal number between 0 and 1
    #[arg(long, short, value_parser = between_0_1)]
    loop_prob: Option<f64>,
    /// The path where to save the generated map as png
    #[arg(long, short)]
    path: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("{e}");
    }
}

fn run(cli: &Cli) -> anyhow::Result<()> {
    // if args.contains(&"solve".to_owned()) {
    //     find_path()
    // } else if args.contains(&"gen".to_owned()) {
    //     get_maze()
    // } else {
    //     Err(anyhow!(
    //         "Please specify a command as argument. The available commands are: 'solve' and 'gen'."
    //     ))
    // }
    match &cli.command {
        Commands::Solve(solve_args) => solve(&solve_args),
        Commands::Gen(gen_args) => gen(&gen_args),
    }
}

fn gen(args: &GenArgs) -> anyhow::Result<()> {
    let width: usize = args
        .width
        .ok_or("No width arg specified")
        .or_else(|_| prompt("Specify the width of the maze"))?;

    let height: usize = args
        .height
        .ok_or("No width arg specified")
        .or_else(|_| prompt("Specify the height of the maze"))?;

    let loop_prob: Option<f64> = if let Some(l) = args.loop_prob {
        Some(l)
    } else {
        prompt_opt("Specify the probability for loops as f64 between 0 and 1 (0)")?
    };

    let loop_prob = loop_prob.unwrap_or(0.0);

    if loop_prob >= 1.0 || loop_prob < 0.0 {
        return Err(anyhow!("Please specify a loop probability between 0 and 1"));
    }

    let maze_map = generate_maze(width / 2, height / 2, Some(loop_prob))?;
    let map = Map::from(maze_map);

    println!("{map}");

    let path: Option<PathBuf> = if let Some(p) = &args.path {
        Some(p.clone())
    } else {
        prompt_opt("Enter the path where to save the map as png")?
    };

    map.to_image()
        .ok_or(anyhow!("Failed to create image"))?
        .save(path.ok_or(anyhow!("No path specified. Discarding the image"))?)?;

    Ok(())
}

fn solve(args: &SolveArgs) -> anyhow::Result<()> {
    let path: PathBuf = if let Some(p) = &args.path {
        p.clone()
    } else {
        prompt("Enter the path to the map as png")?
    };

    let img = image::open(path)?;
    let map: Map = Map::from(img);

    println!("{map}");

    let start_line: String = args
        .start_y
        .and_then(|y| args.start_x.map(|x| format!("{x} {y}")))
        .ok_or("No start x y arg specified")
        .or_else(|_| prompt("Enter the start as x y"))?;

    let start_block = parse_block(&start_line, &map)?;

    let destination_line: String = args
        .dest_y
        .and_then(|y| args.dest_x.map(|x| format!("{x} {y}")))
        .ok_or("No dest x y arg specified")
        .or_else(|_| prompt("Enter the destination as x y"))?;

    let destination_block = parse_block(&destination_line, &map)?;

    if let Ok(solution) = a_star(&map, start_block, destination_block) {
        let solution_file = args
            .txt
            .as_ref()
            .cloned()
            .unwrap_or(PathBuf::from("solution.txt".to_string()));

        let mut file = File::create(solution_file)?;

        let solution_seq = solution.as_sequence_of_maps(&map);
        let solution_str = solution.to_string();

        for state in solution_seq {
            file.write_all(format!("{}\n", state).as_bytes())?;
            println!("{}", state);
        }

        file.write_all(format!("{}\n", solution_str).as_bytes())?;
        println!("{solution_str}");

        let should_be_saved_as_png: bool =
            args.png.is_some() || prompt("Do you want to save this solution as png?")?;

        if should_be_saved_as_png {
            let path: PathBuf = args
                .png
                .as_ref()
                .ok_or("Not png path arg specified")
                .cloned()
                .or_else(|_| prompt("Enter the path where the map should be saved"))?;
            //prompt("Enter the path where the map should be saved")?;
            solution
                .to_solution_map()
                .to_image()
                .ok_or(anyhow!("Failed to create image"))?
                .save(path)?;
        }
    } else {
        eprintln!("No path found 😢");
    };

    Ok(())
}

fn parse_block(line: &str, map: &Map) -> anyhow::Result<Block> {
    let coords: Result<Vec<usize>, ParseIntError> = line
        .split(" ")
        .map(|string_num| string_num.parse::<usize>())
        .collect();

    let coords = coords.map_err(|_| anyhow!("Please specify valid usize numbers"))?;

    if coords.len() != 2 {
        return Err(anyhow!("Please specify two coordinates"));
    }

    map.get_block(
        *coords.get(0).expect("Can't happen"),
        *coords.get(1).expect("Can't happen"),
    )
    .ok_or(anyhow!("Please specify coordinates within the map"))
}
