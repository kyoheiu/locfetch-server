FROM rust:1.63-buster as builder
WORKDIR /home/rust/
COPY . .
RUN cargo build --release

FROM archlinux:latest
WORKDIR /loc-server
RUN pacman -Syu --noconfirm
RUN pacman -S git --noconfirm
COPY --from=builder /home/rust/target/release/loc-server . 
ENV PORT 8080
ENV RUST_LOG info
EXPOSE 8080
ENTRYPOINT [ "./loc-server" ]
