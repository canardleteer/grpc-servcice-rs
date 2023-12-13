use clap::Parser;
use time_svc_decl::WhatTimeIsItRequest;
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, Layer, Registry};

use std::net::IpAddr;
use tonic::Request;
use tracing::{instrument, warn};

use crate::time_svc_decl::simple_timestamp_service_client::SimpleTimestampServiceClient;

pub mod time_svc_decl {
    tonic::include_proto!("com.github.canardleteer.grpc_service_rs.v1alpha1");
}

pub const TIME_SVC_FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("_descriptor");

/// This is generally our Command Line Arguments declaration for the client,
/// nothing fancy here.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    // Server Arguments
    #[clap(
        help_heading = "client",
        short = 'a',
        long,
        default_value = "127.0.0.1",
        env = "SERVER_ADDR"
    )]
    service_addr: IpAddr,

    #[clap(
        help_heading = "server",
        short = 'p',
        long,
        default_value = "50051",
        help_heading = "client",
        env = "SERVER_PORT"
    )]
    service_port: u16,
}

/// All this application does, is call the service exactly once.
#[tokio::main]
#[instrument(level = "info")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse our CLI Args
    let args = Cli::parse();

    // Setup logging. See the notes in the service for more information.
    setup_logging();

    // Build a client.
    let mut client = SimpleTimestampServiceClient::connect(format!(
        "http://{}:{}",
        args.service_addr, args.service_port
    ))
    .await?;

    // Query
    //
    // NOTE: We can add intercepting layers here, we just don't in this example.
    let rsp = client
        .what_time_is_it(Request::new(WhatTimeIsItRequest {}))
        .await?;

    // Print the response.
    println!(
        "Response from service was: {}",
        rsp.get_ref().seconds_since_epoch
    );

    Ok(())
}

/// In general, this should lead to a more common definition, that is uniform for
/// your services fleet, wiring up to your observability stack as
/// appropriate.
///
/// This is somewhat overkill for this example, but get's things in place
/// for the layered approach for tracing.
fn setup_logging() {
    // Filter our emissions, based on environment.
    let text_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let text_filter_level = text_filter.max_level_hint();

    // We only intend to ship logs via stdout, in this example.
    let stdout_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(text_filter);

    // Make a telemetry Subscriber, from the overall Tracing system.
    let subscriber = Registry::default().with(stdout_layer);

    // And set this Subscriber, as the global defaul for this application.
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => {
            warn!("Text to stdout Level set to: {:?}", text_filter_level);
        }
        Err(e) => {
            panic!("Unable to setup logging, failing: {}", e)
        }
    }
}
