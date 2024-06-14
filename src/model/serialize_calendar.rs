pub use model::Calendar;

use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::JsValue;

pub fn serialize_calendar(calendar: &stremio_core::models::calendar::Calendar) -> JsValue {
    <JsValue as JsValueSerdeExt>::from_serde(&Calendar::from(calendar))
        .expect("JsValue from Calendar")
}

mod model {
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Calendar<'a> {
        #[serde(flatten)]
        pub calendar: &'a stremio_core::models::calendar::Calendar,
    }

    impl<'a> From<&'a stremio_core::models::calendar::Calendar> for Calendar<'a> {
        fn from(calendar: &'a stremio_core::models::calendar::Calendar) -> Self {
            Self { calendar }
        }
    }
}
