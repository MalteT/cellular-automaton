use yew::{html, ComponentLink, Html};

use crate::{automaton::Automaton, Model, Msg};

#[derive(Debug, Clone, Default)]
pub struct Settings {
    visible: bool,
    auto_run: bool,
}

impl Settings {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn toggle_auto_run(&mut self) {
        self.auto_run = !self.auto_run;
    }

    pub fn auto_run(&self) -> bool {
        self.auto_run
    }

    pub fn html<A: Automaton>(&self, link: &ComponentLink<Model<A>>) -> Html {
        let toggle = link.callback(|_| Msg::ToggleSettings);
        html! {
            <>
                <button id="toggle-settings" onclick=toggle>
                </button>
                { if self.visible { self.menu_html(link) } else { html!{} } }
            </>
        }
    }

    fn menu_html<A: Automaton>(&self, link: &ComponentLink<Model<A>>) -> Html {
        let auto_run = if self.auto_run {
            "auto-run-on"
        } else {
            "auto-run-off"
        };
        let auto_run_cb = link.callback(|_| Msg::ToggleAutoRun);
        let auto_zoom_cb = link.callback(|_| Msg::ResetZoom);
        html! {
            <div id="settings">
                <button id="auto-zoom" onclick=auto_zoom_cb />
                <button id="auto-run" class={auto_run} onclick=auto_run_cb />
            </div>
        }
    }
}
