# Build stage
FROM rust:latest AS builder

WORKDIR /app

RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-pack

# RUN rustup target add x86_64-unknown-linux-musl
# ENV RUSTFLAGS='-Clinker=x86_64-linux-gnu-gcc'
# RUN cd ./backend && cargo build --release --target x86_64-unknown-linux-musl

COPY ./game_state ./game_state
RUN cd ./game_state && wasm-pack build --target bundler --release

RUN cd ../

COPY ./backend ./backend
COPY ./game_state ./game_state
RUN cd ./backend && cargo build --release --target x86_64-unknown-linux-gnu

FROM node:20 as FrontendBuilder

WORKDIR /app

COPY ./package.json ./package.json

COPY ./front/package.json ./front/package.json

COPY ./package-lock.json ./package-lock.json

RUN npm config set strict-ssl false
RUN npm install --force --loglevel verbose

COPY --from=builder /app/game_state/pkg /app/game_state/pkg

RUN npm install

# Define a variable
ARG FRONT_SERVER

# Use the variable
ENV FRONT_SERVER=$FRONT_SERVER

COPY ./front ./front

RUN cd ./front && npm run build

# Final run stage
FROM ubuntu:24.04 as Runner

VOLUME [ "/data" ]

COPY --from=builder /app/backend/target/x86_64-unknown-linux-gnu/release/game game
COPY --from=FrontendBuilder /app/front/dist dist

CMD ["/game"]