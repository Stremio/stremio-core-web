#[cfg(debug_assertions)]
use serde::Serialize;

use wasm_bindgen::JsValue;

use stremio_core::{
    models::{
        addon_details::AddonDetails,
        catalog_with_filters::CatalogWithFilters,
        catalogs_with_extra::CatalogsWithExtra,
        continue_watching_preview::ContinueWatchingPreview,
        ctx::Ctx,
        data_export::DataExport,
        installed_addons_with_filters::InstalledAddonsWithFilters,
        library_with_filters::{ContinueWatchingFilter, LibraryWithFilters, NotRemovedFilter},
        link::Link,
        local_search::LocalSearch,
        meta_details::MetaDetails,
        player::Player,
        streaming_server::StreamingServer,
    },
    runtime::Effects,
    types::{
        addon::DescriptorPreview, api::LinkAuthKey, library::LibraryBucket,
        notifications::NotificationsBucket, profile::Profile, resource::MetaItemPreview,
        streams::StreamsBucket,
    },
    Model,
};

use crate::{
    env::WebEnv,
    model::{
        serialize_catalogs_with_extra, serialize_continue_watching_preview, serialize_ctx,
        serialize_data_export, serialize_discover, serialize_installed_addons, serialize_library,
        serialize_local_search, serialize_meta_details, serialize_player, serialize_remote_addons,
        serialize_streaming_server,
    },
};

#[derive(Model, Clone)]
#[cfg_attr(debug_assertions, derive(Serialize))]
#[model(WebEnv)]
pub struct WebModel {
    pub ctx: Ctx,
    pub auth_link: Link<LinkAuthKey>,
    pub data_export: DataExport,
    pub continue_watching_preview: ContinueWatchingPreview,
    pub board: CatalogsWithExtra,
    pub discover: CatalogWithFilters<MetaItemPreview>,
    pub library: LibraryWithFilters<NotRemovedFilter>,
    pub continue_watching: LibraryWithFilters<ContinueWatchingFilter>,
    pub search: CatalogsWithExtra,
    /// Pre-loaded results for local search
    pub local_search: LocalSearch,
    pub meta_details: MetaDetails,
    pub remote_addons: CatalogWithFilters<DescriptorPreview>,
    pub installed_addons: InstalledAddonsWithFilters,
    pub addon_details: AddonDetails,
    pub streaming_server: StreamingServer,
    pub player: Player,
}

impl WebModel {
    pub fn new(
        profile: Profile,
        library: LibraryBucket,
        streams: StreamsBucket,
        notifications: NotificationsBucket,
    ) -> (WebModel, Effects) {
        let (continue_watching_preview, continue_watching_preview_effects) =
            ContinueWatchingPreview::new(&library, &notifications);
        let (discover, discover_effects) = CatalogWithFilters::<MetaItemPreview>::new(&profile);
        let (library_, library_effects) =
            LibraryWithFilters::<NotRemovedFilter>::new(&library, &notifications);
        let (continue_watching, continue_watching_effects) =
            LibraryWithFilters::<ContinueWatchingFilter>::new(&library, &notifications);
        let (remote_addons, remote_addons_effects) =
            CatalogWithFilters::<DescriptorPreview>::new(&profile);
        let (installed_addons, installed_addons_effects) =
            InstalledAddonsWithFilters::new(&profile);
        let (streaming_server, streaming_server_effects) = StreamingServer::new::<WebEnv>(&profile);
        let (local_search, local_search_effects) = LocalSearch::init::<WebEnv>();
        let model = WebModel {
            ctx: Ctx::new(profile, library, streams, notifications),
            auth_link: Default::default(),
            data_export: Default::default(),
            local_search,
            continue_watching_preview,
            board: Default::default(),
            discover,
            library: library_,
            continue_watching,
            search: Default::default(),
            meta_details: Default::default(),
            remote_addons,
            installed_addons,
            addon_details: Default::default(),
            streaming_server,
            player: Default::default(),
        };
        (
            model,
            continue_watching_preview_effects
                .join(discover_effects)
                .join(library_effects)
                .join(continue_watching_effects)
                .join(remote_addons_effects)
                .join(installed_addons_effects)
                .join(streaming_server_effects)
                .join(local_search_effects),
        )
    }
    pub fn get_state(&self, field: &WebModelField) -> JsValue {
        match field {
            WebModelField::Ctx => serialize_ctx(&self.ctx),
            WebModelField::AuthLink => JsValue::from_serde(&self.auth_link).unwrap(),
            WebModelField::DataExport => serialize_data_export(&self.data_export),
            WebModelField::ContinueWatchingPreview => serialize_continue_watching_preview(
                &self.continue_watching_preview,
                &self.ctx.streams,
                &self.ctx.profile.settings,
            ),
            WebModelField::Board => serialize_catalogs_with_extra(&self.board, &self.ctx),
            WebModelField::Discover => {
                serialize_discover(&self.discover, &self.ctx)
            }
            WebModelField::Library => serialize_library(
                &self.library,
                &self.ctx.streams,
                &self.ctx.profile.settings,
                "library".to_owned(),
            ),
            WebModelField::ContinueWatching => serialize_library(
                &self.continue_watching,
                &self.ctx.streams,
                &self.ctx.profile.settings,
                "continuewatching".to_owned(),
            ),
            WebModelField::Search => serialize_catalogs_with_extra(&self.search, &self.ctx),
            WebModelField::LocalSearch => serialize_local_search(&self.local_search),
            WebModelField::MetaDetails => {
                serialize_meta_details(&self.meta_details, &self.ctx)
            }
            WebModelField::RemoteAddons => serialize_remote_addons(&self.remote_addons, &self.ctx),
            WebModelField::InstalledAddons => serialize_installed_addons(&self.installed_addons),
            WebModelField::AddonDetails => JsValue::from_serde(&self.addon_details).unwrap(),
            WebModelField::StreamingServer => serialize_streaming_server(&self.streaming_server),
            WebModelField::Player => {
                serialize_player(&self.player, &self.ctx)
            }
        }
    }
}
