use serde::Serialize;
use stremio_core::models::installed_addons_with_filters::{
    InstalledAddonsRequest, InstalledAddonsWithFilters, Selected,
};
use stremio_deeplinks::AddonsDeepLinks;
use wasm_bindgen::JsValue;

mod model {
    use super::*;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DescriptorPreview<'a> {
        #[serde(flatten)]
        pub addon: &'a stremio_core::types::addon::DescriptorPreview,
        pub installed: bool,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableType<'a> {
        pub r#type: &'a Option<String>,
        pub selected: &'a bool,
        pub deep_links: AddonsDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SelectableCatalog {
        pub name: String,
        pub selected: bool,
        pub deep_links: AddonsDeepLinks,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Selectable<'a> {
        pub types: Vec<SelectableType<'a>>,
        pub catalogs: Vec<SelectableCatalog>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InstalledAddonsWithFilters<'a> {
        pub selected: &'a Option<Selected>,
        pub selectable: Selectable<'a>,
        pub catalog: Vec<DescriptorPreview<'a>>,
    }
}

pub fn serialize_installed_addons(installed_addons: &InstalledAddonsWithFilters) -> JsValue {
    JsValue::from_serde(&model::InstalledAddonsWithFilters {
        selected: &installed_addons.selected,
        selectable: model::Selectable {
            types: installed_addons
                .selectable
                .types
                .iter()
                .map(|selectable_type| model::SelectableType {
                    r#type: &selectable_type.r#type,
                    selected: &selectable_type.selected,
                    deep_links: AddonsDeepLinks::from(&selectable_type.request),
                })
                .collect(),
            catalogs: vec![model::SelectableCatalog {
                name: "Installed".to_owned(),
                selected: installed_addons.selected.is_some(),
                deep_links: AddonsDeepLinks::from(&InstalledAddonsRequest { r#type: None }),
            }],
        },
        catalog: installed_addons
            .catalog
            .iter()
            .map(|addon| model::DescriptorPreview {
                addon,
                installed: true,
            })
            .collect(),
    })
    .unwrap()
}
