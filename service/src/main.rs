use clap::Parser;
use time_svc_decl::{
    simple_timestamp_service_server::SimpleTimestampService, WhatTimeIsItRequest,
    WhatTimeIsItResponse,
};

use std::{
    net::{IpAddr, SocketAddr},
    time::{SystemTime, UNIX_EPOCH},
};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, instrument, warn};

use crate::time_svc_decl::simple_timestamp_service_server::SimpleTimestampServiceServer;

use time_service_bindings::{time_svc_decl, TIME_SVC_FILE_DESCRIPTOR_SET};
use time_service_common::setup_logging;

/// This is generally our Command Line Arguments declaration for the service,
/// nothing fancy here.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    // Server Arguments
    #[clap(
        help_heading = "server",
        short = 'a',
        long,
        default_value = "0.0.0.0",
        env = "SERVER_LISTEN_ADDR"
    )]
    listen_interface: IpAddr,

    #[clap(
        help_heading = "server",
        short = 'p',
        long,
        default_value = "50051",
        help_heading = "server",
        env = "SERVER_LISTEN_PORT"
    )]
    listen_port: u16,
}

/// This is our main function, that starts our service binary.
///
/// This could be collapsed into a 3-4 line function without
/// much effort, but I'm leaving it broken out.
#[tokio::main]
#[instrument(level = "info")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse our CLI Args so we can capture any environment/CLI components.
    let args = Cli::parse();
    let listen_addr = SocketAddr::new(args.listen_interface, args.listen_port);

    // Setup logging.
    //
    // In general, this is where we'd wireup all kinds of tooling, but for now,
    // let's just setup a nice logging layer.
    setup_logging();

    // Now, we just spin up what we intend to expose.

    // Build our gRPC reflection service, adding our FileDescriptorSet + the one for Health Check.
    let reflection_svc = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(TIME_SVC_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    // Health checks should be fairly precise, and deeply integrated with the
    // service, but in our case, it's trivial, so we'll add a basic one that
    // always reports SERVING while the process is up.
    let (mut health_reporter, health_svc) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<SimpleTimestampServiceServer<TimeService>>()
        .await;

    // This is our actual service, that we intend to expose.
    let time_svc = TimeService::default();

    // This will hold the process alive in a serve loop until there's reason
    // to end it.
    info!("serving gRPC on: {}...", listen_addr);
    Server::builder()
        .add_service(SimpleTimestampServiceServer::new(time_svc))
        .add_service(health_svc)
        .add_service(reflection_svc)
        .serve(listen_addr)
        .await?;

    Ok(())
}

/// TimeService is our actual service.
#[derive(Default, Debug)]
struct TimeService {}

/// This is the implementation, of our gRPC Service, for TimeService.
#[tonic::async_trait]
impl SimpleTimestampService for TimeService {
    // This is the one verb we support.
    #[instrument(level = "info")]
    async fn what_time_is_it(
        &self,
        _request: Request<WhatTimeIsItRequest>,
    ) -> Result<Response<WhatTimeIsItResponse>, Status> {
        let since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Status::internal("the service is time travelling again"))?;

        Ok(Response::new(WhatTimeIsItResponse {
            seconds_since_epoch: since_the_epoch.as_secs(),
        }))
    }
}
