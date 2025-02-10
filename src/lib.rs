/*!
`emit_web`

Use [`emit`](https://docs.rs/emit) in WebAssembly applications targeting NodeJS and the browser.

# Getting started

First, add `emit` and `emit_web` to your `Cargo.toml`:

```toml
[dependencies.emit]
version = "0.11"
default-features = false
features = ["std", "implicit_rt"]

[dependencies.emit_web]
version = "0.1.0"
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
*/

#![doc(html_logo_url = "https://raw.githubusercontent.com/emit-rs/emit/main/asset/logo.svg")]
#![deny(missing_docs)]

use std::{ops::ControlFlow, time::Duration};

use emit::Props as _;
use js_sys::{Date, Object, Reflect};
use wasm_bindgen::prelude::*;

/**
An emitter based on the [Console API](https://developer.mozilla.org/en-US/docs/Web/API/Console_API).
*/
pub const fn console() -> ConsoleEmitter {
    ConsoleEmitter::new()
}

/**
An emitter based on the [Console API](https://developer.mozilla.org/en-US/docs/Web/API/Console_API).
*/
pub struct ConsoleEmitter {}

impl ConsoleEmitter {
    /**
    Create a new instance of the console emitter.
    */
    pub const fn new() -> Self {
        ConsoleEmitter {}
    }
}

impl emit::Emitter for ConsoleEmitter {
    fn emit<E: emit::event::ToEvent>(&self, evt: E) {
        let evt = evt.to_event();

        let msg = evt.msg().to_string();
        let extent = encode_extent(evt.extent());
        let props = encode_props(evt.props());

        match evt.props().pull("lvl") {
            Some(emit::Level::Debug) => console::debug(&msg, extent, props),
            Some(emit::Level::Info) => console::info(&msg, extent, props),
            Some(emit::Level::Warn) => console::warn(&msg, extent, props),
            Some(emit::Level::Error) => console::error(&msg, extent, props),
            _ => console::log(&msg, extent, props),
        }
    }

    fn blocking_flush(&self, _: std::time::Duration) -> bool {
        true
    }
}

fn encode_extent(extent: Option<&emit::Extent>) -> JsValue {
    let Some(extent) = extent else {
        return JsValue::NULL;
    };

    let map = Object::new();

    let timestamp_millis = duration_millis_f64(extent.as_point().to_unix());

    let _ = Reflect::set(
        &map,
        &JsValue::from_str("timestamp"),
        &JsValue::from(Date::new(&JsValue::from_f64(timestamp_millis))),
    );

    if let Some(len) = extent.len() {
        let len_millis = duration_millis_f64(len);

        let _ = Reflect::set(
            &map,
            &JsValue::from_str("milliseconds"),
            &JsValue::from_f64(len_millis),
        );
    }

    map.into()
}

fn duration_millis_f64(d: Duration) -> f64 {
    let d_secs = d.as_secs() as f64;
    let d_subsec_nanos = d.subsec_nanos() as f64;

    d_secs * 1_000.0 + d_subsec_nanos / 1_000_000.0
}

fn encode_props(props: impl emit::Props) -> JsValue {
    struct PropsObject<P>(P);

    impl<P: emit::Props> serde::Serialize for PropsObject<P> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            use serde::ser::SerializeMap as _;

            let mut map = serializer.serialize_map(None)?;

            let mut r = Ok(());
            self.0.for_each(|k, v| {
                match (|| {
                    map.serialize_key(&k)?;
                    map.serialize_value(&v)
                })() {
                    Ok(()) => ControlFlow::Continue(()),
                    Err(e) => {
                        r = Err(e);
                        ControlFlow::Break(())
                    }
                }
            });
            r?;

            map.end()
        }
    }

    to_jsvalue(PropsObject(props))
}

fn to_jsvalue(v: impl serde::Serialize) -> JsValue {
    match v.serialize(&serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true)) {
        Ok(value) => value,
        Err(err) => err.into(),
    }
}

// NOTE: `Temporal.Now.instant()` would be good to support when available

/**
A clock based on the [Performance API](https://developer.mozilla.org/en-US/docs/Web/API/Performance_API).
*/
pub struct PerformanceClock {}

impl PerformanceClock {
    /**
    Create a new instance of the performance clock.
    */
    pub const fn new() -> Self {
        PerformanceClock {}
    }
}

/**
A clock based on the [Performance API](https://developer.mozilla.org/en-US/docs/Web/API/Performance_API).
*/
pub const fn performance_clock() -> PerformanceClock {
    PerformanceClock::new()
}

impl emit::Clock for PerformanceClock {
    fn now(&self) -> Option<emit::Timestamp> {
        emit::Timestamp::from_unix(performance_now())
    }
}

fn performance_now() -> Duration {
    let origin_millis = performance::PERFORMANCE.with(|performance| performance.time_origin());
    let now_millis = performance::now();

    let origin_nanos = (origin_millis * 1_000_000.0) as u128;
    let now_nanos = (now_millis * 1_000_000.0) as u128;

    let timestamp_nanos = origin_nanos + now_nanos;

    let timestamp_secs = (timestamp_nanos / 1_000_000_000) as u64;
    let timestamp_subsec_nanos = (timestamp_nanos % 1_000_000_000) as u32;

    Duration::new(timestamp_secs, timestamp_subsec_nanos)
}

/**
A clock based on the [Date type](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date).
*/
pub const fn date_clock() -> DateClock {
    DateClock::new()
}

/**
A clock based on the [Date type](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date).
*/
pub struct DateClock {}

impl DateClock {
    /**
    Create a new instance of the date clock.
    */
    pub const fn new() -> Self {
        DateClock {}
    }
}

impl emit::Clock for DateClock {
    fn now(&self) -> Option<emit::Timestamp> {
        emit::Timestamp::from_unix(date_now())
    }
}

fn date_now() -> Duration {
    let timestamp_millis = Date::new_0().get_time();

    let timestamp_nanos = (timestamp_millis * 1_000_000.0) as u128;

    let timestamp_secs = (timestamp_nanos / 1_000_000_000) as u64;
    let timestamp_subsec_nanos = (timestamp_nanos % 1_000_000_000) as u32;

    Duration::new(timestamp_secs, timestamp_subsec_nanos)
}

/**
A source of randomness based on the [Crypto API](https://developer.mozilla.org/en-US/docs/Web/API/Crypto).
*/
pub const fn crypto_rng() -> CryptoRng {
    CryptoRng::new()
}

/**
An RNG based on the [Crypto API](https://developer.mozilla.org/en-US/docs/Web/API/Crypto).
*/
pub struct CryptoRng {}

impl CryptoRng {
    /**
    Create a new instance of the crypto RNG.
    */
    pub const fn new() -> Self {
        CryptoRng {}
    }
}

impl emit::Rng for CryptoRng {
    fn fill<A: AsMut<[u8]>>(&self, mut arr: A) -> Option<A> {
        crypto_fill(arr.as_mut());

        Some(arr)
    }
}

fn crypto_fill(buf: &mut [u8]) {
    crypto::get_random_values(buf);
}

mod console {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(msg: &str, extent: JsValue, props: JsValue);
        #[wasm_bindgen(js_namespace = console)]
        pub fn debug(msg: &str, extent: JsValue, props: JsValue);
        #[wasm_bindgen(js_namespace = console)]
        pub fn info(msg: &str, extent: JsValue, props: JsValue);
        #[wasm_bindgen(js_namespace = console)]
        pub fn warn(msg: &str, extent: JsValue, props: JsValue);
        #[wasm_bindgen(js_namespace = console)]
        pub fn error(msg: &str, extent: JsValue, props: JsValue);
    }
}

mod performance {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        pub type Performance;

        #[wasm_bindgen(thread_local_v2, js_name = performance)]
        pub static PERFORMANCE: Performance;

        #[wasm_bindgen(method, getter = timeOrigin)]
        pub fn time_origin(this: &Performance) -> f64;

        #[wasm_bindgen(js_namespace = performance)]
        pub fn now() -> f64;
    }
}

mod crypto {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = crypto, js_name = getRandomValues)]
        pub fn get_random_values(buf: &mut [u8]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    use emit::{Clock as _, Rng as _};

    use std::collections::BTreeMap;

    #[wasm_bindgen_test]
    #[test]
    fn date_clock_produces_timestamps() {
        assert_ne!(emit::Timestamp::MIN, DateClock::new().now().unwrap());
    }

    #[wasm_bindgen_test]
    #[test]
    fn performance_clock_produces_timestamps() {
        assert_ne!(emit::Timestamp::MIN, PerformanceClock::new().now().unwrap());
    }

    #[wasm_bindgen_test]
    #[test]
    fn crypto_rng_produces_random_data() {
        let mut buf = [0; 32];

        CryptoRng::new().fill(&mut buf).unwrap();

        assert_ne!([0; 32], buf);
    }

    #[wasm_bindgen_test]
    #[test]
    fn emit() {
        let rt = emit::runtime::Runtime::default()
            .with_emitter(console())
            .with_clock(date_clock())
            .with_rng(crypto_rng());

        let data = {
            let mut map = BTreeMap::new();

            map.insert("c", 1);
            map.insert("d", 2);

            map
        };

        let name = "event";

        emit::emit!(rt, "test {name} with {#[emit::as_serde] data}");
    }

    #[wasm_bindgen_test]
    #[test]
    fn emit_span() {
        static RT: emit::runtime::Runtime<
            ConsoleEmitter,
            emit::Empty,
            emit::platform::thread_local_ctxt::ThreadLocalCtxt,
            DateClock,
            CryptoRng,
        > = emit::runtime::Runtime::build(
            console(),
            emit::Empty,
            emit::platform::thread_local_ctxt::ThreadLocalCtxt::shared(),
            date_clock(),
            crypto_rng(),
        );

        #[emit::span(rt: RT, "test {name} with {#[emit::as_serde] data}")]
        fn exec(name: &str, data: &BTreeMap<&str, i32>) {}

        let data = {
            let mut map = BTreeMap::new();

            map.insert("c", 1);
            map.insert("d", 2);

            map
        };

        let name = "event";

        exec(name, &data);
    }
}
