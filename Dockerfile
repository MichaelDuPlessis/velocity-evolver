FROM rust
WORKDIR /app

ENTRYPOINT [ "cargo", "run", "--release" ]
