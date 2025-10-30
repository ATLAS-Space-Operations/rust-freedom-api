use freedom_models::bundle::fps_task;
use time::{OffsetDateTime, format_description::well_known::Iso8601};

use super::{Api, Error};

/// Adds additional functionality by exposing bundle endpoints.
///
/// These are primarily used internally by the FPS and Gateway but exist in the public API, and are
/// thus included here. In general, use of these should be avoided for customers
pub trait BundleApi: Api {
    /// Produces a list of [`fps_task::Bundle`]s within the designated window
    ///
    /// See [`get`](Api::get) documentation for more details about the process and return type
    fn get_fps_task_bundle(
        &self,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> impl Future<Output = Result<Self::Container<Vec<fps_task::Bundle>>, Error>> + Send + Sync
    {
        async move {
            let start = start.format(&Iso8601::DEFAULT).map_err(Error::from)?;
            let end = end.format(&Iso8601::DEFAULT).map_err(Error::from)?;

            let mut uri = self.path_to_url("fpstaskbundle/search/findByOverlapping");
            uri.set_query(Some(&format!("start={}&end={}", start, end)));

            self.get_json_map::<Self::Container<Vec<fps_task::Bundle>>>(uri)
                .await
        }
    }
}

impl<T> BundleApi for T where T: Api {}
