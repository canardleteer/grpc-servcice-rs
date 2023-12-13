use clap::Parser;
use time_svc_decl::{
    simple_timestamp_service_server::SimpleTimestampService, WhatTimeIsItRequest,
    WhatTimeIsItResponse,
};
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, Layer, Registry};

use std::{
    net::{IpAddr, SocketAddr},
    time::{SystemTime, UNIX_EPOCH},
};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, instrument, warn};

use crate::time_svc_decl::simple_timestamp_service_server::SimpleTimestampServiceServer;

pub mod time_svc_decl {
    tonic::include_proto!("com.github.canardleteer.grpc_service_rs.v1alpha1");
}

pub const TIME_SVC_FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("_descriptor");

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

#[tokio::main]
#[instrument(level = "info")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse our CLI Args
    let args = Cli::parse();
    let listen_addr = SocketAddr::new(args.listen_interface, args.listen_port);

    // Setup logging.
    setup_logging();

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

    let time_svc = TimeService::default();

    info!("serving gRPC on: {}...", listen_addr);
    Server::builder()
        .add_service(SimpleTimestampServiceServer::new(time_svc))
        .add_service(health_svc)
        .add_service(reflection_svc)
        .serve(listen_addr)
        .await?;

    Ok(())
}

#[derive(Default, Debug)]
struct TimeService {}

#[tonic::async_trait]
impl SimpleTimestampService for TimeService {
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

/// In general, this should lead to a more common definition, uniform for
/// your services fleet, wiring up to your observability stack as
/// appropriate.
///
/// This is somewhat overkill for this example, but get's things in place
/// for the layered approach for tracing.
fn setup_logging() {
    let text_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let text_filter_level = text_filter.max_level_hint();

    let stdout_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(text_filter);

    let subscriber = Registry::default().with(stdout_layer);

    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => {
            warn!("Text to stdout Level set to: {:?}", text_filter_level);
        }
        Err(e) => {
            panic!("Unable to setup logging, failing: {}", e)
        }
    }
}
