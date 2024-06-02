# Build stage
FROM rust:bookworm AS builder


WORKDIR /app

COPY ./backend ./backend
COPY ./game_state ./game_state

RUN rustup target add x86_64-unknown-linux-musl
RUN cd ./backend && cargo build --release --target x86_64-unknown-linux-musl

FROM rust:bookworm as WasmBuilder

WORKDIR /app

RUN cargo install wasm-pack
RUN rustup target add wasm32-unknown-unknown

COPY ./game_state ./game_state
RUN cd ./game_state && wasm-pack build

FROM node:22 as FrontendBuilder

WORKDIR /app

COPY ./package.json ./package.json

RUN npm install

COPY ./front ./front
COPY ./game_state ./game_state
COPY --from=WasmBuilder /app/game_state/pkg /app/game_state/pkg

RUN npm install

RUN cd ./front && npm run build

# Final run stage
FROM scratch AS runner

COPY --from=builder /app/backend/target/x86_64-unknown-linux-musl/release/game game
COPY --from=FrontendBuilder /app/front/dist dist

CMD ["/game"]
# CMD ["ls"]