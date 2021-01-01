use advent::helpers;
use anyhow::{Context, Result};
use derive_more::{Add, AddAssign, Display, Mul, Sub};
use itertools::Itertools;

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
enum MoveDirection {
    #[display(fmt = "N")]
    North,
    #[display(fmt = "E")]
    East,
    #[display(fmt = "S")]
    South,
    #[display(fmt = "W")]
    West,
}

#[derive(Debug, Display)]
enum RotationAmount {
    #[display(fmt = "D90")]
    D90,
    #[display(fmt = "D180")]
    D180,
    #[display(fmt = "D270")]
    D270,
}

#[derive(Debug, Display)]
enum RotationDirection {
    #[display(fmt = "L")]
    Left,
    #[display(fmt = "R")]
    Right,
}

type MoveAmount = isize;

#[derive(Debug, Display)]
enum Op {
    #[display(fmt = "M_{}_{}", _0, _1)]
    Move(MoveDirection, MoveAmount),
    #[display(fmt = "R_{}_{}", _0, _1)]
    Rotate(RotationDirection, RotationAmount),
    #[display(fmt = "F_{}", _0)]
    Forward(MoveAmount),
}

type Ops = Vec<Op>;

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, Add, AddAssign, Mul, Sub)]
#[display(fmt = "({},{})", _0, _1)]
struct Pos(isize, isize);

#[derive(Debug, PartialEq, Eq)]
struct NavigationState {
    pos: Pos,
    move_dir: MoveDirection,
}

fn validate_rotation_amount(a: isize) -> anyhow::Result<RotationAmount> {
    match a {
        90 => Ok(RotationAmount::D90),
        180 => Ok(RotationAmount::D180),
        270 => Ok(RotationAmount::D270),
        _ => anyhow::bail!("Invalid rotation amount"),
    }
}

fn parse_ops(s: &str) -> anyhow::Result<Ops> {
    s.trim()
        .lines()
        .map(|l| {
            let (op, amount) = l.split_at(1);
            let amount = amount.parse::<isize>()?;
            let op = op
                .chars()
                .next()
                .ok_or_else(|| anyhow::anyhow!("No op char"))?;
            match op {
                'F' => Ok(Op::Forward(amount)),
                'N' => Ok(Op::Move(MoveDirection::North, amount)),
                'S' => Ok(Op::Move(MoveDirection::South, amount)),
                'W' => Ok(Op::Move(MoveDirection::West, amount)),
                'E' => Ok(Op::Move(MoveDirection::East, amount)),
                'L' => Ok(Op::Rotate(
                    RotationDirection::Left,
                    validate_rotation_amount(amount)?,
                )),
                'R' => Ok(Op::Rotate(
                    RotationDirection::Right,
                    validate_rotation_amount(amount)?,
                )),
                _ => anyhow::bail!("Invalid op whole"),
            }
        })
        .try_collect()
}

impl MoveDirection {
    fn get_pos_delta(&self) -> Pos {
        match self {
            MoveDirection::North => Pos(0, 1),
            MoveDirection::South => Pos(0, -1),
            MoveDirection::West => Pos(-1, 0),
            MoveDirection::East => Pos(1, 0),
        }
    }

    fn next_cw(&self) -> MoveDirection {
        match self {
            MoveDirection::North => MoveDirection::East,
            MoveDirection::East => MoveDirection::South,
            MoveDirection::South => MoveDirection::West,
            MoveDirection::West => MoveDirection::North,
        }
    }

    fn next_ccw(&self) -> MoveDirection {
        match self {
            MoveDirection::North => MoveDirection::West,
            MoveDirection::West => MoveDirection::South,
            MoveDirection::South => MoveDirection::East,
            MoveDirection::East => MoveDirection::North,
        }
    }
}

fn rotate_direction(
    amount: &RotationAmount,
    rotate_dir: &RotationDirection,
    direction: MoveDirection,
) -> MoveDirection {
    let rotate_fn = match rotate_dir {
        RotationDirection::Left => MoveDirection::next_ccw,
        RotationDirection::Right => MoveDirection::next_cw,
    };
    let mut rotate_iter = itertools::iterate(direction, rotate_fn).skip(1);
    match amount {
        RotationAmount::D90 => rotate_iter.next().unwrap(),
        RotationAmount::D180 => rotate_iter.nth(1).unwrap(),
        RotationAmount::D270 => rotate_iter.nth(2).unwrap(),
    }
}

fn rotate_waypoint(amount: &RotationAmount, rotate_dir: &RotationDirection, pos: Pos) -> Pos {
    let rotate_cw = |pos: &Pos| Pos(pos.1, -pos.0);
    let rotate_ccw = |pos: &Pos| Pos(-pos.1, pos.0);
    let rotate_fn = match rotate_dir {
        RotationDirection::Right => rotate_cw,
        RotationDirection::Left => rotate_ccw,
    };
    let mut rotate_iter = itertools::iterate(pos, rotate_fn).skip(1);

    match amount {
        RotationAmount::D90 => rotate_iter.next().unwrap(),
        RotationAmount::D180 => rotate_iter.nth(1).unwrap(),
        RotationAmount::D270 => rotate_iter.nth(2).unwrap(),
    }
}

impl NavigationState {
    fn apply_op(&mut self, op: &Op) {
        match op {
            Op::Forward(amount) => self.pos += self.move_dir.get_pos_delta() * amount,
            Op::Move(direction, amount) => self.pos += direction.get_pos_delta() * amount,
            Op::Rotate(direction, amount) => {
                self.move_dir = rotate_direction(amount, direction, self.move_dir)
            }
        }
    }

    fn apply_op_using_waypoint(&mut self, ship: &mut NavigationState, op: &Op) {
        match op {
            Op::Forward(amount) => {
                let waypoint_delta = self.pos - ship.pos;
                self.pos += waypoint_delta * amount;
                ship.pos += waypoint_delta * amount;
            }
            Op::Move(direction, amount) => self.pos += direction.get_pos_delta() * amount,
            Op::Rotate(direction, amount) => {
                let waypoint_delta = self.pos - ship.pos;
                self.pos = ship.pos + rotate_waypoint(amount, direction, waypoint_delta);
            }
        }
    }
}

enum ComputationKind {
    Simple,
    UsingWaypoint,
}

fn compute_distance_between_start_and_end_pos(s: &str, kind: &ComputationKind) -> isize {
    let ops = parse_ops(s).expect("Invalid ops");

    let mut waypoint = NavigationState {
        pos: Pos(10, 1),
        move_dir: MoveDirection::East,
    };
    let mut simple = |op, ship: &mut NavigationState| ship.apply_op(op);
    let mut using_waypoint = |op, ship: &mut NavigationState| {
        waypoint.apply_op_using_waypoint(ship, op);
    };

    let nav_fn: &mut dyn FnMut(_, &mut NavigationState) = match kind {
        ComputationKind::Simple => &mut simple,
        ComputationKind::UsingWaypoint => &mut using_waypoint,
    };

    let initial_ship = NavigationState {
        pos: Pos(0, 0),
        move_dir: MoveDirection::East,
    };
    let final_ship = ops.iter().fold(initial_ship, |mut ship, op| {
        nav_fn(op, &mut ship);
        ship
    });

    final_ship.pos.0.abs() + final_ship.pos.1.abs()
}

fn solve_p1() -> Result<()> {
    let input = helpers::get_data_from_file_res("d12").context("Coudn't read file contents.")?;
    let result = compute_distance_between_start_and_end_pos(&input, &ComputationKind::Simple);
    println!("The manhattan distance is: {}", result);
    Ok(())
}

fn solve_p2() -> Result<()> {
    let input = helpers::get_data_from_file_res("d12").context("Coudn't read file contents.")?;
    let result =
        compute_distance_between_start_and_end_pos(&input, &ComputationKind::UsingWaypoint);
    println!("The manhattan distance using waypoints is: {}", result);
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
F10
N3
F7
R90
F11";
        let result = compute_distance_between_start_and_end_pos(input, &ComputationKind::Simple);
        assert_eq!(result, 25);
    }

    #[test]
    fn test_p2() {
        let input = "
F10
N3
F7
R90
F11";

        let result =
            compute_distance_between_start_and_end_pos(input, &ComputationKind::UsingWaypoint);
        assert_eq!(result, 286);
    }
}
