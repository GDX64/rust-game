# Build stage
FROM rust:latest AS builder


WORKDIR /app

COPY ./backend ./backend
COPY ./game_state ./game_state

# RUN apt update && apt install -y musl-tools musl-dev
# RUN apt-get install -y build-essential
# RUN yes | apt install gcc-x86-64-linux-gnu

RUN rustup target add x86_64-unknown-linux-musl

ENV RUSTFLAGS='-Clinker=x86_64-linux-gnu-gcc'

RUN cd ./backend && cargo build --release --target x86_64-unknown-linux-musl

FROM rust:latest as WasmBuilder

WORKDIR /app

RUN cargo install wasm-pack
RUN rustup target add wasm32-unknown-unknown

COPY ./game_state ./game_state
RUN cd ./game_state && npm run build

FROM node:22 as FrontendBuilder

WORKDIR /app

COPY ./package.json ./package.json

RUN npm install

COPY ./front ./front

COPY --from=WasmBuilder /app/game_state /app/game_state

RUN npm install

# Define a variable
ARG FRONT_SERVER

# Use the variable
ENV FRONT_SERVER=$FRONT_SERVER

RUN cd ./front && npm run build

# Final run stage
FROM scratch AS runner

COPY --from=builder /app/backend/target/x86_64-unknown-linux-musl/release/game game
COPY --from=FrontendBuilder /app/front/dist dist/game
COPY --from=FrontendBuilder /app/game_state/dist dist/editor

CMD ["/game"]
# CMD ["ls"]