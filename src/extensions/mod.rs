//! # FPS API Extension Traits
//!
//! This modules contains a collection of extension traits for the `freedom_models` to enable a
//! HATEOAS-esque API from within rust.
//!
//! These are implemented as traits to allow the `freedom_models` crate to remain extremely thin,
//! so that when it is ingested by other crates which do not require this functionality, it does
//! not contribute to the dependency graph.

use std::collections::HashMap;

use freedom_models::{utils::Content, Hateoas};

use crate::{api::Value, prelude::Api};
mod account;
mod band;
mod request;
mod satellite;
mod site;
mod task;
mod user;

pub use {
    account::AccountExt,
    band::BandExt,
    request::TaskRequestExt,
    satellite::SatelliteExt,
    site::{SiteConfigurationExt, SiteExt},
    task::TaskExt,
    user::UserExt,
};

fn get_id(reference: &'static str, links: &HashMap<String, url::Url>) -> Result<i32, crate::Error> {
    let url = links
        .get(reference)
        .ok_or(crate::error::Error::MissingUri(reference))?;

    let id_str = url
        .path_segments()
        .ok_or(crate::error::Error::InvalidUri("Missing Path".into()))?
        .last()
        .unwrap();

    id_str.parse().map_err(|_| crate::error::Error::InvalidId)
}

async fn get_item<T, C>(
    reference: &'static str,
    links: &HashMap<String, url::Url>,
    client: &C,
) -> Result<T, crate::Error>
where
    C: Api,
    T: Value,
{
    let uri = links
        .get(reference)
        .ok_or(crate::error::Error::MissingUri(reference))?
        .clone();

    client.get_json_map(uri).await
}

// TODO: There are BOTH "_embedded" and "content" wrapping maps. However the former contains the
// links within the map, the later contains the links on the outside of the map. We need to treat
// these differently.
async fn get_embedded<T, C>(
    reference: &'static str,
    links: &HashMap<String, url::Url>,
    client: &C,
) -> Result<<C as Api>::Container<T>, crate::Error>
where
    C: Api,
    T: Value,
{
    use freedom_models::utils::Embedded;

    let wrapped =
        get_item::<Embedded<<C as Api>::Container<T>>, C>(reference, links, client).await?;

    Ok(wrapped.items)
}

async fn get_content<T, C>(
    reference: &'static str,
    links: &HashMap<String, url::Url>,
    client: &C,
) -> Result<T, crate::Error>
where
    C: Api + Send,
    T: Value + Hateoas,
{
    let wrapped = get_item::<Content<T>, C>(reference, links, client).await?;

    Ok(wrapped.inner)
}
