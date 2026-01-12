//! # FPS API Extension Traits
//!
//! This modules contains a collection of extension traits for the `freedom_models` to enable a
//! HATEOAS-esque API from within rust.
//!
//! These are implemented as traits to allow the `freedom_models` crate to remain extremely thin,
//! so that when it is ingested by other crates which do not require this functionality, it does
//! not contribute to the dependency graph.
//!
//! ## Container
//!
//! Unlike the objects fetched directly from the API, objects fetched through a parent are not
//! wrapped in the Container<T> type. This is deliberate as certain objects are nested deeply and
//! muddy the API when wrapping.

use std::collections::HashMap;

use freedom_models::{Hateoas, utils::Content};

use crate::{api::Value, error, prelude::Api};
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
    satellite::{SatelliteConfigurationExt, SatelliteExt},
    site::{SiteConfigurationExt, SiteExt},
    task::TaskExt,
    user::UserExt,
};

fn get_id(reference: &'static str, links: &HashMap<String, url::Url>) -> Result<i32, error::Error> {
    let url = links
        .get(reference)
        .ok_or(error::Error::MissingUri(reference))?;

    let id_str = url
        .path_segments()
        .ok_or(error::Error::InvalidUri("Missing Path".into()))?
        .next_back()
        .ok_or(error::Error::InvalidId)?;

    id_str.parse().map_err(|_| error::Error::InvalidId)
}

async fn get_item<T, C>(
    reference: &'static str,
    links: &HashMap<String, url::Url>,
    client: &C,
) -> Result<T, error::Error>
where
    C: Api,
    T: Value,
{
    let uri = links
        .get(reference)
        .ok_or(error::Error::MissingUri(reference))?
        .clone();

    client.get_json_map(uri).await
}

async fn get_embedded<T, C>(
    reference: &'static str,
    links: &HashMap<String, url::Url>,
    client: &C,
) -> Result<T, error::Error>
where
    C: Api,
    T: Value,
{
    use freedom_models::utils::Embedded;

    let wrapped = get_item::<Embedded<T>, C>(reference, links, client).await?;

    Ok(wrapped.items)
}

async fn get_content<T, C>(
    reference: &'static str,
    links: &HashMap<String, url::Url>,
    client: &C,
) -> Result<T, error::Error>
where
    C: Api + Send,
    T: Value + Hateoas,
{
    let wrapped = get_item::<Content<T>, C>(reference, links, client).await?;

    Ok(wrapped.inner)
}
