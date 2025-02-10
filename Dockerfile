
# Stage 2: Build Rust binary
FROM rust:1.78 AS builder
COPY dummy.rs .
COPY Cargo.toml Cargo.lock .
RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml
COPY . .
RUN cargo build --release

# Stage 3: Slim runtime environment
FROM debian:bookworm-slim AS runtime
COPY --from=builder target/release/gym_tracker gym_tracker
CMD ["/gym_tracker", "run"]

