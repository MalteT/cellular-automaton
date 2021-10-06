use gloo_timers::callback::Interval;
use lazy_static::lazy_static;
use nalgebra::{Point2, Translation2};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use weblog::console_log;
use yew::prelude::*;

use std::{f64, mem};

mod automaton;

use automaton::{Automaton, Grid, Life};

const CANVAS_ID: &str = "canvas";
const CELL_WIDTH: usize = 50;

lazy_static! {
    static ref MIN_DRAG: Point2<i32> = Point2::new(5, 5);
}

#[wasm_bindgen(module = "/js/resize-canvas.js")]
extern "C" {
    fn setResizeHandler(id: &str, callback: &Closure<dyn Fn()>, timeout: u32);
}

enum Msg {
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    Redraw,
    Resized,
    Scroll(WheelEvent),
    Update,
}

struct Model<A: Automaton + 'static> {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    canvas: Option<HtmlCanvasElement>,
    context: Option<CanvasRenderingContext2d>,
    resize_callback: Closure<dyn Fn()>,
    automaton: AutomatonWrapper<A>,
    last_mouse_click: Option<Point2<i32>>,
}

struct AutomatonWrapper<A: Automaton> {
    trans: Translation2<f64>,
    scale: Scale,
    front_buf: Grid<A::State>,
    swap_buf: Grid<A::State>,
}

enum Scale {
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

impl<A: Automaton> AutomatonWrapper<A> {
    fn new(width: usize, height: usize) -> Self {
        let grid = Grid::generate(width, height);
        Self {
            front_buf: grid.clone(),
            swap_buf: grid,
            trans: Translation2::from([0.0, 0.0]),
            scale: Scale::Auto(1.0),
        }
    }

    fn update(&mut self) {
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

    fn to_screen_coordinates(&self, obj: Point2<f64>) -> Point2<f64> {
        self.scale.raw() * self.trans.transform_point(&obj)
    }

    fn from_screen_coordinates(&self, obj: Point2<f64>) -> Point2<f64> {
        self.trans
            .inverse_transform_point(&(obj / self.scale.raw()))
    }
}

impl<A: Automaton> Model<A> {
    fn draw(&mut self) {
        if let (Some(ctx), Some(canvas)) = (self.context.as_mut(), self.canvas.as_mut()) {
            // Clear the background
            ctx.set_fill_style(&JsValue::from("rgb(40,40,40)"));
            ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
            //let width = canvas.width() as f64;
            //let height = canvas.height() as f64;
            for x in 0..self.automaton.front_buf.width() {
                for y in 0..self.automaton.front_buf.height() {
                    let state = &self.automaton.front_buf[(x as isize, y as isize)];
                    ctx.set_fill_style(&A::style(state));
                    let pos = self.automaton.to_screen_coordinates(Point2::new(
                        (x * CELL_WIDTH) as f64 + 1.0,
                        (y * CELL_WIDTH) as f64 + 1.0,
                    ));
                    let size = (CELL_WIDTH as f64 - 2.0) * self.automaton.scale.raw();
                    ctx.fill_rect(pos.x, pos.y, size, size);
                }
            }
        }
    }
}

impl<A: Automaton + 'static> Component for Model<A> {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let render_link = link.clone();
        let render_interval = Interval::new(10_000, move || render_link.send_message(Msg::Update));
        render_interval.forget();
        Self {
            link: link.clone(),
            canvas_ref: NodeRef::default(),
            canvas: None,
            context: None,
            resize_callback: Closure::wrap(Box::from(move || link.send_message(Msg::Resized))),
            automaton: AutomatonWrapper::new(20, 20),
            last_mouse_click: None,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();
            let context: CanvasRenderingContext2d = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into()
                .unwrap();
            // Add resize handler to document
            setResizeHandler(CANVAS_ID, &self.resize_callback, 1500);
            // Initial resize
            self.link.send_message(Msg::Resized);

            self.canvas = Some(canvas);
            self.context = Some(context);
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Redraw => {
                self.draw();
                false
            }
            Msg::MouseDown(ev) => {
                self.last_mouse_click = Some(Point2::new(ev.client_x(), ev.client_y()));
                false
            }
            Msg::Update => {
                self.automaton.update();
                self.link.send_message(Msg::Redraw);
                false
            }
            Msg::MouseUp(ev) => {
                if let Some(from) = self.last_mouse_click {
                    let to = Point2::new(ev.client_x(), ev.client_y());
                    let diff = to - from;
                    if diff.x.abs() <= MIN_DRAG.x && diff.y.abs() <= MIN_DRAG.y {
                        // Not a drag, just a click
                        let pos = self.automaton.from_screen_coordinates(Point2::new(
                            ev.client_x() as f64,
                            ev.client_y() as f64,
                        ));
                        let x = pos.x as isize / CELL_WIDTH as isize;
                        let y = pos.y as isize / CELL_WIDTH as isize;
                        let old = self.automaton.front_buf[(x, y)].clone();
                        self.automaton.front_buf[(x, y)] = A::toggle(old);
                        self.link.send_message(Msg::Redraw);
                        false
                    } else {
                        self.automaton.trans = Translation2::from([
                            diff.x as f64 + self.automaton.trans.x,
                            diff.y as f64 + self.automaton.trans.y,
                        ]);
                        console_log!("trans", self.automaton.trans.x, self.automaton.trans.y, ev);
                        self.link.send_message(Msg::Redraw);
                        false
                    }
                } else {
                    false
                }
            }
            Msg::Scroll(ev) => {
                let mouse = Point2::new(ev.client_x() as f64, ev.client_y() as f64);
                let orig_pos = self.automaton.from_screen_coordinates(mouse);
                self.automaton.scale =
                    Scale::Manual(self.automaton.scale.raw() + 0.001 * ev.delta_y());
                self.automaton.scale = Scale::Manual(self.automaton.scale.raw().max(0.0));
                let trans_adj = mouse / self.automaton.scale.raw() - orig_pos;
                self.automaton.trans.x = trans_adj.x;
                self.automaton.trans.y = trans_adj.y;
                console_log!(&ev, self.automaton.scale.raw());
                self.link.send_message(Msg::Redraw);
                false
            }
            Msg::Resized => {
                if let (Scale::Auto(_), Some(canvas)) = (&self.automaton.scale, &self.canvas) {
                    let target_width = canvas.width() as f64;
                    let target_height = canvas.height() as f64;
                    let curr_width = self.automaton.front_buf.width() as f64 * CELL_WIDTH as f64;
                    let curr_height = self.automaton.front_buf.height() as f64 * CELL_WIDTH as f64;
                    console_log!(target_width, target_height, curr_width, curr_height);

                    let width_scale = target_width / curr_width;
                    let height_scale = target_height / curr_height;
                    let min_scale = width_scale.min(height_scale);
                    self.automaton.scale = Scale::Auto(min_scale);
                    let offset_x = (target_width / min_scale - curr_width) / 2.0;
                    let offset_y = (target_height / min_scale - curr_height) / 2.0;
                    self.automaton.trans = Translation2::from([offset_x, offset_y]);
                }
                console_log!(
                    "resized!",
                    self.automaton.trans.x,
                    self.automaton.trans.y,
                    self.automaton.scale.raw()
                );
                self.link.send_message(Msg::Redraw);
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let onmousedown = self.link.callback(|ev| Msg::MouseDown(ev));
        let onmouseup = self.link.callback(|ev| Msg::MouseUp(ev));
        let onwheel = self.link.callback(|ev| Msg::Scroll(ev));
        html! {
            <>
                <canvas ref=self.canvas_ref.clone() id="canvas"
                        onmousedown=onmousedown
                        onmouseup=onmouseup
                        onwheel=onwheel />
                <button class="over" onclick={self.link.callback(|_| Msg::Update)}>
                    { "Next" }
                </button>
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model<Life>>();
}
