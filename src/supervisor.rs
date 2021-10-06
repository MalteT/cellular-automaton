use std::mem;

use nalgebra::{Point2, Translation2};
use web_sys::CanvasRenderingContext2d;

use crate::{
    automaton::{Automaton, Grid},
    CELL_WIDTH,
};

pub struct Supervisor<A: Automaton> {
    pub trans: Translation2<f64>,
    pub scale: Scale,
    front_buf: Grid<A::State>,
    swap_buf: Grid<A::State>,
}

impl<A: Automaton> Supervisor<A> {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = Grid::generate(width, height);
        Self {
            front_buf: grid.clone(),
            swap_buf: grid,
            trans: Translation2::from([0.0, 0.0]),
            scale: Scale::Auto(1.0),
        }
    }

    pub fn reset_zoom(&mut self, target_width: u32, target_height: u32) {
        let target_width = target_width as f64;
        let target_height = target_height as f64;
        let curr_width = self.width() as f64 * CELL_WIDTH as f64;
        let curr_height = self.height() as f64 * CELL_WIDTH as f64;
        let width_scale = target_width / curr_width;
        let height_scale = target_height / curr_height;
        let min_scale = width_scale.min(height_scale);
        self.scale = Scale::Auto(min_scale);
        let offset_x = (target_width / min_scale - curr_width) / 2.0;
        let offset_y = (target_height / min_scale - curr_height) / 2.0;
        self.trans = Translation2::from([offset_x, offset_y]);
    }

    pub fn update(&mut self) {
        mem::swap(&mut self.front_buf, &mut self.swap_buf);
        for x in 0..self.front_buf.width() {
            let x = x as isize;
            for y in 0..self.front_buf.height() {
                let y = y as isize;
                let new = A::update((x, y), &self.swap_buf);
                self.front_buf[(x, y)] = new;
            }
        }
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d) {
        for x in 0..self.front_buf.width() {
            for y in 0..self.front_buf.height() {
                let state = &self.front_buf[(x as isize, y as isize)];
                ctx.set_fill_style(&A::style(state));
                let pos = self.to_screen_coordinates(Point2::from([
                    (x * CELL_WIDTH) as f64 + 1.0,
                    (y * CELL_WIDTH) as f64 + 1.0,
                ]));
                let size = (CELL_WIDTH as f64 - 2.0) * self.scale.raw();
                ctx.fill_rect(pos.x, pos.y, size, size);
            }
        }
    }

    pub fn toggle(&mut self, x: isize, y: isize) {
        let old = self.front_buf[(x, y)].clone();
        self.front_buf[(x, y)] = A::toggle(old);
    }

    pub fn to_screen_coordinates(&self, obj: Point2<f64>) -> Point2<f64> {
        self.scale.raw() * self.trans.transform_point(&obj)
    }

    pub fn from_screen_coordinates(&self, obj: Point2<f64>) -> Point2<f64> {
        self.trans
            .inverse_transform_point(&(obj / self.scale.raw()))
    }

    pub fn width(&self) -> usize {
        self.front_buf.width()
    }

    pub fn height(&self) -> usize {
        self.front_buf.height()
    }
}

pub enum Scale {
    Manual(f64),
    Auto(f64),
}

impl Scale {
    pub fn raw(&self) -> f64 {
        match self {
            Self::Manual(s) | Self::Auto(s) => *s,
        }
    }
}
