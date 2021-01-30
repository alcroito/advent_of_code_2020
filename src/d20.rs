use advent::helpers;
use anyhow::{Context, Result};
use derive_more::Display;
use helpers::grid::Grid;
use itertools::Itertools;
use num_integer::Roots;
use std::ops::RangeInclusive;
use std::{collections::VecDeque, str::FromStr};

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
enum Pixel {
    #[display(fmt = ".")]
    Empty,
    #[display(fmt = "#")]
    Full,
    #[display(fmt = " ")]
    Wildcard,
    #[display(fmt = "O")]
    Monster,
}

type TileId = u32;
type Pixels = Grid<Pixel>;
#[derive(Debug, Display, Clone)]
#[display(fmt = "Tile {}:\n{}", id, pixels)]
struct ImageTile {
    id: TileId,
    pixels: Pixels,
}

impl FromStr for Pixel {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.chars().next() {
            None => anyhow::bail!("No pixel character"),
            Some('.') => Ok(Pixel::Empty),
            Some('#') => Ok(Pixel::Full),
            Some(' ') => Ok(Pixel::Wildcard),
            Some('O') => Ok(Pixel::Monster),
            _ => anyhow::bail!("Invalid pixel"),
        }
    }
}

impl FromStr for ImageTile {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let id_line = s
            .lines()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No tile id found"))?;
        let first_newline = s
            .find('\n')
            .ok_or_else(|| anyhow::anyhow!("No newline found"))?;
        let pixels_str = s
            .get(first_newline..)
            .ok_or_else(|| anyhow::anyhow!("No pixel data found"))?;

        let id = id_line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("No separator found before tile id"))?
            .get(..4)
            .ok_or_else(|| anyhow::anyhow!("No tile id found"))?
            .parse::<TileId>()
            .map_err(|_| anyhow::anyhow!("Non numeric tile id found"))?;
        let pixels = pixels_str.parse::<Pixels>()?;

        ImageTile::new(id, pixels).ok()
    }
}

impl ImageTile {
    fn new(id: TileId, pixels: Grid<Pixel>) -> Self {
        ImageTile { id, pixels }
    }

    fn ok(self) -> Result<Self> {
        Ok(self)
    }

    /*
    1 2 3
    4 5 6
    7 8 9

    7 4 1 0,0 -> 0,2  1,0 -> 0,1  2,0 -> 0,0
    8 5 2 0,1 -> 1,2  1,1 -> 1,1  2,1 -> 1,0
    9 6 3 0,2 -> 2,2  1,2 -> 2,1  2,2 -> 2,0
    */
    fn rotate_cw(&mut self) {
        let pixels = &self.pixels;
        let mut copy = pixels.clone();
        let it = (0..pixels.rows()).cartesian_product(0..pixels.cols());
        it.for_each(|(r, c)| {
            let src = (r, c);
            let tgt = (c, pixels.rows() - 1 - r);
            let src_ref = pixels.get(src).unwrap();
            let copy_ref = copy.get_mut(tgt).unwrap();
            *copy_ref = *src_ref;
        });
        self.pixels = copy;
    }

    fn rotate_cw_count(&mut self, count: usize) {
        for _ in 0..count {
            self.rotate_cw()
        }
    }

    fn flip_horizontal(&mut self) {
        for r in 0..(self.pixels.rows() / 2) {
            for c in 0..self.pixels.cols() {
                let src = (r, c);
                let tgt = (self.pixels.rows() - 1 - r, c);
                let tmp = *self.pixels.get(src).unwrap();
                *self.pixels.get_mut(src).unwrap() = *self.pixels.get(tgt).unwrap();
                *self.pixels.get_mut(tgt).unwrap() = tmp;
            }
        }
    }

    fn flip_vertical(&mut self) {
        for r in 0..(self.pixels.rows()) {
            for c in 0..self.pixels.cols() / 2 {
                let src = (r, c);
                let tgt = (r, self.pixels.cols() - 1 - c);
                let tmp = *self.pixels.get(src).unwrap();
                *self.pixels.get_mut(src).unwrap() = *self.pixels.get(tgt).unwrap();
                *self.pixels.get_mut(tgt).unwrap() = tmp;
            }
        }
    }

    fn mutations_iter(&self) -> ImageTileMutationsIter {
        ImageTileMutationsIter {
            tile: self,
            next_index: Some(0),
        }
    }

    fn side_iter(&self, tile_side: &ImageTileSide) -> ImageTileSideIter {
        ImageTileSideIter {
            tile: self,
            tile_side: *tile_side,
            next_index: Some(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageTileSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl ImageTileSide {
    fn opposite(&self) -> ImageTileSide {
        match self {
            ImageTileSide::Top => ImageTileSide::Bottom,
            ImageTileSide::Right => ImageTileSide::Left,
            ImageTileSide::Bottom => ImageTileSide::Top,
            ImageTileSide::Left => ImageTileSide::Right,
        }
    }

    fn point_delta(&self) -> Point2DTuple {
        match self {
            ImageTileSide::Top => (-1, 0),
            ImageTileSide::Right => (0, 1),
            ImageTileSide::Bottom => (1, 0),
            ImageTileSide::Left => (0, -1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageTileMutationKind {
    Original,
    Rotate90,
    Rotate180,
    Rotate270,
    FlipHorizontal,
    FlipHorizontalRotate90,
    FlipHorizontalRotate180,
    FlipHorizontalRotate270,
    FlipVertical,
    FlipVerticalRotate90,
    FlipVerticalRotate180,
    FlipVerticalRotate270,
}

struct ImageTileMutationsIter<'a> {
    tile: &'a ImageTile,
    next_index: Option<usize>,
}

impl<'a> std::iter::Iterator for ImageTileMutationsIter<'_> {
    type Item = (ImageTile, ImageTileMutationKind);
    fn next(&mut self) -> Option<Self::Item> {
        self.next_index
            .and_then(|i| if i < 12 { Some(i) } else { None })
            .map(|i| {
                let mut tile = self.tile.clone();
                let kind = match i {
                    0 => ImageTileMutationKind::Original,
                    1 => {
                        tile.rotate_cw_count(1);
                        ImageTileMutationKind::Rotate90
                    }
                    2 => {
                        tile.rotate_cw_count(2);
                        ImageTileMutationKind::Rotate180
                    }
                    3 => {
                        tile.rotate_cw_count(3);
                        ImageTileMutationKind::Rotate270
                    }
                    4 => {
                        tile.flip_horizontal();
                        ImageTileMutationKind::FlipHorizontal
                    }
                    5 => {
                        tile.flip_horizontal();
                        tile.rotate_cw_count(1);
                        ImageTileMutationKind::FlipHorizontalRotate90
                    }
                    6 => {
                        tile.flip_horizontal();
                        tile.rotate_cw_count(2);
                        ImageTileMutationKind::FlipHorizontalRotate180
                    }
                    7 => {
                        tile.flip_horizontal();
                        tile.rotate_cw_count(3);
                        ImageTileMutationKind::FlipHorizontalRotate270
                    }
                    8 => {
                        tile.flip_vertical();
                        ImageTileMutationKind::FlipVertical
                    }
                    9 => {
                        tile.flip_vertical();
                        tile.rotate_cw_count(1);
                        ImageTileMutationKind::FlipVerticalRotate90
                    }
                    10 => {
                        tile.flip_vertical();
                        tile.rotate_cw_count(2);
                        ImageTileMutationKind::FlipVerticalRotate180
                    }
                    11 => {
                        tile.flip_vertical();
                        tile.rotate_cw_count(3);
                        ImageTileMutationKind::FlipVerticalRotate270
                    }
                    _ => unreachable!(),
                };
                self.next_index = Some(i + 1);
                (tile, kind)
            })
    }
}

struct ImageTileSideIter<'a> {
    tile: &'a ImageTile,
    tile_side: ImageTileSide,
    next_index: Option<usize>,
}

impl<'a> std::iter::Iterator for ImageTileSideIter<'_> {
    type Item = Pixel;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_index
            .and_then(|i| {
                if i < self.tile.pixels.rows() {
                    Some(i)
                } else {
                    None
                }
            })
            .map(|i| {
                let pos = match self.tile_side {
                    ImageTileSide::Top => (0, i),
                    ImageTileSide::Right => (i, self.tile.pixels.cols() - 1),
                    ImageTileSide::Bottom => (self.tile.pixels.rows() - 1, i),
                    ImageTileSide::Left => (i, 0),
                };
                self.next_index = Some(i + 1);
                *self.tile.pixels.get(pos).unwrap()
            })
    }
}

type ImageTilesMap = std::collections::HashMap<Point2D, ImageTile>;
#[derive(Debug, Clone)]
struct Image {
    tiles: ImageTilesMap,
    bounds: ImageTilePosBounds,
}

impl Image {
    fn new() -> Self {
        Self {
            tiles: ImageTilesMap::new(),
            bounds: ImageTilePosBounds {
                row_range: 0..=0,
                col_range: 0..=0,
            },
        }
    }

    fn display_ids(&self) -> ImageDisplayIds<'_> {
        ImageDisplayIds { image: self }
    }

    fn update_bounds(&mut self) {
        let p = self.tiles.keys().next().unwrap();
        let mut r_min = p.r;
        let mut r_max = p.r;
        let mut c_min = p.c;
        let mut c_max = p.c;
        self.tiles.keys().for_each(|p| {
            r_min = r_min.min(p.r);
            r_max = r_max.max(p.r);
            c_min = c_min.min(p.c);
            c_max = c_max.max(p.c);
        });
        self.bounds = ImageTilePosBounds::new(r_min..=r_max, c_min..=c_max)
    }
}

struct ImageDisplayIds<'a> {
    image: &'a Image,
}

impl<'a> std::fmt::Display for ImageDisplayIds<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in self.image.bounds.row_range.clone() {
            for c in self.image.bounds.col_range.clone() {
                let value = self.image.tiles.get(&Point2D::new(r, c));
                if let Some(value) = value {
                    write!(f, "{:5}", value.id)?;
                }
            }
            if r != *self.image.bounds.row_range.end() {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ImageDisplayTiles<'a> {
    image: &'a Image,
}

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Hash)]
#[display(fmt = "({}, {})", r, c)]
struct Point2D {
    r: isize,
    c: isize,
}

impl Point2D {
    fn new(r: isize, c: isize) -> Point2D {
        Point2D { r, c }
    }
}

type Point2DTuple = (isize, isize);
impl std::ops::Add<Point2DTuple> for Point2D {
    type Output = Self;

    fn add(self, other: Point2DTuple) -> Self {
        Self {
            r: self.r + other.0,
            c: self.c + other.1,
        }
    }
}

#[derive(Debug, Clone)]
struct ImageTilePosBounds {
    row_range: RangeInclusive<isize>,
    col_range: RangeInclusive<isize>,
}

impl ImageTilePosBounds {
    fn new(
        row_range: RangeInclusive<isize>,
        col_range: RangeInclusive<isize>,
    ) -> ImageTilePosBounds {
        ImageTilePosBounds {
            row_range,
            col_range,
        }
    }
}

// This might be an anti-pattern, but i just wanted to check if it works.
trait PrintableIter<T>
where
    Self: std::ops::Deref<Target = [T]>,
    T: std::fmt::Display,
{
    fn print(&self) {
        self.deref()
            .iter()
            .for_each(|value| println!("{}\n", value));
    }
}

impl<Value, Container> PrintableIter<Value> for Container
where
    Value: std::fmt::Display,
    Container: std::ops::Deref<Target = [Value]>,
{
}

fn parse_image_tiles(s: &str) -> Vec<ImageTile> {
    s.split("\n\n")
        .map(|one_tile_str| one_tile_str.parse::<ImageTile>())
        .try_collect()
        .expect("Invalid image tiles")
}

#[allow(unused)]
fn try_match_tiles(
    tile_1: &ImageTile,
    tile_2: &ImageTile,
) -> Option<(ImageTileSide, ImageTileMutationKind, ImageTile)> {
    let sides = [
        ImageTileSide::Top,
        ImageTileSide::Right,
        ImageTileSide::Bottom,
        ImageTileSide::Left,
    ];
    for tile_1_side in &sides {
        let maybe_match = try_match_tiles_with_side(tile_1, tile_1_side, tile_2);
        if maybe_match.is_some() {
            return maybe_match;
        }
    }
    None
}

fn try_match_tiles_with_sides(
    tile_1: &ImageTile,
    tile_1_sides: &[ImageTileSide],
    tile_2: &ImageTile,
) -> Option<(ImageTileSide, ImageTileMutationKind, ImageTile)> {
    for tile_1_side in tile_1_sides {
        let maybe_match = try_match_tiles_with_side(tile_1, tile_1_side, tile_2);
        if maybe_match.is_some() {
            return maybe_match;
        }
    }
    None
}

fn try_match_tiles_with_side(
    tile_1: &ImageTile,
    tile_1_side: &ImageTileSide,
    tile_2: &ImageTile,
) -> Option<(ImageTileSide, ImageTileMutationKind, ImageTile)> {
    let tile_2_side = tile_1_side.opposite();
    for (mutated_tile_2, kind) in tile_2.mutations_iter() {
        let is_match = tile_1
            .side_iter(tile_1_side)
            .eq(mutated_tile_2.side_iter(&tile_2_side));
        if is_match {
            return Some((*tile_1_side, kind, mutated_tile_2));
        }
    }
    None
}

fn tile_unoccupied_sides(tile_pos: &Point2D, image: &Image) -> Vec<ImageTileSide> {
    let sides = [
        ImageTileSide::Top,
        ImageTileSide::Right,
        ImageTileSide::Bottom,
        ImageTileSide::Left,
    ];
    sides
        .iter()
        .filter(|side| {
            let tile_pos = *tile_pos + side.point_delta();
            !image.tiles.contains_key(&tile_pos)
        })
        .cloned()
        .collect_vec()
}

fn solve_jigsaw(s: &str) -> Image {
    let mut tiles = parse_image_tiles(s)
        .into_iter()
        .collect::<VecDeque<ImageTile>>();
    println!("Initial tile count: {}", tiles.len());

    let mut unmatched = VecDeque::<ImageTile>::new();
    let mut image = Image::new();
    image
        .tiles
        .insert(Point2D::new(0, 0), tiles.pop_front().unwrap());
    loop {
        let mut tiles_added = false;
        let mut matched_tiles_to_add = Vec::<(Point2D, ImageTile)>::new();
        for (pos_1, tile_1) in image.tiles.iter() {
            while let Some(tile_2) = tiles.pop_front() {
                let tile_1_sides = tile_unoccupied_sides(pos_1, &image);
                let maybe_match = try_match_tiles_with_sides(tile_1, &tile_1_sides, &tile_2);
                if let Some((side_1, _kind, mutated_tile_2)) = maybe_match {
                    let pos_2 = *pos_1 + side_1.point_delta();
                    matched_tiles_to_add.push((pos_2, mutated_tile_2));
                    break;
                } else {
                    unmatched.push_front(tile_2);
                }
            }
            if !matched_tiles_to_add.is_empty() {
                tiles.extend(unmatched.drain(..));
                break;
            }
            std::mem::swap(&mut tiles, &mut unmatched);
        }
        if !matched_tiles_to_add.is_empty() {
            tiles_added = true;
        }
        image.tiles.extend(matched_tiles_to_add);
        image.update_bounds();
        if !tiles_added {
            println!("Remaining tile count: {}", tiles.len());
            break;
        }
    }

    println!("{}", image.display_ids());
    println!("Final tile count: {}", image.tiles.len());
    image
}

fn multiply_corner_tile_ids(s: &str) -> u64 {
    let image = solve_jigsaw(s);
    let rows = [
        *image.bounds.row_range.start(),
        *image.bounds.row_range.end(),
    ];
    let cols = [
        *image.bounds.col_range.start(),
        *image.bounds.col_range.end(),
    ];
    let result: u64 = rows
        .iter()
        .cartesian_product(cols.iter())
        .map(|(r, c)| {
            let point = Point2D::new(*r, *c);
            let id = image.tiles.get(&point).unwrap().id as u64;
            id
        })
        .product();
    result
}

fn assemble_final_image_tile(image: Image) -> ImageTile {
    let image_tile_rows = image.tiles.len().sqrt();
    let tile_side_size = image.tiles[&Point2D::new(0, 0)].pixels.rows();
    let image_side_size_no_borders = (tile_side_size - 2) * image_tile_rows;
    let pixels = vec![Pixel::Empty; image_side_size_no_borders * image_side_size_no_borders];

    let mut assembled_tile = ImageTile::new(
        0,
        Pixels::new(
            image_side_size_no_borders,
            image_side_size_no_borders,
            pixels,
        ),
    );

    for r in image.bounds.row_range.clone() {
        for c in image.bounds.col_range.clone() {
            let tile = &image.tiles[&Point2D::new(r, c)];
            let normalized_r = r + (-image.bounds.row_range.start());
            let normalized_c = c + (-image.bounds.col_range.start());
            let tile_shifting_r = normalized_r as usize * (tile_side_size - 2);
            let tile_shifting_c = normalized_c as usize * (tile_side_size - 2);
            for tile_r in 1..tile.pixels.rows() - 1 {
                for tile_c in 1..tile.pixels.cols() - 1 {
                    let pixel = tile.pixels.get((tile_r, tile_c)).unwrap();
                    let new_pos = (tile_r - 1 + tile_shifting_r, tile_c - 1 + tile_shifting_c);
                    *assembled_tile.pixels.get_mut(new_pos).unwrap() = *pixel;
                }
            }
        }
    }
    assembled_tile
}

const MONSTER_STR: &str = r"                  # 
#    ##    ##    ###
 #  #  #  #  #  #   ";

fn parse_monster() -> Result<Grid<Pixel>, anyhow::Error> {
    let s = MONSTER_STR;
    let g = s
        .lines()
        .flat_map(|l| l.chars().map(|c| c.to_string().parse::<Pixel>()))
        .try_collect()?;
    let rows = s.lines().count();
    let cols = s
        .lines()
        .next()
        .map(|l| l.chars().count())
        .ok_or_else(|| anyhow::anyhow!("Row has no tiles"))?;
    Ok(Grid::new(rows, cols, g))
}

fn monster() -> &'static ImageTile {
    static INSTANCE: once_cell::sync::Lazy<ImageTile> = once_cell::sync::Lazy::new(|| {
        // Can't use FromStr<Grid> because it does a trim() :/
        let pixels = parse_monster().expect("Invalid monster");
        ImageTile::new(1, pixels)
    });
    &INSTANCE
}

fn is_monster_at_pos(image: &ImageTile, pos: (usize, usize), monster: &ImageTile) -> bool {
    let monster_rows = monster.pixels.rows();
    let monster_cols = monster.pixels.cols();
    for r in 0..monster_rows {
        for c in 0..monster_cols {
            let monster_pixel = monster.pixels.get((r, c)).unwrap();
            let image_pixel = image.pixels.get((pos.0 + r, pos.1 + c)).unwrap();
            match (monster_pixel, image_pixel) {
                (Pixel::Full, Pixel::Full) => (),
                (Pixel::Full, _) => return false,
                (_, _) => (),
            }
        }
    }
    true
}

fn mark_monster_at_pos(image: &mut ImageTile, pos: (usize, usize), monster: &ImageTile) {
    let monster_rows = monster.pixels.rows();
    let monster_cols = monster.pixels.cols();
    for r in 0..monster_rows {
        for c in 0..monster_cols {
            let monster_pixel = monster.pixels.get((r, c)).unwrap();
            let image_pixel = image.pixels.get_mut((pos.0 + r, pos.1 + c)).unwrap();
            match (*monster_pixel, *image_pixel) {
                (Pixel::Full, Pixel::Full) => {
                    *image_pixel = Pixel::Monster;
                }
                (_, _) => (),
            }
        }
    }
}

fn mark_monsters(image: &ImageTile, monster: &ImageTile) -> Option<ImageTile> {
    for (mut mutated_image, _) in image.mutations_iter() {
        let mut image_has_monsters = false;
        for r in 0..(image.pixels.rows() - monster.pixels.rows()) {
            for c in 0..(image.pixels.cols() - monster.pixels.cols()) {
                let is_match = is_monster_at_pos(&mutated_image, (r, c), monster);
                if is_match {
                    image_has_monsters = true;
                    // println!("match at ({},{})", r, c);
                    mark_monster_at_pos(&mut mutated_image, (r, c), monster);
                }
            }
        }
        if image_has_monsters {
            return Some(mutated_image);
        }
    }
    None
}

fn filter_non_monster_pixels(image: &mut ImageTile) {
    for r in 0..image.pixels.rows() {
        for c in 0..image.pixels.cols() {
            let pixel = image.pixels.get_mut((r, c)).unwrap();
            match pixel {
                Pixel::Monster => (),
                _ => *pixel = Pixel::Wildcard,
            }
        }
    }
}

fn count_rough_water(image: &ImageTile) -> u32 {
    let mut count = 0;
    for r in 0..image.pixels.rows() {
        for c in 0..image.pixels.cols() {
            let pixel = image.pixels.get((r, c)).unwrap();
            if let Pixel::Full = pixel {
                count += 1;
            }
        }
    }
    count
}

fn check_water_roughness(s: &str) -> u32 {
    let image = solve_jigsaw(s);
    let tile = assemble_final_image_tile(image);
    let monster = monster();
    // println!("{}", monster);
    let image_with_monsters = mark_monsters(&tile, monster);
    if let Some(mut image_with_monsters) = image_with_monsters {
        let rought_water_count = count_rough_water(&image_with_monsters);
        filter_non_monster_pixels(&mut image_with_monsters);
        println!("{}", image_with_monsters);
        return rought_water_count;
    }
    0
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d20").context("Coudn't read file contents.")?;
    let result = multiply_corner_tile_ids(&input);
    println!("The multiplication of the 4 corner tile ids is: {}", result);
    Ok(())
}

fn solve_p2() -> Result<()> {
    let input = helpers::get_data_from_file_res("d20").context("Coudn't read file contents.")?;
    let result = check_water_roughness(&input);
    println!("Water roughness is: {}", result);
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
    fn test_basic_ops() {
        let tile = "
Tile 2311:
..##.#..#.
##..#.....
#...##..#.
####.#...#
##.##.###.
##...#.###
.#.#.#..##
..#....#..
###...#.#.
..###..###";
        let tile = tile.parse::<ImageTile>().unwrap();
        let mut tile_2 = tile.clone();

        tile_2.rotate_cw_count(4);
        assert_eq!(tile.pixels, tile_2.pixels);

        tile_2.flip_horizontal();
        tile_2.flip_horizontal();
        assert_eq!(tile.pixels, tile_2.pixels);

        tile_2.flip_vertical();
        tile_2.flip_vertical();
        assert_eq!(tile.pixels, tile_2.pixels);

        let side: String = tile
            .side_iter(&ImageTileSide::Top)
            .map(|elem| format!("{}", elem))
            .collect();
        assert_eq!(side, "..##.#..#.");

        let side: String = tile
            .side_iter(&ImageTileSide::Right)
            .map(|elem| format!("{}", elem))
            .collect();
        assert_eq!(side, "...#.##..#");

        let side: String = tile
            .side_iter(&ImageTileSide::Bottom)
            .map(|elem| format!("{}", elem))
            .collect();
        assert_eq!(side, "..###..###");

        let side: String = tile
            .side_iter(&ImageTileSide::Left)
            .map(|elem| format!("{}", elem))
            .collect();
        assert_eq!(side, ".#####..#.");

        // Iterator side equality.
        assert!(tile
            .side_iter(&ImageTileSide::Left)
            .eq(tile.side_iter(&ImageTileSide::Left)));
    }

    #[test]
    fn test_matcher() {
        let tile = "
Tile 1951:
#.##...##.
#.####...#
.....#..##
#...######
.##.#....#
.###.#####
###.##.##.
.###....#.
..#.#..#.#
#...##.#..";
        let tile_1 = tile.parse::<ImageTile>().unwrap();

        let tile = "
Tile 2311:
..##.#..#.
##..#.....
#...##..#.
####.#...#
##.##.###.
##...#.###
.#.#.#..##
..#....#..
###...#.#.
..###..###";
        let tile_2 = tile.parse::<ImageTile>().unwrap();

        let maybe_match = try_match_tiles(&tile_1, &tile_2).unwrap();
        assert_eq!(maybe_match.0, ImageTileSide::Right);
        assert_eq!(maybe_match.1, ImageTileMutationKind::Original);
    }

    #[test]
    fn test_p1() {
        macro_rules! test {
            ($expr: literal, $solution: expr) => {
                let input = helpers::get_data_from_file_res($expr)
                    .context("Coudn't read file contents.")
                    .unwrap();
                assert_eq!(multiply_corner_tile_ids(&input), $solution)
            };
        }

        test!("d20_sample", 20899048083289);
    }

    #[test]
    fn test_p2() {
        macro_rules! test {
            ($expr: literal, $solution: expr) => {
                let input = helpers::get_data_from_file_res($expr)
                    .context("Coudn't read file contents.")
                    .unwrap();
                assert_eq!(check_water_roughness(&input), $solution)
            };
        }

        test!("d20_sample", 273);
    }
}
