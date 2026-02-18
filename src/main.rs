#![allow(dead_code)]

mod app;
mod config;
mod model;
mod net;
mod sieve;
mod store;
mod ui;

pub fn main() -> iced::Result {
    iced::application("Sievert — SIEVE Filter Manager", app::update, app::view)
        .subscription(app::subscription)
        .theme(app::theme)
        .font(ui::icons::ICON_FONT_BYTES)
        .window_size((1000.0, 650.0))
        .centered()
        .run()
}
