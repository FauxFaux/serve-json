FROM clux/muslrust AS build
ADD . .
RUN cargo build --release

FROM alpine:3
COPY --from=build /volume/target/x86_64-unknown-linux-musl/release/serve-json /bin/serve-json
