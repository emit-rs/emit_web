# `emit_web`

[![web](https://github.com/emit-rs/emit_web/actions/workflows/web.yml/badge.svg)](https://github.com/emit-rs/emit_web/actions/workflows/web.yml)

[Current docs](https://docs.rs/emit_web/0.2.1/emit_web/index.html)

Use [`emit`](https://docs.rs/emit) in WebAssembly applications targeting NodeJS and the browser.

`emit` itself and some emitters, like [`emit_otlp`](https://docs.rs/emit_otlp) support WebAssembly directly. This library includes support for emitting events to the [Console API](https://developer.mozilla.org/en-US/docs/Web/API/console). It also has alternative clocks and randomness using different web features. These aren't required for configuration, but can be used to more directly control the JavaScript APIs `emit` makes use of.

`emit_web` also supports the `wasm32v1-none` target.

# Getting started

First, add `emit` and `emit_web` to your `Cargo.toml`:

```toml
[dependencies.emit]
version = "1"
features = ["std", "implicit_rt"]

[dependencies.emit_web]
version = "0.2.1"
```

Next, configure `emit` to use web APIs in its runtime:

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn setup() {
    let _ = emit::setup()
        .emit_to(emit_web::console())
        .try_init();
}
```

The name of this `setup` function doesn't matter, you'll just need to call it somewhere early in your application.

# Output

`emit_web` will output events to the [Console API](https://developer.mozilla.org/en-US/docs/Web/API/Console_API), where they'll appear in browser dev tools.

![`emit` events written to the browser console](https://raw.githubusercontent.com/emit-rs/emit_web/refs/heads/main/asset/console-output.png)
