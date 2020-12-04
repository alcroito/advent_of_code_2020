use advent::helpers;

#[derive(Debug)]
enum Cell {
    Empty,
    Tree,
}

impl From<&u8> for Cell {
    fn from(c: &u8) -> Self {
        match *c as char {
            '#' => Cell::Tree,
            _ => Cell::Empty,
        }
    }
}

type Row = Vec<Cell>;
type Matrix = Vec<Row>;

#[derive(Debug)]
struct Grid {
    grid: Matrix,
}

impl Grid {
    fn get(&self, r: usize, c: usize) -> Option<&Cell> {
        let wrapped_col = c % self.grid[0].len();
        self.grid.get(r).and_then(|row| row.get(wrapped_col))
    }

    fn slide(&self, (r_delta, c_delta): (usize, usize)) -> i64 {
        let mut current_r = 0;
        let mut current_c = 0;
        let mut tree_count = 0;

        loop {
            current_r += r_delta;
            current_c += c_delta;
            let cell = self.get(current_r, current_c);

            match cell {
                Some(Cell::Tree) => tree_count += 1,
                Some(Cell::Empty) => (),
                None => break,
            }
        }
        tree_count
    }
}

impl From<&str> for Grid {
    fn from(s: &str) -> Self {
        let s = s.trim();
        let mut grid: Matrix = vec![];
        for line in s.lines() {
            let mut row: Row = vec![];
            for c in line.as_bytes() {
                row.push(c.into());
            }
            grid.push(row);
        }
        Grid { grid: grid }
    }
}

fn count_tree_while_sliding(grid: &Grid, slope: (usize, usize)) -> i64 {
    grid.slide(slope)
}

fn solve_p1() {
    let input = helpers::get_data_from_file("d3").expect("Coudn't read file contents.");
    let grid: Grid = (*input).into();
    let tree_count = count_tree_while_sliding(&grid, (1, 3));
    println!("Evaded tree count: {}", tree_count);
}

fn solve_p2() {
    let input = helpers::get_data_from_file("d3").expect("Coudn't read file contents.");
    let grid: Grid = (*input).into();
    let slopes = vec![(1, 1), (1, 3), (1, 5), (1, 7), (2, 1)];
    let product: i64 = slopes
        .iter()
        .map(|slope| count_tree_while_sliding(&grid, *slope))
        .product();
    println!("Multpied evaded tree count: {}", product);
}

#[test]
fn test_p1() {
    let input = "
..##.......
#...#...#..
.#....#..#.
..#.#...#.#
.#...##..#.
..#.##.....
.#.#.#....#
.#........#
#.##...#...
#...##....#
.#..#...#.#";
    let grid = input.into();
    let slope = (1, 3);
    assert_eq!(count_tree_while_sliding(&grid, slope), 7);
}

#[test]
fn test_p2() {
    let input = "
..##.......
#...#...#..
.#....#..#.
..#.#...#.#
.#...##..#.
..#.##.....
.#.#.#....#
.#........#
#.##...#...
#...##....#
.#..#...#.#";
    let grid = input.into();
    let slopes = vec![(1, 1), (1, 3), (1, 5), (1, 7), (2, 1)];
    let expected_results = vec![2, 7, 3, 4, 2];

    let product = slopes
        .iter()
        .zip(expected_results.iter())
        .map(|(&slope, &expected_count)| {
            let count = count_tree_while_sliding(&grid, slope);
            assert_eq!(count, expected_count);
            count
        })
        .product::<i64>();
    assert_eq!(product, 336);
}

fn main() {
    solve_p1();
    solve_p2();
}
