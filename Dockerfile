# Copy source for the panning and building steps
FROM lukemathwalker/cargo-chef:latest-rust-1-bookworm AS chef

WORKDIR /pgpaste
COPY Cargo.toml Cargo.lock ./
COPY ./pgpaste-api-types ./pgpaste-api-types
COPY ./pgpaste-cli ./pgpaste-cli
COPY ./pgpaste-server ./pgpaste-server

# Prepare workspace dependencies with `cargo chef`
FROM chef AS planner
RUN cargo chef prepare --recipe-path recipe.json

# Creating the `pgpaste-server` binary
FROM chef AS builder

# Install link-time C dependencies
RUN apt update -y && apt install -y nettle-dev libssl-dev clang llvm pkg-config

# Cache dependencies and build the project
COPY --from=planner /pgpaste/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
RUN cargo build --release --bin pgpaste-server

# Prepare runtime image
FROM debian:bookworm as runtime
LABEL org.opencontainers.image.source https://github.com/MrNossiom/pgpaste-server

# Install runtime C dependencies
RUN apt update -y && apt install -y libpq5 libnettle8 && rm -rf /var/lib/apt/lists/*

WORKDIR /pgpaste-server
COPY --from=builder /pgpaste/target/release/pgpaste-server ./pgpaste-server

EXPOSE 3000
CMD ["./pgpaste-server"]
