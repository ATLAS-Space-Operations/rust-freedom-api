# ATLAS Freedom API

[![Crates.io](https://img.shields.io/crates/v/freedom-api.svg)](https://crates.io/crates/freedom-api)
[![Documentation](https://docs.rs/freedom-api/badge.svg)](https://docs.rs/freedom-api/)

This library is a Rust library which focuses on wrapping the ATLAS Freedom REST
API in an easy to use and idiomatic way. The API is entirely asynchronous,
support for a blocking client may be added sometime in the future, but for now
an executor is required for usage, we recommend [tokio](https://tokio.rs/).

## Installation 

To incorporate the Freedom API into an existing cargo project simply invoke the
following from the project's root directory:

```console
$ cargo add --git https://github.com/ATLAS-Space-Operations/rust-freedom-api
```

## Getting Started

Once added, simply import the crate's prelude, build a client and make a
query:

```rust, no_run
use freedom_api::prelude::*;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the client, grabbing the API keys from environment variables
    let atlas_config = Config::builder()
        .environment(Test)
        .key_from_env()?    // Sources the key from ATLAS_KEY
        .secret_from_env()? // Sources the secret from ATLAS_SECRET
        .build()?;

    let atlas_client = Client::from_config(atlas_config);

    // Query Freedom for a list of all Satellites, printing the names of the 
    // satellite which passed deserialization
    atlas_client.get_satellites()
        .collect::<Vec<_>>()
        .await
        .iter()
        .flatten()
        .for_each(|sat| println!("Satellite Name: {: <20}", sat.name));

    Ok(())
}
```

### Creating Resources

In addition to fetching resources, the API can also be used to create resources
for example a task request can be constructed with the following:

```rust, no_run
use std::time::Duration;

use freedom_api::prelude::*;
use time::OffsetDateTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let atlas_client = Client::from_env()?;

    let response = atlas_client.new_task_request()
        .test_task("my_test_file.bin")
        .target_time_utc(OffsetDateTime::now_utc() + Duration::from_secs(15 * 60))
        .task_duration(120)
        .satellite_id(1)
        .site_id(2)
        .site_configuration_id(3)
        .band_ids([4, 5, 6])
        .send()
        .await?;

    Ok(())
}
```

## Documentation

The freedom API has a significant amount of documentation to get users up and 
running quickly. To build the docs, simply run the following from the root of 
this repository, once cloned. 

```console
$ cargo doc --no-deps --open
```

## Chaining API Returns

Many of the data types exposed in this library can be navigated to through other
resources, for instance a task request object holds links to the site object the
task was scheduled at.

Rather than making a call to fetch the request, then parse the site ID, then
request the site from the ID, you can instead fetch the site directly from the
return of the request call:

```rust, no_run
use freedom_api::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let atlas_config = Config::from_env()?;
    let atlas_client = Client::from_config(atlas_config);

    let site_from_request: Site = atlas_client
        .get_request_by_id(42)
        .await?
        .get_site(&atlas_client)
        .await?;


    Ok(())
}
```

## API Return Type

### Container

You will note that what is returned by the `get_` methods of the API is of type
`Self::Container<T>` rather than simply type `T`. This complexity is required
since there are multiple API clients, most notably the default [`Client`] and
the `CachingClient` (available via the `caching` feature flag). The caching
client is backed by a concurrent caching system, and in order to avoid
unnecessarily cloning all responses from the caching client to the call site,
the cached values are stored as `Arc<T>` so they can be cheaply cloned from the
cache. This complexity will be mostly transparent to the caller, since the
container is required to implement [`Deref<T>`](std::ops::Deref).

If however you need to mutate the data after receiving it, call the
`Container::into_inner` method on the returned type to get an owned
version of the wrapped type.

```rust, ignore
let mut request = atlas_client
    .get_request_by_id(42)
    .await?
    .into_inner();
```
