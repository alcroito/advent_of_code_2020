use advent::helpers;
use anyhow::{Context, Result};
use derive_more::Display;
use helpers::grid::{Grid, GridTileIsVisible, TileNeighbourIterKind};
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

impl GridTileIsVisible for Tile {
    fn is_visible(&self) -> bool {
        matches!(self, Tile::Occupied | Tile::Empty)
    }
}

type MyGrid = Grid<Tile>;

fn simulate_one_arrival_round(
    current_round: MyGrid,
    kind: &TileNeighbourIterKind,
) -> (MyGrid, bool) {
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
    let mut current_round = s.parse::<MyGrid>().expect("Invalid grid");
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
    use helpers::grid::TileNeighbourIterKind;

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
