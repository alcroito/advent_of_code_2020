use advent::helpers;
use anyhow::{Context, Result};
use derive_more::Display;
use itertools::Itertools;
use once_cell::sync::Lazy;
use std::ops::RangeInclusive;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
enum Cube {
    #[display(fmt = "#")]
    Active,
    #[display(fmt = ".")]
    Inactive,
}

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Hash)]
#[display(fmt = "({}, {}, {})", x, y, z)]
struct Point4D {
    x: isize,
    y: isize,
    z: isize,
    w: isize,
}

#[derive(Debug, Clone)]
enum PointIterKind {
    D3,
    D4,
}

struct Point4DNeighboursIter {
    pos: Point4D,
    max_id: u8,
    iter_kind: PointIterKind,
    next_direction: Option<u8>,
}

struct ActivePoint4DIter<'a> {
    iter: std::collections::hash_set::Iter<'a, Point4D>,
}

#[derive(Debug, Clone)]
struct Bounds {
    x_range: RangeInclusive<isize>,
    y_range: RangeInclusive<isize>,
    z_range: RangeInclusive<isize>,
    w_range: RangeInclusive<isize>,
}

type ActivePointSet = std::collections::HashSet<Point4D>;
type ActiveNeighborCounter = std::collections::HashMap<Point4D, u8>;
type Point4DTuple = (isize, isize, isize, isize);

#[derive(Debug, Clone)]
struct Grid4D {
    grid: ActivePointSet,
    bounds: Bounds,
}

impl FromStr for Cube {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Cube::*;
        match s.chars().next() {
            None => anyhow::bail!("No character given to create a cube"),
            Some('#') => Ok(Active),
            Some('.') => Ok(Inactive),
            _ => anyhow::bail!("Invalid cube state"),
        }
    }
}

impl FromStr for Grid4D {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut y_len = 0;
        let mut x_len = 0;
        let g: Vec<(Point4D, Cube)> = s
            .lines()
            .enumerate()
            .flat_map(|(y, line)| {
                y_len = y_len.max(y as isize);
                line.chars().enumerate().map(move |(x, character)| {
                    x_len = x_len.max(x as isize);
                    character
                        .to_string()
                        .parse::<Cube>()
                        .map(|cube| (Point4D::new(x as isize, y as isize, 0, 0), cube))
                })
            })
            .try_collect()?;
        let g = g
            .into_iter()
            .filter(|(_, cube)| *cube == Cube::Active)
            .map(|(p, _)| p)
            .collect();

        Ok(Grid4D {
            grid: g,
            bounds: Bounds::new(0..=(x_len - 1), 0..=(y_len - 1), 0..=0, 0..=0),
        })
    }
}

impl std::fmt::Display for Grid4D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for w in self.bounds.w_range.clone() {
            for z in self.bounds.z_range.clone() {
                writeln!(f, "z={}, w={}", z, w)?;
                for y in self.bounds.y_range.clone() {
                    for x in self.bounds.x_range.clone() {
                        let p = Point4D::new(x, y, z, w);
                        write!(f, "{}", self.get(&p))?;
                    }
                    writeln!(f)?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl Bounds {
    fn new(
        x_range: RangeInclusive<isize>,
        y_range: RangeInclusive<isize>,
        z_range: RangeInclusive<isize>,
        w_range: RangeInclusive<isize>,
    ) -> Bounds {
        Bounds {
            x_range,
            y_range,
            z_range,
            w_range,
        }
    }
}

impl Point4D {
    fn new(x: isize, y: isize, z: isize, w: isize) -> Point4D {
        Point4D { x, y, z, w }
    }
}

impl std::ops::Add<Point4DTuple> for Point4D {
    type Output = Self;

    fn add(self, other: Point4DTuple) -> Self {
        Self {
            x: self.x + other.0,
            y: self.y + other.1,
            z: self.z + other.2,
            w: self.w + other.3,
        }
    }
}

impl Point4DNeighboursIter {
    fn get_delta_3d(i: u8) -> Point4DTuple {
        // The first 26 values coincide for both 3d and 4d neighbors.
        Self::get_delta_4d(i)
    }

    fn get_delta_4d(i: u8) -> Point4DTuple {
        static NEIGHBOR_VEC: Lazy<Vec<Point4DTuple>> = Lazy::new(|| {
            let range = -1..=1;
            // Creates a cross-product iterator of the 4 ranges.
            let w_cube = |w_range| {
                itertools::iproduct!(range.clone(), range.clone(), range.clone(), w_range)
            };
            // First visit the points that have w == 0, so that they can be used
            // for 3d traversal for part 1. Chain the other 'w' coordinates after those.
            w_cube(0..=0)
                // Skip the all-0 point, because it's not a neighbor.
                .filter(|p| !matches!(p, (0, 0, 0, 0)))
                .chain(w_cube(-1..=-1))
                .chain(w_cube(1..=1))
                .collect_vec()
        });

        NEIGHBOR_VEC[i as usize]
    }
}

impl std::iter::Iterator for Point4DNeighboursIter {
    type Item = Point4D;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_direction
            .and_then(|i| if i < self.max_id { Some(i) } else { None })
            .map(|i| {
                self.next_direction = Some(i + 1);
                let delta = match self.iter_kind {
                    PointIterKind::D3 => Point4DNeighboursIter::get_delta_3d(i),
                    PointIterKind::D4 => Point4DNeighboursIter::get_delta_4d(i),
                };
                self.pos + delta
            })
    }
}

impl std::iter::Iterator for ActivePoint4DIter<'_> {
    type Item = Point4D;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
    }
}

impl Grid4D {
    const NEIGHBOR_COUNT_3D: u8 = 26;
    const NEIGHBOR_COUNT_4D: u8 = 80;

    fn get(&self, p: &Point4D) -> Cube {
        self.grid
            .get(p)
            .map(|_| Cube::Active)
            .unwrap_or(Cube::Inactive)
    }

    fn point_neighbors_iter(
        &self,
        p: &Point4D,
        iter_kind: &PointIterKind,
    ) -> Point4DNeighboursIter {
        let max_id = match iter_kind {
            PointIterKind::D3 => Self::NEIGHBOR_COUNT_3D,
            PointIterKind::D4 => Self::NEIGHBOR_COUNT_4D,
        };
        Point4DNeighboursIter {
            pos: *p,
            max_id,
            iter_kind: iter_kind.clone(),
            next_direction: Some(0),
        }
    }

    fn active_point_iter(&self) -> ActivePoint4DIter {
        ActivePoint4DIter {
            iter: self.grid.iter(),
        }
    }
}

fn compute_bounds(g: &ActivePointSet) -> Bounds {
    // TODO: Cleaner way to do this?
    let p = g.iter().next().unwrap();
    let mut x_min = p.x;
    let mut x_max = p.x;
    let mut y_min = p.y;
    let mut y_max = p.y;
    let mut z_min = p.z;
    let mut z_max = p.z;
    let mut w_min = p.w;
    let mut w_max = p.w;
    g.iter().for_each(|p| {
        x_min = x_min.min(p.x);
        x_max = x_max.max(p.x);
        y_min = y_min.min(p.y);
        y_max = y_max.max(p.y);
        z_min = z_min.min(p.z);
        z_max = z_max.max(p.z);
        w_min = w_min.min(p.w);
        w_max = w_max.max(p.w);
    });
    Bounds::new(x_min..=x_max, y_min..=y_max, z_min..=z_max, w_min..=w_max)
}

fn simulate_one_cycle(s: &Grid4D, iter_kind: &PointIterKind) -> Grid4D {
    let mut active_neighbor_counter = ActiveNeighborCounter::new();

    s.active_point_iter().for_each(|active_p| {
        s.point_neighbors_iter(&active_p, iter_kind)
            .for_each(|neighbor_p| *active_neighbor_counter.entry(neighbor_p).or_insert(0) += 1);
    });
    let grid = active_neighbor_counter
        .into_iter()
        .filter(|&(p, count)| count == 3 || (count == 2 && s.grid.contains(&p)))
        .map(|(p, _)| p)
        .collect();
    let bounds = compute_bounds(&grid);
    Grid4D { grid, bounds }
}

fn count_active_cubes_after_six_cycles(s: &str, iter_kind: &PointIterKind) -> u64 {
    let s = s.parse::<Grid4D>().expect("Invalid grid");
    let mut s = s;
    (1..=6).for_each(|_| {
        s = simulate_one_cycle(&s, iter_kind);
    });
    s.grid.iter().count() as u64
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d17").context("Coudn't read file contents.")?;
    let result = count_active_cubes_after_six_cycles(&input, &PointIterKind::D3);
    println!(
        "Nunmber of active cubes after six cycles in 3d pocket dimension: {}",
        result
    );
    Ok(())
}

fn solve_p2() -> Result<()> {
    let input = helpers::get_data_from_file_res("d17").context("Coudn't read file contents.")?;
    let result = count_active_cubes_after_six_cycles(&input, &PointIterKind::D4);
    println!(
        "Nunmber of active cubes after six cycles in 4d pocket dimension: {}",
        result
    );
    Ok(())
}

fn main() -> Result<()> {
    solve_p1().ok();
    solve_p2().ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p1() {
        let input = "\
.#.
..#
###";
        let result = count_active_cubes_after_six_cycles(input, &PointIterKind::D3);
        assert_eq!(result, 112);
    }

    #[test]
    fn test_p2() {
        let input = "\
.#.
..#
###";
        let result = count_active_cubes_after_six_cycles(input, &PointIterKind::D4);
        assert_eq!(result, 848);
    }
}
