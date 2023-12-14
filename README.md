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
cargo new --bin service
cargo new --bin client
cargo new --lib bindings
cargo new --lib common
mkdir -p proto/com/github/canardleteer/grpc_service_rs/v1alpha1
touch proto/com/github/canardleteer/grpc_service_rs/v1alpha1/time.proto
touch Cargo.toml
```

- I made this a workspace, by editing `Cargo.toml`.
- I also added `rust-toolchain.toml`.
- For both `service` and `client`, I added:

```shell
cargo add bindings --path ../bindings --rename time_service_bindings
cargo add common --path ../common --rename time_service_common
```

- And for **all crates**, while writing code, I added `Cargo.toml` entries as
  appropriate.

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

### I added `bindings/build.rs`

...to build the protobuf bindings. I keep these in a separate package than the
rest, just because this is a workspace, and it's reasonable to. The `build.rs`
could live in each one independently.  These can become more interesting as you
start to add things, like derive macros to the messages, or multiple proto
files, but this is boring for this example.

This is generally just code generation shenanigans. YMMV.

### `protoc`

If you don't have `protoc` installed, now would be a good time to install it.

I recommend grabbing an upstream `protoc` from [https://github.com/google/protobuf/](https://github.com/google/protobuf/),
but you can probably find an older one on your system via the package manager.

On Ubuntu, for instance:

```shell
sudo apt install protobuf-compiler
```

## `common`

This is a mock up of generally shared client code. It's nice and tidy, and off
to the side so churn can happen here with automation, and not change anything
close to the business logic.

## The Service & Client Code

- You can review the comments in the [service](service/src/main.rs) &
  [client](client/src/main.rs) implementations.

## Running it

- You'll probably want [grpcurl](https://github.com/fullstorydev/grpcurl) if
  you don't have it already.

In one terminal window:

```shell
cargo run --bin service
```

In another terminal window:

```shell
cargo run --bin client
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

```shell
-> grpcurl -plaintext localhost:50051 describe
com.github.canardleteer.grpc_service_rs.v1alpha1.SimpleTimestampService is a service:
service SimpleTimestampService {
  // Returns the services current timestamp with no additional information.
  rpc WhatTimeIsIt ( .com.github.canardleteer.grpc_service_rs.v1alpha1.WhatTimeIsItRequest ) returns ( .com.github.canardleteer.grpc_service_rs.v1alpha1.WhatTimeIsItResponse );
}
grpc.reflection.v1alpha.ServerReflection is a service:
service ServerReflection {
  // The reflection service is structured as a bidirectional stream, ensuring
  // all related requests go to a single server.
  rpc ServerReflectionInfo ( stream .grpc.reflection.v1alpha.ServerReflectionRequest ) returns ( stream .grpc.reflection.v1alpha.ServerReflectionResponse );
}
```

#### List

```shell
grpcurl -plaintext localhost:50051 list
```

And you'll see what we offer:

```shell
-> grpcurl -plaintext localhost:50051 list
com.github.canardleteer.grpc_service_rs.v1alpha1.SimpleTimestampService
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

## Actions

- We run the following steps in our GitHub Actions for all branches & PRs:
  - `buf lint`
  - `cargo check`
  - `cargo fmt`
  - `cargo clippy`
- For Pull Requests, we perform a:
  - `buf breaking`
- For pushes to "special" branches, we perform a:
  - `buf push`

We can view the schema in the Buf Schema Registry: [https://buf.build/canardleteer/grpc-service-rs](https://buf.build/canardleteer/grpc-service-rs).

## TODO

- Add some thin Docker containers
- Add a `docker-compose.yaml`
  - Add useful proxying in the `docker-compose.yaml`
