/// Tab Hoarder - Chrome Extension for Tab Management
/// Built with Rust + WASM + Yew

mod domain;
mod tab_data;
mod operations;
mod storage;
pub mod ui;

use wasm_bindgen::prelude::*;

// Set up panic hook for better error messages in the browser console
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
}

// Re-export core domain functions for JavaScript access
#[wasm_bindgen]
pub fn extract_domain(url: &str) -> String {
    domain::extract_domain(url).unwrap_or_else(|| "invalid".to_string())
}

// Start the Yew app for the popup
#[wasm_bindgen]
pub fn start_popup() {
    yew::Renderer::<ui::popup::App>::new().render();
}

// Start the Yew app for the collapsed tabs viewer
#[wasm_bindgen]
pub fn start_collapsed_viewer() {
    yew::Renderer::<ui::collapsed::CollapsedViewer>::new().render();
}
