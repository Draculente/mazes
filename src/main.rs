use std::{fs::File, io::Write, num::ParseIntError, path::PathBuf};

use anyhow::anyhow;
use image_a_star::{a_star, Block, Map};
use promptly::{prompt, prompt_opt};

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
    }
}

fn run() -> anyhow::Result<()> {
    let path: Option<PathBuf> = prompt_opt("Enter the path to the map as png")?;

    let img = image::open(path.ok_or(anyhow!("Please specify a path"))?)
        .expect("Error opening the image");
    let map: Map = Map::from(img);

    println!("{map}");

    let start_line: String = prompt("Enter the start as x y")?;

    let start_block = parse_block(&start_line, &map)?;

    let destination_line: String = prompt("Enter the destination as x y")?;

    let destination_block = parse_block(&destination_line, &map)?;

    if let Ok(solution) = a_star(&map, start_block, destination_block) {
        let solution_file = "solution.txt";

        let mut file = File::create(solution_file)?;

        for state in solution {
            file.write_all(format!("{}\n", state.display_on_map(&map)).as_bytes())?;
            println!("{}", state.display_on_map(&map));
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
