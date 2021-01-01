use itertools::Itertools;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Grid<T> {
    rows: usize,
    cols: usize,
    g: Vec<T>,
}
type GridPos = (usize, usize);

#[derive(Debug, Clone, Copy)]
pub enum Direction {
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
pub enum TileNeighbourIterKind {
    Adjacent,
    InLineOfSight,
}
pub struct TileNeighboursIter<'a, T> {
    tile_pos: GridPos,
    grid: &'a Grid<T>,
    next_direction: Option<Direction>,
    kind: TileNeighbourIterKind,
}

pub struct GridPosIter<'a, T> {
    grid: &'a Grid<T>,
    next_index: Option<usize>,
}

pub trait GridTileIsVisible {
    fn is_visible(&self) -> bool;
}

impl<T> std::ops::Index<GridPos> for Grid<T> {
    type Output = T;
    fn index(&self, index: GridPos) -> &Self::Output {
        let i = self.cols * index.0 + index.1;
        &self.g[i]
    }
}

impl<T> std::ops::IndexMut<GridPos> for Grid<T> {
    fn index_mut(&mut self, index: GridPos) -> &mut Self::Output {
        let i = self.cols * index.0 + index.1;
        &mut self.g[i]
    }
}

impl<T> FromStr for Grid<T>
where
    T: FromStr,
    anyhow::Error: From<T::Err>,
{
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let g = s
            .lines()
            .flat_map(|l| l.chars().map(|c| c.to_string().parse::<T>()))
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

impl<T> std::fmt::Display for Grid<T>
where
    T: std::fmt::Display,
{
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

impl<'a, T: GridTileIsVisible> std::iter::Iterator for TileNeighboursIter<'a, T> {
    type Item = &'a T;
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

impl<'a, T> std::iter::Iterator for GridPosIter<'_, T> {
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

impl<T> Grid<T> {
    pub fn adjacent_tiles_iter(&self, pos: GridPos) -> TileNeighboursIter<T> {
        TileNeighboursIter {
            tile_pos: pos,
            grid: self,
            next_direction: Some(Direction::UpLeft),
            kind: TileNeighbourIterKind::Adjacent,
        }
    }

    pub fn visible_tiles_iter(&self, pos: GridPos) -> TileNeighboursIter<T> {
        TileNeighboursIter {
            tile_pos: pos,
            grid: self,
            next_direction: Some(Direction::UpLeft),
            kind: TileNeighbourIterKind::InLineOfSight,
        }
    }

    pub fn pos_iter(&self) -> GridPosIter<T> {
        GridPosIter {
            grid: self,
            next_index: Some(0),
        }
    }

    pub fn get_pos_in_direction(&self, pos: GridPos, direction: &Direction) -> GridPos {
        let (r_delta, c_delta) = direction.get_delta();
        (
            pos.0.wrapping_add(r_delta as usize),
            pos.1.wrapping_add(c_delta as usize),
        )
    }

    pub fn get_tile_in_direction(&self, pos: GridPos, direction: &Direction) -> Option<&T> {
        self.get(self.get_pos_in_direction(pos, direction))
    }

    pub fn get_visible_tile_in_direction(&self, pos: GridPos, direction: &Direction) -> Option<&T>
    where
        T: GridTileIsVisible,
    {
        let mut new_pos = pos;
        loop {
            new_pos = self.get_pos_in_direction(new_pos, direction);
            let maybe_tile = self.get(new_pos);
            match maybe_tile {
                Some(tile) => {
                    if tile.is_visible() {
                        return maybe_tile;
                    } else {
                    }
                }
                None => return None,
            }
        }
    }

    pub fn get_tile_in_direction_mut(
        &mut self,
        pos: GridPos,
        direction: &Direction,
    ) -> Option<&mut T> {
        self.get_mut(self.get_pos_in_direction(pos, direction))
    }

    pub fn get(&self, pos: GridPos) -> Option<&T> {
        let r = pos.0;
        let c = pos.1;
        if r >= self.rows || c >= self.cols {
            return None;
        };
        Some(&self[pos])
    }

    pub fn get_mut(&mut self, pos: GridPos) -> Option<&mut T> {
        let r = pos.0;
        let c = pos.1;
        if r >= self.rows || c >= self.cols {
            return None;
        };
        Some(&mut self[pos])
    }
}
