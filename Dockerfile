# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-alpine as builder

RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen

RUN npm install -g sass

RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /work
COPY badge/ /work/badge
COPY web-badge/ /work/web-badge
COPY badge_net/ /work/badge_net
COPY badge_draw/ /work/badge_draw

RUN ls -al /work

WORKDIR /work/web-badge
RUN cargo leptos build --release -vv

RUN ls -l /work/web-badge
RUN ls -l /work
RUN find /work/web-badge/target -print

FROM rustlang/rust:nightly-alpine as runner

WORKDIR /app


COPY --from=builder /work/web-badge/target/release/leptos_start /app/
COPY --from=builder /work/web-badge/target/site /app/site
COPY --from=builder /work/web-badge/Cargo.toml /app/

EXPOSE $PORT
ENV LEPTOS_SITE_ROOT=./site

CMD ["/app/leptos_start"]
