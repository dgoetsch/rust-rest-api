#!/bin/bash

# docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder cargo build --release
 docker build . -t hello-server
 docker run -it -p 3000:3000 hello-server