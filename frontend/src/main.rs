pub mod api;
mod app;
mod pages;

use leptos::prelude::*;

fn main() {
    mount_to_body(app::App);
}
