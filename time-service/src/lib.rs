use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};
use tracing::instrument;

/// TimeService is our actual service.
#[derive(Default, Debug)]
pub struct TimeService {}

impl TimeService {
    /// Returns the number of seconds since the unix epoch.
    ///
    /// For this stub service, this is obviously extremely simple, and the
    /// "self" is generally useless.
    pub fn what_time_is_it(&self) -> Result<Duration, SystemTimeError> {
        SystemTime::now().duration_since(UNIX_EPOCH)
    }
}

// Service layer testing.
#[cfg(test)]
mod test {
    use crate::TimeService;

    #[test]
    fn what_time_is_it() {
        TimeService::default()
            .what_time_is_it()
            .expect("a non-errored time");
    }
}

/// This is the implementation, of the gRPC Service, for `v1alpha1` of TimeService.
///
/// In a real service codebase, it's reasonable to put this either in a separate
/// file, or even crate.
pub mod grpc {
    use super::instrument;
    use time_bindings::grpc;
    use tonic::{Request, Response, Status};

    pub mod v1alpha1 {
        use super::grpc::v1alpha1::{
            simple_timestamp_service_server::SimpleTimestampService, WhatTimeIsItRequest,
            WhatTimeIsItResponse,
        };
        use super::instrument;
        use super::{Request, Response, Status};

        #[derive(Default, Debug)]
        pub struct TimeServiceGRPCV1Alpha1 {
            inner: super::super::TimeService,
        }

        #[tonic::async_trait]
        impl SimpleTimestampService for TimeServiceGRPCV1Alpha1 {
            // This is the one verb we support.
            #[instrument(level = "info")]
            async fn what_time_is_it(
                &self,
                _request: Request<WhatTimeIsItRequest>,
            ) -> Result<Response<WhatTimeIsItResponse>, Status> {
                let since_the_epoch = self
                    .inner
                    .what_time_is_it()
                    .map_err(|_| Status::internal("the service is time travelling again"))?;

                Ok(Response::new(WhatTimeIsItResponse {
                    seconds_since_epoch: since_the_epoch.as_secs(),
                }))
            }

            // NOTE(canardleteer): We rely upon the default implementation of
            //                     `SomethingUnimplemented`. which returns
            //                     `NOT_IMPLEMENTED`.
        }

        // Resource layer testing.
        #[cfg(test)]
        mod test {
            use super::{SimpleTimestampService, TimeServiceGRPCV1Alpha1, WhatTimeIsItRequest};
            use time_bindings::grpc::v1alpha1::SomethingUnimplementedRequest;
            use tonic::{Request, Status};

            #[tokio::test(flavor = "multi_thread")]
            async fn what_time_is_it() {
                let ts = TimeServiceGRPCV1Alpha1::default();

                let req = Request::new(WhatTimeIsItRequest {});
                let rsp = ts
                    .what_time_is_it(req)
                    .await
                    .expect("a result, not a status");

                assert_ne!(rsp.get_ref().seconds_since_epoch, 0);
            }

            #[tokio::test(flavor = "multi_thread")]
            async fn something_unimplemented() {
                let ts = TimeServiceGRPCV1Alpha1::default();

                let req = Request::new(SomethingUnimplementedRequest {});
                let err = ts.something_unimplemented(req).await.unwrap_err();

                assert_eq!(err.code(), Status::unimplemented("").code());
            }
        }
    }
}
