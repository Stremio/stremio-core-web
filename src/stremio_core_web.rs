use crate::env::WebEnv;
use crate::event::WebEvent;
use crate::model::WebModel;
use futures::{future, StreamExt};
use lazy_static::lazy_static;
use std::sync::RwLock;
use stremio_core::constants::{
    LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY, PROFILE_STORAGE_KEY,
};
use stremio_core::models::common::Loadable;
use stremio_core::runtime::msg::Action;
use stremio_core::runtime::{Env, EnvError, Runtime, RuntimeAction, RuntimeEvent};
use stremio_core::types::library::LibraryBucket;
use stremio_core::types::profile::Profile;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

lazy_static! {
    static ref RUNTIME: RwLock<Option<Loadable<Runtime<WebEnv, WebModel>, EnvError>>> =
        Default::default();
}

#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub async fn initialize_runtime(emit_to_ui: js_sys::Function) -> Result<(), JsValue> {
    if RUNTIME.read().expect("runtime read failed").is_some() {
        panic!("unable to initialize runtime multiple times");
    };

    *RUNTIME.write().expect("runtime write failed") = Some(Loadable::Loading);
    let env_init_result = WebEnv::init().await;
    match env_init_result {
        Ok(_) => {
            let storage_result = future::try_join3(
                WebEnv::get_storage::<Profile>(PROFILE_STORAGE_KEY),
                WebEnv::get_storage::<LibraryBucket>(LIBRARY_RECENT_STORAGE_KEY),
                WebEnv::get_storage::<LibraryBucket>(LIBRARY_STORAGE_KEY),
            )
            .await;
            match storage_result {
                Ok((profile, recent_bucket, other_bucket)) => {
                    let profile = profile.unwrap_or_default();
                    let mut library = LibraryBucket::new(profile.uid(), vec![]);
                    if let Some(recent_bucket) = recent_bucket {
                        library.merge_bucket(recent_bucket);
                    };
                    if let Some(other_bucket) = other_bucket {
                        library.merge_bucket(other_bucket);
                    };
                    let (model, effects) = WebModel::new(profile, library);
                    let (runtime, rx) = Runtime::<WebEnv, _>::new(model, effects, 1000);
                    WebEnv::exec(rx.for_each(move |event| {
                        if let RuntimeEvent::CoreEvent(event) = &event {
                            let runtime = RUNTIME.read().expect("runtime read failed");
                            let runtime = runtime
                                .as_ref()
                                .expect("runtime is not ready")
                                .as_ref()
                                .expect("runtime is not ready");
                            let model = runtime.model().expect("model read failed");
                            WebEnv::emit_to_analytics(
                                &WebEvent::CoreEvent(event.to_owned()),
                                &model,
                            );
                        };
                        emit_to_ui
                            .call1(&JsValue::NULL, &JsValue::from_serde(&event).unwrap())
                            .expect("emit event failed");
                        future::ready(())
                    }));
                    *RUNTIME.write().expect("runtime write failed") =
                        Some(Loadable::Ready(runtime));
                    Ok(())
                }
                Err(error) => {
                    *RUNTIME.write().expect("runtime write failed") =
                        Some(Loadable::Err(error.to_owned()));
                    Err(JsValue::from_serde(&error).unwrap())
                }
            }
        }
        Err(error) => {
            *RUNTIME.write().expect("runtime write failed") = Some(Loadable::Err(error.to_owned()));
            Err(JsValue::from_serde(&error).unwrap())
        }
    }
}

#[wasm_bindgen]
pub fn get_state(field: JsValue) -> JsValue {
    let field = field.into_serde().expect("get state failed");
    let runtime = RUNTIME.read().expect("runtime read failed");
    let runtime = runtime
        .as_ref()
        .expect("runtime is not ready")
        .as_ref()
        .expect("runtime is not ready");
    let model = runtime.model().expect("model read failed");
    model.get_state(&field)
}

#[wasm_bindgen]
pub fn dispatch(action: JsValue, field: JsValue) {
    let action = action.into_serde::<Action>().expect("dispatch failed");
    let field = field.into_serde().expect("dispatch failed");
    let runtime = RUNTIME.read().expect("runtime read failed");
    let runtime = runtime
        .as_ref()
        .expect("runtime is not ready")
        .as_ref()
        .expect("runtime is not ready");
    {
        let model = runtime.model().expect("model read failed");
        WebEnv::emit_to_analytics(&WebEvent::CoreAction(action.to_owned()), &model);
    }
    runtime.dispatch(RuntimeAction { action, field });
}

#[wasm_bindgen]
pub fn analytics(event: JsValue) {
    let event = event.into_serde().expect("analytics failed");
    let runtime = RUNTIME.read().expect("runtime read failed");
    let runtime = runtime
        .as_ref()
        .expect("runtime is not ready")
        .as_ref()
        .expect("runtime is not ready");
    let model = runtime.model().expect("model read failed");
    WebEnv::emit_to_analytics(&WebEvent::UIEvent(event), &model);
}
