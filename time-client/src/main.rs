use clap::Parser;
use time_service_bindings::grpc::v1alpha1::WhatTimeIsItRequest;

use tonic::Request;
use tracing::{instrument, warn};

use time_service_bindings::grpc::v1alpha1::simple_timestamp_service_client::SimpleTimestampServiceClient;

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
        env = "TARGET_SERVER_ADDR"
    )]
    service_addr: String,

    #[clap(
        help_heading = "server",
        short = 'p',
        long,
        default_value = "50051",
        help_heading = "client",
        env = "TARGET_SERVER_PORT"
    )]
    service_port: u16,
}

/// All this application does, is call the service exactly once.
#[tokio::main]
#[instrument(level = "info")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse our CLI Args
    let args = Cli::parse();

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
