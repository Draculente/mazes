use image_a_star::{a_star, Map};

fn main() {
    let img = image::open("./images/lageplan.png").expect("Error opening the image");
    let map: Map = Map::from(img);

    if let Ok(solution) = a_star(&map, 6, 12, 9, 14) {
        for state in solution {
            println!("{}", state.display_on_map(&map));
        }
    } else {
        eprintln!("No path found 😢");
    };
}
