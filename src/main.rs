use gloo_timers::callback::Interval;
use lazy_static::lazy_static;
use nalgebra::{Point2, Translation2};
use supervisor::Supervisor;
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast, JsValue,
};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use weblog::console_log;
use yew::prelude::*;

use std::f64;

mod automaton;
mod settings;
mod supervisor;

use automaton::{Automaton, Life};

use crate::{settings::Settings, supervisor::Scale};

const CANVAS_ID: &str = "canvas";
const CELL_WIDTH: usize = 50;
const TIME_BETWEEN_RENDERS_MS: u32 = 100;

lazy_static! {
    static ref MIN_DRAG: Point2<i32> = Point2::new(5, 5);
}

#[wasm_bindgen(module = "/js/resize-canvas.js")]
extern "C" {
    fn setResizeHandler(id: &str, callback: &Closure<dyn Fn()>, timeout: u32);
}

pub enum Msg {
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    Redraw,
    Resized,
    Scroll(WheelEvent),
    Update,
    ToggleSettings,
    ToggleAutoRun,
    ResetZoom,
}

pub struct Model<A: Automaton + 'static> {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    canvas: Option<HtmlCanvasElement>,
    context: Option<CanvasRenderingContext2d>,
    resize_callback: Closure<dyn Fn()>,
    automaton: Supervisor<A>,
    last_mouse_click: Option<Point2<i32>>,
    settings: Settings,
    render_timer: Option<Interval>,
}

impl<A: Automaton> Model<A> {
    fn draw(&mut self) {
        if let (Some(ctx), Some(canvas)) = (self.context.as_mut(), self.canvas.as_mut()) {
            // Clear the background
            ctx.set_fill_style(&JsValue::from("rgb(40,40,40)"));
            ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
            // Draw the current automaton
            self.automaton.draw(ctx);
        }
    }
}

impl<A: Automaton + 'static> Component for Model<A> {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link: link.clone(),
            canvas_ref: NodeRef::default(),
            canvas: None,
            context: None,
            resize_callback: Closure::wrap(Box::from(move || link.send_message(Msg::Resized))),
            automaton: Supervisor::new(20, 20),
            last_mouse_click: None,
            settings: Settings::default(),
            render_timer: None,
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
                self.last_mouse_click = Some(Point2::from([ev.client_x(), ev.client_y()]));
                false
            }
            Msg::Update => {
                self.automaton.update();
                self.link.send_message(Msg::Redraw);
                false
            }
            Msg::MouseUp(ev) => {
                if let Some(from) = self.last_mouse_click {
                    let to = Point2::from([ev.client_x(), ev.client_y()]);
                    let diff = to - from;
                    if diff.x.abs() <= MIN_DRAG.x && diff.y.abs() <= MIN_DRAG.y {
                        // Not a drag, just a click
                        let pos = self.automaton.from_screen_coordinates(Point2::from([
                            ev.client_x() as f64,
                            ev.client_y() as f64,
                        ]));
                        let x = pos.x as isize / CELL_WIDTH as isize;
                        let y = pos.y as isize / CELL_WIDTH as isize;
                        self.automaton.toggle(x, y);
                        self.link.send_message(Msg::Redraw);
                        false
                    } else {
                        self.automaton.trans = Translation2::from([
                            diff.x as f64 + self.automaton.trans.x,
                            diff.y as f64 + self.automaton.trans.y,
                        ]);
                        self.link.send_message(Msg::Redraw);
                        false
                    }
                } else {
                    false
                }
            }
            Msg::Scroll(ev) => {
                let mouse = Point2::from([ev.client_x() as f64, ev.client_y() as f64]);
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
                    self.automaton.reset_zoom(canvas.width(), canvas.height());
                }
                self.link.send_message(Msg::Redraw);
                false
            }
            Msg::ToggleSettings => {
                self.settings.toggle();
                true
            }
            Msg::ToggleAutoRun => {
                self.settings.toggle_auto_run();
                if self.settings.auto_run() {
                    let link = self.link.clone();
                    self.render_timer = Some(Interval::new(TIME_BETWEEN_RENDERS_MS, move || {
                        link.send_message(Msg::Update)
                    }));
                } else {
                    if let Some(interval) = self.render_timer.take() {
                        interval.cancel();
                    }
                }
                true
            }
            Msg::ResetZoom => {
                if let Some(canvas) = &self.canvas {
                    self.automaton.reset_zoom(canvas.width(), canvas.height());
                    self.link.send_message(Msg::Redraw);
                }
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
        let onmousedown = self.link.callback(Msg::MouseDown);
        let onmouseup = self.link.callback(Msg::MouseUp);
        let onwheel = self.link.callback(Msg::Scroll);
        html! {
            <>
                <canvas ref=self.canvas_ref.clone() id="canvas"
                        onmousedown=onmousedown
                        onmouseup=onmouseup
                        onwheel=onwheel />
                { self.settings.html(&self.link) }
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model<Life>>();
}
