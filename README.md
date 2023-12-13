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
mkdir -p proto/com/github/canardleteer/grpc_service_rs/v1alpha1
touch proto/com/github/canardleteer/grpc_service_rs/v1alpha1/time.proto

for PRJ in client service; do
  pushd ${PRJ}
    # Add our async runtime.
    cargo add tokio@1 --features=macros,rt-multi-thread;
    
    # Add our CLI arg parser
    cargo add clap@4 --features=derive,env,cargo

    # Add a "bunch of service stuff"
    cargo add tonic prost prost-types tonic-reflection tonic-health env_logger@0.10 tracing@0.1

    # Add the Tracing Layer components + environment features.
    cargo add tracing-subscriber --features=env-filter

    # Add tonic-build to help build the grpc service.
    cargo add tonic-build --build
  popd
done

# Since we're in a larger git repository:
rm -rf {client,service}/.git
```

- I also added `rust-toolchain.toml` files to both `service` & `client`.

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

I don't use `buf` to build Rust, I let `prost` do that.

Normally, I would let `proto` be a relative submodule, but not for this example.

## I added `{client,service}/build.rs`

I had to add a:

- `service/build.rs`
- `client/build.rs`

...to build the protobuf bindings. These can become more interesting as you start
to add things, like derive macros to the messages, or multiple proto files, but
this is boring for this example.

This is generally just codegeneration shenanigans, that could be replaced with
a package/crate if needed.


## The Service & Client Code

- You can review the comments in the [service](service/src/main.rs) &
  [client](client/src/main.rs) implementations.
- There's more scaffolding in there then I'd normally "copy around," but it's
  useful for these examples.

## `protoc`

If you don't have `protoc` installed, now would be a good time to install it.

I recommend grabbing an upstream `protoc` from [https://github.com/google/protobuf/](https://github.com/google/protobuf/),
but you can probably find an older one on your system via the package manager.

On Ubuntu, for instance:

```shell
sudo apt install protobuf-compiler
```

## Running it

- You'll probably want [grpcurl](https://github.com/fullstorydev/grpcurl) if
  you don't have it already.

In one terminal window:

```shell
cd service
cargo run
```

In another terminal window:

```shell
cd client
cargo run
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

## TODO

- Add some thin Docker containers
- Add a `docker-compose.yaml`
  - Add useful proxying in the `docker-compose.yaml`
- Overall, this may work better as a Cargo Workspace.
  - Sharing a crate on top of a shared proto directory.
  - Remove some of the uniform logging initalizers.
