# ATLAS Freedom API

This library is a Rust library which focuses on wrapping the ATLAS Freedom REST API in an easy
to use and idiomatic way. The API is entirely asynchronous, support for a blocking client may
be added sometime in the future, but for now an executor is required for usage, we recommend
[tokio](https://tokio.rs/).

To get started with this library, simply import the crate's prelude, build a client and make a
query.

```rust, no_run
use freedom_api::prelude::*;
use freedom_config::{Config, Test};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the client, grabbing the API keys from environment variables
    let atlas_config = Config::builder()
        .environment(Test)
        .key_from_env()?
        .secret_from_env()?
        .build()?;

    let mut atlas_client = Client::from_config(atlas_config);

    // Query Freedom for a list of all Satellites, printing the names of the satellite which
    // passed deserialization
    atlas_client.get_satellites()
        .collect::<Vec<_>>()
        .await
        .iter()
        .flatten()
        .for_each(|sat| println!("Satellite Name: {: <20}", sat.name));

    Ok(())
}
```

## Api Return Type

### Async Trait

When looking at the return type of the API methods, they may appear daunting. This is mostly
resulting from async lifetimes brought about by useage of [`async_trait`](https://docs.rs/async-trait/latest/async_trait/).
Once the [async trait feature](https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html)
is release in stable rust. This complexity will be alleviated.

### Container

There is however another complexity that exists in the return types of API methods. You will
note that what is returned by a given method call is of type `Self::Container<T>` rather than
simply type `T`. This complexity is required since there are multiple API clients, most notably
the default [`Client`] and the [`CachingClient`](crate::caching_client::CachingClient)
(available via the `caching` feature flag). The caching client is backed by a concurrent caching
system, and in order to avoid unnecessarily cloning all responses from the caching client to the
call site, the cached values are stored as `Arc<T>` so they can be cheaply cloned from the
cache. This complexity will be mostly transparent to the caller, since the container is required
to implement [`Deref<T>`](std::ops::Deref). If however you need to mutate the data after
receiving it, simply clone the value out of its container and mutate the cloned value.

```rust
# use std::sync::Arc;
let arc_value = Arc::new(String::from("Hello "));
let mut mutable_value = (*arc_value).clone();
mutable_value.push_str("World!");
```