FROM rust:latest AS builder

WORKDIR /app

# RUN rustup target add x86_64-unknown-linux-gnu

COPY ./backend ./backend
COPY ./game_state ./game_state

# Use build cache for target directory
RUN --mount=type=cache,target=/app/backend/target,sharing=locked \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  cd ./backend && cargo build --release --bin game --target x86_64-unknown-linux-gnu --features state_traces

RUN --mount=type=cache,target=/app/backend/target,sharing=locked \
  --mount=type=cache,target=/usr/local/cargo/git/db \
  --mount=type=cache,target=/usr/local/cargo/registry/ \
  cd ./backend && cargo build --release --bin bot_client --target x86_64-unknown-linux-gnu 


RUN --mount=type=cache,target=/app/backend/target,sharing=locked \
  cp /app/backend/target/x86_64-unknown-linux-gnu/release/game /app/game 

RUN --mount=type=cache,target=/app/backend/target,sharing=locked \
  cp /app/backend/target/x86_64-unknown-linux-gnu/release/bot_client /app/bot_client

FROM ubuntu:24.04 AS runner

COPY --from=builder /app/game /game
COPY --from=builder /app/bot_client /bot_client