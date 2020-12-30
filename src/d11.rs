use advent::helpers;
use anyhow::{Context, Result};
use derive_more::Display;
use itertools::Itertools;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
enum Tile {
    #[display(fmt = "L")]
    Empty,
    #[display(fmt = "#")]
    Occupied,
    #[display(fmt = ".")]
    Floor,
}

#[derive(Debug, Clone)]
struct Grid {
    rows: usize,
    cols: usize,
    g: Vec<Tile>,
}
type GridPos = (usize, usize);

#[derive(Debug, Clone, Copy)]
enum Direction {
    UpLeft,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
}

#[derive(Debug, PartialEq, Eq)]
enum TileNeighbourIterKind {
    Adjacent,
    InLineOfSight,
}
struct TileNeighboursIter<'a> {
    tile_pos: GridPos,
    grid: &'a Grid,
    next_direction: Option<Direction>,
    kind: TileNeighbourIterKind,
}

struct GridPosIter<'a> {
    grid: &'a Grid,
    next_index: Option<usize>,
}

impl FromStr for Tile {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            None => anyhow::bail!("No tile character"),
            Some('L') => Ok(Tile::Empty),
            Some('#') => Ok(Tile::Occupied),
            Some('.') => Ok(Tile::Floor),
            _ => anyhow::bail!("Invalid tile char"),
        }
    }
}

impl std::ops::Index<GridPos> for Grid {
    type Output = Tile;
    fn index(&self, index: GridPos) -> &Self::Output {
        let i = self.cols * index.0 + index.1;
        &self.g[i]
    }
}

impl std::ops::IndexMut<GridPos> for Grid {
    fn index_mut(&mut self, index: GridPos) -> &mut Self::Output {
        let i = self.cols * index.0 + index.1;
        &mut self.g[i]
    }
}

impl FromStr for Grid {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let g = s
            .lines()
            .flat_map(|l| l.chars().map(|c| c.to_string().parse::<Tile>()))
            .try_collect()?;
        let rows = s.lines().count();
        let cols = s
            .lines()
            .next()
            .map(|l| l.chars().count())
            .ok_or_else(|| anyhow::anyhow!("Row has no tiles"))?;
        Ok(Grid { rows, cols, g })
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.rows {
            for c in 0..self.cols {
                write!(f, "{}", self[(r, c)])?;
            }
            if r != self.rows - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl Direction {
    fn update_to_next_direction(current: &mut Option<Direction>) {
        *current = current.and_then(|d| match d {
            Direction::UpLeft => Some(Direction::Up),
            Direction::Up => Some(Direction::UpRight),
            Direction::UpRight => Some(Direction::Right),
            Direction::Right => Some(Direction::DownRight),
            Direction::DownRight => Some(Direction::Down),
            Direction::Down => Some(Direction::DownLeft),
            Direction::DownLeft => Some(Direction::Left),
            Direction::Left => None,
        })
    }

    fn get_delta(&self) -> (isize, isize) {
        match self {
            Direction::UpLeft => (-1, -1),
            Direction::Up => (-1, 0),
            Direction::UpRight => (-1, 1),
            Direction::Right => (0, 1),
            Direction::DownRight => (1, 1),
            Direction::Down => (1, 0),
            Direction::DownLeft => (1, -1),
            Direction::Left => (0, -1),
        }
    }
}

impl<'a> std::iter::Iterator for TileNeighboursIter<'a> {
    type Item = &'a Tile;
    fn next(&mut self) -> Option<Self::Item> {
        let visibility_fn = match self.kind {
            TileNeighbourIterKind::Adjacent => Grid::get_tile_in_direction,
            TileNeighbourIterKind::InLineOfSight => Grid::get_visible_tile_in_direction,
        };

        while let Some(current_direction) = self.next_direction {
            Direction::update_to_next_direction(&mut self.next_direction);
            let maybe_tile = visibility_fn(self.grid, self.tile_pos, &current_direction);
            if maybe_tile.is_some() {
                return maybe_tile;
            }
        }
        None
    }
}

impl<'a> std::iter::Iterator for GridPosIter<'_> {
    type Item = GridPos;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_index
            .and_then(|i| if i < self.grid.g.len() { Some(i) } else { None })
            .map(|i| {
                let current_index = i;
                self.next_index = Some(i + 1);
                let r = current_index / self.grid.cols;
                let c = current_index % self.grid.cols;
                (r, c)
            })
    }
}

impl Grid {
    fn adjacent_tiles_iter(&self, pos: GridPos) -> TileNeighboursIter {
        TileNeighboursIter {
            tile_pos: pos,
            grid: self,
            next_direction: Some(Direction::UpLeft),
            kind: TileNeighbourIterKind::Adjacent,
        }
    }

    fn visible_tiles_iter(&self, pos: GridPos) -> TileNeighboursIter {
        TileNeighboursIter {
            tile_pos: pos,
            grid: self,
            next_direction: Some(Direction::UpLeft),
            kind: TileNeighbourIterKind::InLineOfSight,
        }
    }

    fn pos_iter(&self) -> GridPosIter {
        GridPosIter {
            grid: self,
            next_index: Some(0),
        }
    }

    fn get_pos_in_direction(&self, pos: GridPos, direction: &Direction) -> GridPos {
        let (r_delta, c_delta) = direction.get_delta();
        (
            pos.0.wrapping_add(r_delta as usize),
            pos.1.wrapping_add(c_delta as usize),
        )
    }

    fn get_tile_in_direction(&self, pos: GridPos, direction: &Direction) -> Option<&Tile> {
        self.get(self.get_pos_in_direction(pos, direction))
    }

    fn get_visible_tile_in_direction(&self, pos: GridPos, direction: &Direction) -> Option<&Tile> {
        let mut new_pos = pos;
        loop {
            new_pos = self.get_pos_in_direction(new_pos, direction);
            let maybe_tile = self.get(new_pos);
            match maybe_tile {
                Some(Tile::Occupied) | Some(Tile::Empty) => return maybe_tile,
                Some(Tile::Floor) => (),
                None => return None,
            }
        }
    }

    #[allow(dead_code)]
    fn get_tile_in_direction_mut(
        &mut self,
        pos: GridPos,
        direction: &Direction,
    ) -> Option<&mut Tile> {
        self.get_mut(self.get_pos_in_direction(pos, direction))
    }

    fn get(&self, pos: GridPos) -> Option<&Tile> {
        let r = pos.0;
        let c = pos.1;
        if r >= self.rows || c >= self.cols {
            return None;
        };
        Some(&self[pos])
    }

    #[allow(dead_code)]
    fn get_mut(&mut self, pos: GridPos) -> Option<&mut Tile> {
        let r = pos.0;
        let c = pos.1;
        if r >= self.rows || c >= self.cols {
            return None;
        };
        Some(&mut self[pos])
    }
}

fn simulate_one_arrival_round(current_round: Grid, kind: &TileNeighbourIterKind) -> (Grid, bool) {
    let iter_kind_fn = match kind {
        TileNeighbourIterKind::Adjacent => Grid::adjacent_tiles_iter,
        TileNeighbourIterKind::InLineOfSight => Grid::visible_tiles_iter,
    };

    let mut new_round = current_round.clone();
    let changed = current_round.pos_iter().fold(false, |mut changed, pos| {
        let current_tile = current_round[pos];
        let tile_neighbour_count = iter_kind_fn(&current_round, pos)
            .filter(|tile| *tile == &Tile::Occupied)
            .count();
        new_round[pos] = {
            match current_tile {
                Tile::Empty if tile_neighbour_count == 0 => {
                    changed = true;
                    Tile::Occupied
                }
                Tile::Occupied
                    if tile_neighbour_count >= 5
                        && kind == &TileNeighbourIterKind::InLineOfSight =>
                {
                    changed = true;
                    Tile::Empty
                }
                Tile::Occupied
                    if tile_neighbour_count >= 4 && kind == &TileNeighbourIterKind::Adjacent =>
                {
                    changed = true;
                    Tile::Empty
                }
                _ => current_tile,
            }
        };
        changed
    });
    // println!("New round:\n{}\n", new_round);
    (new_round, changed)
}

fn simulate_arrival(s: &str, kind: &TileNeighbourIterKind) -> usize {
    let mut current_round = s.parse::<Grid>().expect("Invalid grid");
    let mut changed = true;
    let mut round_count = 0;
    while changed {
        let (new_round, new_changed) = simulate_one_arrival_round(current_round, kind);
        current_round = new_round;
        changed = new_changed;
        round_count += 1;
    }
    round_count -= 1;
    println!("\nStopped after {} rounds.", round_count);
    current_round
        .pos_iter()
        .map(|p| current_round[p])
        .filter(|t| t == &Tile::Occupied)
        .count()
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d11").context("Coudn't read file contents.")?;
    let occupied_count = simulate_arrival(&input, &TileNeighbourIterKind::Adjacent);
    println!("The number of occupied seats is: {}", occupied_count);
    Ok(())
}

fn solve_p2() -> Result<()> {
    let input = helpers::get_data_from_file_res("d11").context("Coudn't read file contents.")?;
    let occupied_count = simulate_arrival(&input, &TileNeighbourIterKind::InLineOfSight);
    println!("The number of occupied seats is: {}", occupied_count);
    Ok(())
}

fn main() -> Result<()> {
    solve_p1().ok();
    solve_p2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p1() {
        let input = "
L.LL.LL.LL
LLLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLLL
L.LLLLLL.L
L.LLLLL.LL";
        let occupied_seats = simulate_arrival(&input, &TileNeighbourIterKind::Adjacent);
        assert_eq!(occupied_seats, 37);
    }

    #[test]
    fn test_p2() {
        let input = "
L.LL.LL.LL
LLLLLLL.LL
L.L.L..L..
LLLL.LL.LL
L.LL.LL.LL
L.LLLLL.LL
..L.L.....
LLLLLLLLLL
L.LLLLLL.L
L.LLLLL.LL";
        let occupied_seats = simulate_arrival(&input, &TileNeighbourIterKind::InLineOfSight);
        assert_eq!(occupied_seats, 26);
    }
}
