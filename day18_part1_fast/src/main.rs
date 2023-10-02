use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;

use priority_queue::DoublePriorityQueue;

const ENTRANCE: char = '@';
const WALL: char = '#';
const SPACE: char = '.';

const INFINITY: usize = 10_000_000; // max u 4,294,967,295 ; max 2,147,483,647
const NOT_HERE: (usize, usize) = (0, 0);

fn parse(filename: &str) -> Vec<Vec<char>> {
    let data = fs::read_to_string(filename).expect(&format!("File not found: {filename}"));

    data.lines()
        .map(|s| s.chars().collect::<Vec<char>>())
        .collect::<Vec<Vec<char>>>()
}

fn get_neighbors(row: usize, col: usize, maze: &Vec<Vec<char>>) -> Vec<(usize, usize)> {
    let steps: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let mut neighbors: Vec<(usize, usize)> = vec![];

    for (step_row, step_col) in steps {
        let new_row: usize = (row as i32 + step_row) as usize;
        let new_col: usize = (col as i32 + step_col) as usize;
        if maze[new_row][new_col] != WALL {
            neighbors.push((new_row, new_col));
        }
    }
    neighbors
}

fn bfs(
    node: usize,
    position: (usize, usize),
    maze: &Vec<Vec<char>>,
    adjacency_matrix: &mut Vec<Vec<usize>>,
    dependencies: &mut Vec<Vec<usize>>,
    node_id: &HashMap<char, usize>,
) {
    // BFS init
    let mut queue: VecDeque<(usize, usize, usize, usize)> =
        VecDeque::from([(0, 0, position.0, position.1)]);
    let mut visited: HashSet<(usize, usize)> = HashSet::from([(position.0, position.1)]);

    // BFS
    while queue.len() > 0 {
        let (distance, path_dependencies, row, col) = queue.pop_front().unwrap();

        for (new_row, new_col) in get_neighbors(row, col, &maze) {
            if visited.contains(&(new_row, new_col)) {
                continue;
            }
            match maze[new_row][new_col] {
                'A'..='Z' => {
                    let mut new_dependencies = path_dependencies;
                    new_dependencies |= 1 << node_id[&maze[new_row][new_col].to_ascii_lowercase()];
                    visited.insert((new_row, new_col));
                    queue.push_back((distance + 1, new_dependencies, new_row, new_col));
                }
                'a'..='z' => {
                    let new_node: usize = node_id[&maze[new_row][new_col]];
                    // dependencies
                    adjacency_matrix[node][new_node] = distance + 1;
                    dependencies[node][new_node] = path_dependencies;
                    visited.insert((new_row, new_col));
                }
                ENTRANCE | SPACE => {
                    visited.insert((new_row, new_col));
                    queue.push_back((distance + 1, path_dependencies.clone(), new_row, new_col));
                }
                WALL => continue,
                _ => panic!("what?"),
            }
        }
    }
}


fn solve(maze: Vec<Vec<char>>) -> i32 {
    let mut char_keys: Vec<char> = vec![];

    for (_row, a_row) in maze.iter().enumerate() {
        for (_col, cell) in a_row.iter().enumerate() {
            match *cell {
                'a'..='z' => {
                    char_keys.push(*cell);
                }
                _ => continue,
            }
        }
    }
    char_keys.sort();
    let number_of_keys: usize = char_keys.len();
    char_keys.push(ENTRANCE);
    let node_id: HashMap<char, usize> = char_keys.iter().enumerate().map(|t| (*t.1, t.0)).collect();
    let total_nodes: usize = char_keys.len();

    let mut adjacency_matrix: Vec<Vec<usize>> = vec![vec![INFINITY; total_nodes]; total_nodes];
    let mut nodes_positions: Vec<(usize, usize)> = vec![(0, 0); total_nodes];
    let mut dependencies: Vec<Vec<usize>> = vec![vec![0; total_nodes]; total_nodes];

    // get nodes positions
    for (row, a_row) in maze.iter().enumerate() {
        for (col, cell) in a_row.iter().enumerate() {
            match *cell {
                ENTRANCE | 'a'..='z' => nodes_positions[node_id[cell]] = (row, col),
                _ => continue,
            }
        }
    }

    // build adjacency matrix
    for i in 0..number_of_keys {
        adjacency_matrix[i][i] = 0;
    }
    for (node, position) in nodes_positions.iter().enumerate() {
        if *position != NOT_HERE {
            bfs(
                node,
                *position,
                &maze,
                &mut adjacency_matrix,
                &mut dependencies,
                &node_id,
            );
        }
    }

    // Floyd-Warshall
    let mut dp = adjacency_matrix.clone();
    let mut dep = dependencies.clone();

    for k in 0..number_of_keys {
        for i in 0..number_of_keys {
            for j in 0..number_of_keys {
                if dp[i][k] + dp[k][j] < dp[i][j] {
                    dp[i][j] = dp[i][k] + dp[k][j];
                    // join dependencies
                    dep[i][j] = dep[i][k] | dep[k][j];
                }
            }
        }
    }

    // BFS init
    let initial_state: usize = 0;
    let mut queue: DoublePriorityQueue<(usize, usize), usize> =
        DoublePriorityQueue::new();
    queue.push((initial_state, node_id[&ENTRANCE]), 0);

    let mut visited: HashMap<(usize, usize), usize> = HashMap::new();
    visited.insert((node_id[&ENTRANCE], initial_state), 0);

    let goal: usize = (2 as usize).pow(number_of_keys as u32) - 1;
    let mut min_distance: usize = 0;
    // let mut keys: usize;
    // let mut node: usize;

    // BFS
    while queue.len() > 0 {

        let ((keys, node), distance) = queue.pop_min().unwrap();

        if keys == goal {
            min_distance = distance;
            break;
        }

        for new_node in 0..number_of_keys {
            // let new_distance: usize = adjacency_matrix[node][new_node];
            let new_distance: usize = dp[node][new_node];

            // skip same node and not connecting ones
            if new_node == node || new_distance >= INFINITY {
                continue;
            }

            // check dependencies 
            if (dep[node][new_node] & keys) != dep[node][new_node] {
                continue;
            }

            
            // have key already?
            let key_bit = 1 << new_node;
            if (keys & key_bit) != 0 {
                continue;
            }
            let new_keys = keys | key_bit;

            // dijkstra logic
            let state_key = (new_node, new_keys);

            if !visited.contains_key(&state_key) {
                visited.insert(state_key, distance + new_distance);
                queue.push((new_keys, new_node), distance + new_distance);
            } else {
                let old_distance = visited[&state_key];
                let current_distance = distance + new_distance;

                if current_distance < old_distance {
                    visited.insert(state_key, current_distance);
                    queue.push((new_keys, new_node), current_distance);
                }
            }
        }
    }
    min_distance as i32
}

fn solution(filename: &str) -> i32 {
    let maze: Vec<Vec<char>> = parse(filename);
    solve(maze)
}

fn main() {
    println!("{}", solution("./input.txt")); // 4900
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example1_should_be_8() {
        assert_eq!(solution("./example1.txt"), 8);
    }

    #[test]
    fn example2_should_be_86() {
        assert_eq!(solution("./example2.txt"), 86);
    }

    #[test]
    fn example3_should_be_132() {
        assert_eq!(solution("./example3.txt"), 132);
    }
    #[test]
    fn example4_should_be_136() {
        assert_eq!(solution("./example4.txt"), 136);
    }
    #[test]
    fn example5_should_be_81() {
        assert_eq!(solution("./example5.txt"), 81);
    }
}
