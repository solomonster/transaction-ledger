FROM rust:1.89.0
WORKDIR /app
COPY . .
RUN cargo build --release
CMD ["./target/release/myapp"]
