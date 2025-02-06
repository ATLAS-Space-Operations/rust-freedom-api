# ATLAS Freedom API

[![Crates.io](https://img.shields.io/crates/v/freedom-api.svg)](https://crates.io/crates/freedom-api)
[![Documentation](https://docs.rs/freedom-api/badge.svg)](https://docs.rs/freedom-api/)

This library is a Rust library which focuses on wrapping the ATLAS Freedom REST
API in an easy to use and idiomatic way. The API is entirely asynchronous,
support for a blocking client may be added sometime in the future, but for now
an executor is required for usage. We recommend [tokio](https://tokio.rs/), as
it is already a dependency of the asynchronous http client used.

## Installation 

To incorporate the Freedom API into an existing cargo project simply invoke the
following from the project's root directory:

```console
$ cargo add freedom-api
```

## Documentation

The freedom API has a significant amount of documentation to get users up and 
running quickly. 

The latest docs are available
[here](https://docs.rs/freedom-api/latest/freedom_api/), via docs.rs. If you
would like to build them locally, you may also clone the repo and run the
following from the top-level:

```console
$ cargo doc --no-deps --open
```

## Usage

Once added, simply import the crate's prelude, build a client and make a
query:

```rust, no_run
use freedom_api::prelude::*;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the client, grabbing the API keys from environment variables
    let config = Config::builder()
        .environment(Test)  // Either Test or Prod
        .key_from_env()?    // Sources the key from ATLAS_KEY
        .secret_from_env()? // Sources the secret from ATLAS_SECRET
        .build()?;

    let client = Client::from_config(config);

    // Query Freedom for a list of all Satellites, printing the names of the 
    // satellite which passed deserialization
    client.get_satellites()
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
    let client = Client::from_env()?;

    let response = client.new_task_request()
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

## Chaining API Returns

Many of the data types exposed in this library can be navigated to through other
resources. For instance, a task request object holds links to the site object
the task was scheduled at.

Rather than making a call to fetch the request, then parse the site ID, then
request the site from the ID, you can instead fetch the site directly from the
return of the task request call:

```rust, no_run
// The prelude includes extensions traits to make expose this functionality
use freedom_api::prelude::*; 

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    let site_from_request: Site = client
        .get_request_by_id(42)
        .await?
        .get_site(&client)
        .await?;

    Ok(())
}
```

## API Return Type

### Container

You will note that what is returned by the `get_` methods of the API is of type
`Self::Container<T>` rather than simply type `T`. More information on why it
exists is available with the documentation for the
[Container](https://docs.rs/freedom-api/latest/freedom_api/trait.Container.html)
trait.

However, since `Container<T>` must implement `Deref<T>`, the return types can be
used in most cases just like a `T`.
