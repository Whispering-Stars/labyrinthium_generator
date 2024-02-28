use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::{BufRead, BufReader, Error, Write},
};

use knossos::maze::*;
use serde::Serialize;

// Define a struct to represent the maze
struct Maze {
    rows: usize,
    cols: usize,
    data: Vec<Vec<char>>,
}

// Define structs for JSON serialization
#[derive(Serialize)]
struct Cell {
    x: usize,
    y: usize,
    #[serde(rename = "type")]
    cell_type: u8,
}

#[derive(Serialize)]
struct MazeJson {
    width: usize,
    height: usize,
    maze: Vec<Cell>,
    solution: Vec<Position>,
}

#[derive(Serialize)]
struct Position {
    x: usize,
    y: usize,
}

fn main() {
    let maze = OrthogonalMazeBuilder::new()
        .height(10)
        .width(10)
        .algorithm(Box::new(GrowingTree::new(Method::Random)))
        .build();

    match maze.save("output/maze.txt", GameMap::new().span(1).with_start_goal()) {
        Ok(_) => {
            if let Ok(maze) = read_maze_from_file("output/maze.txt") {
                println!("Original maze:");
                for row in &maze.data {
                    for &cell in row {
                        print!("{}", cell);
                    }
                    println!();
                }

                if let Some(path) = solve_maze(&maze) {
                    if let Err(err) =
                        create_json_file(maze.cols, maze.rows, &maze, &path, "output/maze.json")
                    {
                        println!("Error creating JSON file: {:?}", err);
                    } else {
                        println!("JSON file created successfully.");
                    }
                } else {
                    println!("No path found.");
                }
            } else {
                println!("Error reading maze file.");
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

// Function to read the content of the file and create a Maze object
fn read_maze_from_file(filename: &str) -> Result<Maze, Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut rows = 0;
    let mut cols = 0;
    let mut data = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let chars: Vec<char> = line.chars().collect();
        cols = chars.len();
        data.push(chars);
        rows += 1;
    }

    Ok(Maze { rows, cols, data })
}

// Function to solve the maze
fn solve_maze(maze: &Maze) -> Option<Vec<(usize, usize)>> {
    let mut visited = HashSet::new();
    let mut stack = VecDeque::new();
    let mut parents = HashMap::new();

    let start = find_start(&maze);
    stack.push_back(start);
    visited.insert(start);

    while let Some((row, col)) = stack.pop_back() {
        if maze.data[row][col] == 'G' {
            return Some(construct_path((row, col), &parents));
        }

        for (dr, dc) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let (new_row, new_col) = (row as i32 + dr, col as i32 + dc);
            if new_row >= 0
                && new_row < maze.rows as i32
                && new_col >= 0
                && new_col < maze.cols as i32
                && maze.data[new_row as usize][new_col as usize] != '#'
                && !visited.contains(&(new_row as usize, new_col as usize))
            {
                stack.push_back((new_row as usize, new_col as usize));
                visited.insert((new_row as usize, new_col as usize));
                parents.insert((new_row as usize, new_col as usize), (row, col));
            }
        }
    }

    None
}

// Helper function to find the starting point in the maze
fn find_start(maze: &Maze) -> (usize, usize) {
    for (i, row) in maze.data.iter().enumerate() {
        for (j, &c) in row.iter().enumerate() {
            if c == 'S' {
                return (i, j);
            }
        }
    }
    panic!("No starting point 'S' found in the maze.");
}

// Helper function to construct the path from the goal to the start
fn construct_path(
    goal: (usize, usize),
    parents: &HashMap<(usize, usize), (usize, usize)>,
) -> Vec<(usize, usize)> {
    let mut path = Vec::new();
    let mut current = goal;
    while let Some(&parent) = parents.get(&current) {
        path.push(current);
        current = parent;
    }
    path.push(current);
    path.reverse();
    path
}

// Function to create a JSON file from maze data and solution
fn create_json_file(
    width: usize,
    height: usize,
    maze: &Maze,
    solution: &Vec<(usize, usize)>,
    filename: &str,
) -> Result<(), std::io::Error> {
    let maze_cells = maze
        .data
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter().enumerate().map(move |(x, &cell)| {
                let cell_type = match cell {
                    'S' => 0,
                    'G' => 1,
                    '.' => 2,
                    '#' => 3,
                    _ => panic!("Unknown cell type"),
                };
                Cell { x, y, cell_type }
            })
        })
        .collect::<Vec<_>>();

    let solution_cells = solution
        .iter()
        .map(|&(y, x)| Position { x, y })
        .collect::<Vec<_>>();

    let maze_json = MazeJson {
        width,
        height,
        maze: maze_cells,
        solution: solution_cells,
    };

    let json_string = serde_json::to_string_pretty(&maze_json)?;

    let mut file = File::create(filename)?;
    file.write_all(json_string.as_bytes())?;

    Ok(())
}
