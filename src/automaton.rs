use std::ops::{Index, IndexMut};

use wasm_bindgen::JsValue;

#[derive(Debug, Clone)]
pub struct Grid<State> {
    width: usize,
    height: usize,
    grid: Vec<State>,
}

impl<State: Default> Grid<State> {
    pub fn generate(width: usize, height: usize) -> Self {
        let grid = (0..width * height).map(|_| Default::default()).collect();
        Self {
            width,
            height,
            grid,
        }
    }
}

pub trait Automaton {
    type State: Default + Clone;
    type Dimension: Dimension;

    fn update(curr: (isize, isize), grid: &Grid<Self::State>) -> Self::State;

    fn toggle(curr: Self::State) -> Self::State;

    fn style(curr: &Self::State) -> JsValue;
}

pub struct Life;

pub trait Dimension {}
pub enum D2 {}
impl Dimension for D2 {}

#[derive(Debug, Clone)]
pub enum LifeStates {
    Dead,
    Alife,
}

impl Default for LifeStates {
    fn default() -> Self {
        Self::Dead
    }
}

impl Automaton for Life {
    type State = LifeStates;
    type Dimension = D2;

    fn update((pos_x, pos_y): (isize, isize), grid: &Grid<Self::State>) -> Self::State {
        let sum: u8 = MooreNeighbors::<1>::new()
            .filter(|(x, y)| *x != 0 || *y != 0)
            .map(|(x, y)| match &grid[(x + pos_x, y + pos_y)] {
                LifeStates::Dead => 0,
                LifeStates::Alife => 1,
            })
            .sum();
        let curr = grid[(pos_x, pos_y)].clone();
        match (sum, curr) {
            (2..=3, LifeStates::Alife) => LifeStates::Alife,
            (3, LifeStates::Dead) => LifeStates::Alife,
            _ => LifeStates::Dead,
        }
    }

    fn toggle(curr: Self::State) -> Self::State {
        match curr {
            LifeStates::Dead => LifeStates::Alife,
            LifeStates::Alife => LifeStates::Dead,
        }
    }

    fn style(curr: &Self::State) -> JsValue {
        match curr {
            LifeStates::Dead => JsValue::from_str("#1d2021"),
            LifeStates::Alife => JsValue::from_str("#ebdbb2"),
        }
    }
}

impl<State> Grid<State> {
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    fn to_idx(&self, x: isize, y: isize) -> usize {
        let x = if x >= 0 {
            x as usize % self.width
        } else {
            self.width - (x.abs() as usize % self.width)
        };
        let y = if y >= 0 {
            y as usize % self.height
        } else {
            self.height - (y.abs() as usize % self.height)
        };
        x + y * self.width
    }
}

impl<State> Index<(isize, isize)> for Grid<State> {
    type Output = State;

    fn index(&self, (x, y): (isize, isize)) -> &Self::Output {
        &self.grid[self.to_idx(x, y)]
    }
}

impl<State> IndexMut<(isize, isize)> for Grid<State> {
    fn index_mut(&mut self, (x, y): (isize, isize)) -> &mut Self::Output {
        let idx = self.to_idx(x, y);
        &mut self.grid[idx]
    }
}

pub struct MooreNeighbors<const RANGE: u16> {
    curr_x: isize,
    curr_y: isize,
    done: bool,
}

impl<const RANGE: u16> MooreNeighbors<RANGE> {
    pub fn new() -> Self {
        let min = -(RANGE as isize);
        Self {
            curr_x: min,
            curr_y: min,
            done: false,
        }
    }
}

impl<const RANGE: u16> Iterator for MooreNeighbors<RANGE> {
    type Item = (isize, isize);

    fn next(&mut self) -> Option<Self::Item> {
        let range = RANGE as isize;
        if self.done {
            None
        } else if self.curr_x == range && self.curr_y == range {
            // Last one
            self.done = true;
            Some((self.curr_x, self.curr_y))
        } else if self.curr_x == range {
            // Next row
            let ret = (self.curr_x, self.curr_y);
            self.curr_x = -range;
            self.curr_y += 1;
            Some(ret)
        } else {
            // Next column
            let ret = (self.curr_x, self.curr_y);
            self.curr_x += 1;
            Some(ret)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn moore_neighborhood_zero() {
        let neighs: Vec<_> = MooreNeighbors::<0>::new().collect();
        assert_eq!(neighs, vec![(0, 0)]);
    }

    #[test]
    fn moore_neighborhood_one() {
        let neighs: HashSet<_> = MooreNeighbors::<1>::new().collect();
        let mut eq = vec![];
        for x in -1..=1 {
            for y in -1..=1 {
                eq.push((x, y));
            }
        }
        assert_eq!(neighs, eq.into_iter().collect());
    }

    #[test]
    fn moore_neighborhood_two() {
        let neighs: HashSet<_> = MooreNeighbors::<2>::new().collect();
        let mut eq = vec![];
        for x in -2..=2 {
            for y in -2..=2 {
                eq.push((x, y));
            }
        }
        assert_eq!(neighs, eq.into_iter().collect());
    }
}
