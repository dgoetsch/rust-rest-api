FROM ekidd/rust-musl-builder as build
ADD Cargo.* /home/rust/src/
ADD src/ /home/rust/src/src/
RUN cargo build --release

FROM scratch

COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/rust-monad /
EXPOSE 3000

CMD ["/rust-monad"]
