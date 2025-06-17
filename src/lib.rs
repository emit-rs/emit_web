/*!
`emit_web`

Use [`emit`](https://docs.rs/emit) in WebAssembly applications targeting NodeJS and the browser.

`emit` itself and some emitters, like [`emit_otlp`](https://docs.rs/emit_otlp) support WebAssembly directly. This library includes support for emitting events to the [Console API](https://developer.mozilla.org/en-US/docs/Web/API/console). It also has alternative clocks and randomness using different web features. These aren't required for configuration, but can be used to more directly control the JavaScript APIs `emit` makes use of.

# Getting started

First, add `emit` and `emit_web` to your `Cargo.toml`:

```toml
[dependencies.emit]
version = "1"

[dependencies.emit_web]
version = "0.2.0"
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
*/

#![doc(html_logo_url = "https://raw.githubusercontent.com/emit-rs/emit/main/asset/logo.svg")]
#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::string::ToString;
use core::{ops::ControlFlow, time::Duration};

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

    fn blocking_flush(&self, _: core::time::Duration) -> bool {
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
    match ser::jsvalue(v) {
        Ok(value) => value,
        Err(err) => err,
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

mod ser {
    use alloc::string::ToString;
    use core::fmt;

    use js_sys::{Array, Object, Reflect, Uint8Array};
    use serde::ser::{
        Error, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
        SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer, StdError,
    };
    use wasm_bindgen::prelude::*;

    pub fn jsvalue(v: impl Serialize) -> Result<JsValue, JsValue> {
        v.serialize(JsSerializer)
            .map_err(|e| JsValue::from(e.to_string()))
    }

    #[derive(Debug)]
    struct JsError;

    impl From<JsValue> for JsError {
        fn from(_: JsValue) -> Self {
            JsError
        }
    }

    impl fmt::Display for JsError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("failed to serialize a value to JavaScript")
        }
    }

    impl StdError for JsError {}

    impl Error for JsError {
        fn custom<T>(_: T) -> Self
        where
            T: fmt::Display,
        {
            JsError
        }
    }

    struct JsSerializer;

    struct JsArraySerializer {
        variant: Option<&'static str>,
        result: Array,
    }

    struct JsObjectSerializer {
        variant: Option<&'static str>,
        key: Option<JsValue>,
        result: Object,
    }

    impl Serializer for JsSerializer {
        type Ok = JsValue;
        type Error = JsError;
        type SerializeSeq = JsArraySerializer;
        type SerializeTuple = JsArraySerializer;
        type SerializeTupleStruct = JsArraySerializer;
        type SerializeTupleVariant = JsArraySerializer;
        type SerializeMap = JsObjectSerializer;
        type SerializeStruct = JsObjectSerializer;
        type SerializeStructVariant = JsObjectSerializer;

        fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::from(v))
        }

        fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
            self.serialize_i128(v as i128)
        }

        fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
            self.serialize_i128(v as i128)
        }

        fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
            self.serialize_i128(v as i128)
        }

        fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
            self.serialize_i128(v as i128)
        }

        fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
            if v.abs() > (i128::MAX >> 74) {
                Ok(JsValue::bigint_from_str(&v.to_string()))
            } else {
                Ok(JsValue::from(v as f64))
            }
        }

        fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
            self.serialize_u128(v as u128)
        }

        fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
            self.serialize_u128(v as u128)
        }

        fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
            self.serialize_u128(v as u128)
        }

        fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
            self.serialize_u128(v as u128)
        }

        fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
            if v > (u128::MAX >> 75) {
                Ok(JsValue::bigint_from_str(&v.to_string()))
            } else {
                Ok(JsValue::from(v as f64))
            }
        }

        fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::from(v as f64))
        }

        fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::from(v))
        }

        fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
            let mut buf = [0; 4];
            let v = v.encode_utf8(&mut buf);

            Ok(JsValue::from(&*v))
        }

        fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::from(v))
        }

        fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
            let buf = Uint8Array::from(v);

            Ok(JsValue::from(buf))
        }

        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::null())
        }

        fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            value.serialize(self)
        }

        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::null())
        }

        fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::from(name))
        }

        fn serialize_unit_variant(
            self,
            _: &'static str,
            _: u32,
            variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::from(variant))
        }

        fn serialize_newtype_struct<T>(
            self,
            _: &'static str,
            value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            value.serialize(self)
        }

        fn serialize_newtype_variant<T>(
            self,
            _: &'static str,
            _: u32,
            variant: &'static str,
            value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            jsvariant(variant, jsvalue(value)?)
        }

        fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            Ok(JsArraySerializer {
                variant: None,
                result: Array::new(),
            })
        }

        fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
            Ok(JsArraySerializer {
                variant: None,
                result: Array::new(),
            })
        }

        fn serialize_tuple_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            Ok(JsArraySerializer {
                variant: None,
                result: Array::new(),
            })
        }

        fn serialize_tuple_variant(
            self,
            _: &'static str,
            _: u32,
            variant: &'static str,
            _: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            Ok(JsArraySerializer {
                variant: Some(variant),
                result: Array::new(),
            })
        }

        fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            Ok(JsObjectSerializer {
                variant: None,
                key: None,
                result: Object::new(),
            })
        }

        fn serialize_struct(
            self,
            _: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            Ok(JsObjectSerializer {
                variant: None,
                key: None,
                result: Object::new(),
            })
        }

        fn serialize_struct_variant(
            self,
            _: &'static str,
            _: u32,
            variant: &'static str,
            _: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            Ok(JsObjectSerializer {
                variant: Some(variant),
                key: None,
                result: Object::new(),
            })
        }
    }

    impl SerializeSeq for JsArraySerializer {
        type Ok = JsValue;
        type Error = JsError;

        fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
        {
            self.result.push(&jsvalue(value)?);

            Ok(())
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::from(self.result))
        }
    }

    impl SerializeTuple for JsArraySerializer {
        type Ok = JsValue;
        type Error = JsError;

        fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
        {
            self.result.push(&jsvalue(value)?);

            Ok(())
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::from(self.result))
        }
    }

    impl SerializeTupleStruct for JsArraySerializer {
        type Ok = JsValue;
        type Error = JsError;

        fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
        {
            self.result.push(&jsvalue(value)?);

            Ok(())
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::from(self.result))
        }
    }

    impl SerializeTupleVariant for JsArraySerializer {
        type Ok = JsValue;
        type Error = JsError;

        fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
        {
            self.result.push(&jsvalue(value)?);

            Ok(())
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            jsvariant(
                self.variant
                    .ok_or_else(|| JsError::custom("missing variant"))?,
                JsValue::from(self.result),
            )
        }
    }

    impl SerializeMap for JsObjectSerializer {
        type Ok = JsValue;
        type Error = JsError;

        fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
        {
            self.key = Some(jsvalue(key)?);

            Ok(())
        }

        fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
        {
            let key = self
                .key
                .take()
                .ok_or_else(|| JsError::custom("missing key for a value"))?;

            Reflect::set(&self.result, &key, &jsvalue(value)?)?;

            Ok(())
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::from(self.result))
        }
    }

    impl SerializeStruct for JsObjectSerializer {
        type Ok = JsValue;
        type Error = JsError;

        fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
        {
            Reflect::set(&self.result, &JsValue::from(key), &jsvalue(value)?)?;

            Ok(())
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            Ok(JsValue::from(self.result))
        }
    }

    impl SerializeStructVariant for JsObjectSerializer {
        type Ok = JsValue;
        type Error = JsError;

        fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
        where
            T: ?Sized + Serialize,
        {
            Reflect::set(&self.result, &JsValue::from(key), &jsvalue(value)?)?;

            Ok(())
        }

        fn end(self) -> Result<Self::Ok, Self::Error> {
            jsvariant(
                self.variant
                    .ok_or_else(|| JsError::custom("missing variant"))?,
                JsValue::from(self.result),
            )
        }
    }

    fn jsvariant(variant: &str, value: JsValue) -> Result<JsValue, JsError> {
        let result = Object::new();

        Reflect::set(&result, &JsValue::from(variant), &value)?;

        Ok(JsValue::from(result))
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
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
