# Simple gRPC Service & Client

## Information

This is just a quick shell service, to show the basics of building a gRPC
service and client from the perspective of code infrastructure. If you
want to learn how to do interesting things in gRPC & Rust, I'd recommend
starting with the [Tonic RouteGuide](https://github.com/hyperium/tonic/blob/master/examples/routeguide-tutorial.md).

This isn't anything fancy, and loosely inspired by
[this article](https://www.thorsten-hans.com/grpc-services-in-rust-with-tonic/),
I just wanted to focus more on form than function.

## I made this repository, like this

```shell
cd ~/dev # everyone has their own place

mkdir grpc-service-rs
cd grpc-service-rs
```

### Basic layout

```shell
cargo new --bin time-service-server
cargo new --bin time-service-client
cargo new --lib time-bindings
cargo new --lib time-common
mkdir -p proto/github/canardleteer/grpc_service_rs/v1alpha1
touch proto/github/canardleteer/grpc_service_rs/v1alpha1/time.proto
touch Cargo.toml
```

- I made this a workspace, by editing `Cargo.toml`.
- I also added `rust-toolchain.toml`.
- For both `time-service-server` and `time-service-client`, I added:

```shell
cargo add time-bindings --path ../time-bindings  --rename time_bindings
cargo add time-service --path ../time-service --rename time_service
```

- And for **all crates**, while writing code, I added `Cargo.toml` entries as
  appropriate.
- I later added some Docker "stuff," mostly in the `docker` directory.

You could pin some workspace crate versions here if you want to. I just didn't
for this example.

## `proto`

I had to build out a protobuf declaration.

I lint & format with [buf](https://github.com/bufbuild/buf).

I also created a default `proto/buf.yaml` to scope rules, if needed.

```shell
# This should pass.
buf lint proto

# This should also pass.
buf format proto
```

I don't use `buf` to build Rust bindings, I let `prost` do that.

Normally, I would let `proto` be a relative submodule, but not for this example.

## `bindings`

This is just a crate that builds the proto and manages the client/service
generated code.

### I added `time-bindings/build.rs`

...to build the protobuf bindings. I keep these in a separate package than the
rest, just because this is a workspace, and it's reasonable to. The `build.rs`
could live in each one independently.  These can become more interesting as you
start to add things, like derive macros to the messages, or multiple proto
files, but this is boring for this example.

This is generally just code generation shenanigans. YMMV.

- You can dive deeper into using `buf` for Rust code generation, but it's not
really necessary for this example. If you're interested, you can convert the
`time-bindings/build.rs` file into a `buf.gen.yaml` by using the
[protoc-gen-prost](https://github.com/neoeinstein/protoc-gen-prost) plugin.

- If you want to do something like JSON transcoding (or anything supported by
  [serde](https://serde.rs/)), you can take a look at
  [adding attributes to generated types with tonic_build](https://docs.rs/tonic-build/latest/tonic_build/struct.Builder.html#method.type_attribute). In general, annotations on generated code,
  is fairly useful.

- There's a lot of useful stuff in [tonic_build::Builder](https://docs.rs/tonic-build/latest/tonic_build/struct.Builder.html)
  that's worth learning about (`generate_default_stubs`, for example).

### `protoc`

If you don't have `protoc` installed, now would be a good time to install it.

I recommend grabbing an upstream `protoc` from [https://github.com/google/protobuf/](https://github.com/google/protobuf/),
but you can probably find an older one on your system via the package manager.

On Ubuntu, for instance:

```shell
sudo apt install protobuf-compiler
```

## `time-common`

This is a mock up of generally shared client code. It's nice and tidy, and off
to the side so churn can happen here with automation, and not change anything
close to the business logic.

## The Service & Client Code

- You can review the comments in the [time-service](time-service/src/main.rs) &
  [time-client](time-client/src/main.rs) implementations.

## Running it

- You'll probably want [grpcurl](https://github.com/fullstorydev/grpcurl) if
  you don't have it already.

In one terminal window:

```shell
cargo run --bin time-service-server
```

In another terminal window:

```shell
cargo run --bin time-service-client
```

The output, from your client run, should look something like:

```text
  2023-12-13T18:59:41.800217Z  WARN client: Text to stdout Level set to: Some(LevelFilter::INFO)
    at src/main.rs:84

Response from service was: 1702493981
```

And that's it! Leave the service running, we're going to do some additional introspection.

### Probe the service for Reflection

#### Describe

```shell
grpcurl -plaintext localhost:50051 describe
```

And you should see everything, including the comments in your proto file.

```text
-> grpcurl -plaintext localhost:50051 describe
github.canardleteer.grpc_service_rs.v1alpha1.SimpleTimestampService is a service:
service SimpleTimestampService {
  // Returns the services current timestamp with no additional information.
  rpc WhatTimeIsIt ( .github.canardleteer.grpc_service_rs.v1alpha1.WhatTimeIsItRequest ) returns ( .github.canardleteer.grpc_service_rs.v1alpha1.WhatTimeIsItResponse );
}
grpc.health.v1.Health is a service:
service Health {
  // If the requested service is unknown, the call will fail with status
  // NOT_FOUND.
  rpc Check ( .grpc.health.v1.HealthCheckRequest ) returns ( .grpc.health.v1.HealthCheckResponse );
  // Performs a watch for the serving status of the requested service.
  // The server will immediately send back a message indicating the current
  // serving status.  It will then subsequently send a new message whenever
  // the service's serving status changes.
  //
  // If the requested service is unknown when the call is received, the
  // server will send a message setting the serving status to
  // SERVICE_UNKNOWN but will *not* terminate the call.  If at some
  // future point, the serving status of the service becomes known, the
  // server will send a new message with the service's serving status.
  //
  // If the call terminates with status UNIMPLEMENTED, then clients
  // should assume this method is not supported and should not retry the
  // call.  If the call terminates with any other status (including OK),
  // clients should retry the call with appropriate exponential backoff.
  rpc Watch ( .grpc.health.v1.HealthCheckRequest ) returns ( stream .grpc.health.v1.HealthCheckResponse );
}
grpc.reflection.v1alpha.ServerReflection is a service:
service ServerReflection {
  // The reflection service is structured as a bidirectional stream, ensuring
  // all related requests go to a single server.
  rpc ServerReflectionInfo ( stream .grpc.reflection.v1alpha.ServerReflectionRequest ) returns ( stream .grpc.reflection.v1alpha.ServerReflectionResponse );
}

}
```

#### List

```shell
grpcurl -plaintext localhost:50051 list
```

And you'll see what we offer:

```shell
-> grpcurl -plaintext localhost:50051 list
github.canardleteer.grpc_service_rs.v1alpha1.SimpleTimestampService
grpc.health.v1.Health
grpc.reflection.v1alpha.ServerReflection
```

### Check the service Health Check

If you want to download [grpc-health-probe](https://github.com/grpc-ecosystem/grpc-health-probe),
you can and use that, or you can use `grpcurl`.

#### `grpc_health_probe`

```shell
-> grpc_health_probe -addr 127.0.0.1:50051
status: SERVING
```

#### `grpcurl`

```shell
-> grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check
{
  "status": "SERVING"
}
```

## Docker Container

You can build the container:

```shell
docker build -t time:latest -f docker/Dockerfile .
```

You can test the container:

```shell
docker network create time-network
docker run --rm -it -d \
    --name time_server \
    --net time-network \
    -p 50051:50051 \
    time:latest

docker run --rm -it \
    --net time-network \
    -e USE_CLIENT_BINARY=true \
    time:latest -a time_server

# cleanup
docker rm -f time_server
docker network remove time-network
```

## Actions

- We run the following steps in our GitHub Actions for all branches & PRs:
  - `buf lint`
  - `cargo check`
  - `cargo fmt`
  - `cargo clippy`
  - `cargo test`
- For Pull Requests, we perform a:
  - `buf breaking`
- For pushes to "special" branches, we perform a:
  - `buf push`
  - `docker build`
  - `docker push`

I've omitted versioning for this example, but auto versioning pipelines are
easy. I the past I've had luck with [cargo-smart-release](https://github.com/byron/cargo-smart-release),
but there are likley more and better tools now.

### Output

- We can view the schema in the Buf Schema Registry: [https://buf.build/canardleteer/grpc-service-rs](https://buf.build/canardleteer/grpc-service-rs).
- We can view the docker image on Dockerhub: [https://hub.docker.com/r/canardleteer/grpc-service-rs](https://hub.docker.com/r/canardleteer/grpc-service-rs)

## `docker compose`

Basic `docker compose` with Envoy:

```shell
docker compose up --build

grpcurl -plaintext localhost:10200 describe
grpcurl -plaintext localhost:10200 github.canardleteer.grpc_service_rs.v1alpha1.SimpleTimestampService/WhatTimeIsIt
```

### Advanced Envoy with `docker compose`

**NOTE:** I haven't quite gotten gRPC transcoding working yet.

- There is a mostly commented out configuration in a second Envoy listener in `envoy/envoy.yaml`
- There is a commented out file mapping for `time_service.binpb` in `docker-compose.yaml`

To generate `envoy/time_service.binpb`, you'll need to do the following:

```text
buf build --as-file-descriptor-set -o envoy/time_service.binpb
```
