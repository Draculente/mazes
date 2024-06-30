mod map;
mod maze_generation;

use std::cmp::Reverse;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Ok;
use itertools::Itertools;
pub use map::Block;
pub use map::Map;
pub use maze_generation::generate_maze;
use priority_queue::PriorityQueue;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct State {
    location: Block,
}

impl State {
    fn new(location: Block) -> Self {
        Self { location }
    }

    pub fn display_on_map(&self, map: &Map) -> String {
        map.to_string_with_locations(&vec![self.location], false)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Node {
    state: State,
    parent: Option<Arc<Node>>,
    cost: u32,
}

impl Node {
    fn new(state: State, parent: Option<Arc<Node>>, cost: u32) -> Self {
        Self {
            state,
            parent,
            cost,
        }
    }

    fn get_steps(&self) -> Vec<State> {
        let mut solution: Vec<State> = Vec::new();
        let mut node = self;
        while let Some(parent) = &node.parent {
            solution.push(node.state);
            node = parent;
        }
        solution.push(node.state);
        solution.reverse();
        solution
    }

    fn euclidean_distance(&self, destination: Block) -> u32 {
        (((self.state.location.x as i32 - destination.x as i32).pow(2)
            + (self.state.location.y as i32 - destination.y as i32).pow(2)) as f64)
            .sqrt() as u32
    }

    fn f(&self, destination: Block) -> u32 {
        self.euclidean_distance(destination) + self.cost
    }
}

pub struct Solution {
    states: Vec<State>,
    map: Map,
    cost: u32,
}

impl Solution {
    fn new(node: &Node, mut map: Map) -> Self {
        let states = node.get_steps();
        let cost = node.cost;
        map.enter_solution(&states.iter().map(|state| state.location).collect_vec());
        Self { states, map, cost }
    }

    pub fn as_sequence_of_maps(&self, map: &Map) -> Vec<String> {
        self.states
            .iter()
            .map(|state| state.display_on_map(map))
            .collect_vec()
    }

    pub fn to_solution_map(self) -> Map {
        self.map
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.map.to_string_with_locations(&vec![], false))?;
        f.write_fmt(format_args!(
            "This solution cost {} and involves {} steps\n",
            self.cost,
            self.states.len()
        ))
    }
}

pub fn a_star(map: &Map, start_block: Block, destination_block: Block) -> anyhow::Result<Solution> {
    let first_state = State::new(start_block);
    let first_node = Arc::new(Node::new(first_state, None, 0));

    let mut frontier: PriorityQueue<Arc<Node>, Reverse<u32>> = PriorityQueue::new();
    let mut reached: HashMap<State, Arc<Node>> = HashMap::new();

    let f = first_node.f(destination_block);

    frontier.push(first_node, Reverse(f));

    while !frontier.is_empty() {
        let (node, _) = frontier.pop().ok_or(anyhow!("Frontier is empty"))?;
        if node.state.location == destination_block {
            return Ok(Solution::new(&node, (*map).clone()));
        }
        for action in map.get_reachable(node.state.location.x, node.state.location.y) {
            let new_state = State::new(action);
            let child = Arc::new(Node::new(
                new_state,
                Some(node.clone()),
                node.cost + new_state.location.speed() as u32,
            ));
            if !reached.contains_key(&new_state) {
                reached.insert(new_state, child.clone());
                frontier.push(child.clone(), Reverse(child.f(destination_block)));
            } else if child.cost < reached[&child.state].cost {
                // Remove old (worse) node
                frontier.remove(&reached[&child.state]);
                reached.insert(child.state, child.clone());
                frontier.push(child.clone(), Reverse(child.f(destination_block)));
            }
        }
    }

    Err(anyhow!("There is no path"))
}
