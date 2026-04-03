use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn setup() {
    // Write panics to the console
    std::panic::set_hook(Box::new(emit_web::panic_hook(emit_web::console())));

    // Write regular events to the console
    let _ = emit::setup().emit_to(emit_web::console()).try_init();
}

#[wasm_bindgen]
pub fn run() {
    emit::debug!("Hello {user}", user: "Web");
    emit::info!("Hello {user}", user: "Web");
    emit::warn!("Hello {user}", user: "Web");
    emit::error!("Hello {user}", user: "Web");

    exec_expensive_operation();
}

#[emit::span("Expensive operation")]
fn exec_expensive_operation() {}
