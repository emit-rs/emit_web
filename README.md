# `emit_web`

[![web](https://github.com/emit-rs/emit_web/actions/workflows/web.yml/badge.svg)](https://github.com/emit-rs/emit_web/actions/workflows/web.yml)

[Current docs](https://docs.rs/emit_web/0.2.0/emit_web/index.html)

Use [`emit`](https://docs.rs/emit) in WebAssembly applications targeting NodeJS and the browser.

# Getting started

First, add `emit` and `emit_web` to your `Cargo.toml`:

```toml
[dependencies.emit]
version = "1"
# Important: Make sure you set `default-features = false`
default-features = false
features = ["std", "implicit_rt"]

[dependencies.emit_web]
version = "0.2.0"
```

Ensure you set `default-features = false` on `emit`, so it won't try compile dependencies that aren't compatible with WebAssembly.

Next, configure `emit` to use web APIs in its runtime:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn setup() {
    let _ = emit::setup()
        .emit_to(emit_web::console())
        .with_clock(emit_web::date_clock())
        .with_rng(emit_web::crypto_rng())
        .try_init();
}
```

The name of this function doesn't matter, you'll just need to call it somewhere early in your application.
You'll need to at least override the default clock and source of randomness, otherwise you'll get events without timestamps, and spans without ids.

# Output

`emit_web` will output events to the [Console API](https://developer.mozilla.org/en-US/docs/Web/API/Console_API), where they'll appear in browser dev tools.

![`emit` events written to the browser console](https://raw.githubusercontent.com/emit-rs/emit_web/refs/heads/main/asset/console-output.png)
