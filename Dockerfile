ARG BUILD_SHA=dev

FROM node:22-alpine AS frontend
WORKDIR /app
ARG BUILD_SHA
ENV BUILD_SHA=${BUILD_SHA}
COPY package.json package-lock.json tsconfig.json vite.config.ts ./
COPY frontend ./frontend
RUN npm ci && npm run build

FROM rust:1.88-slim-bookworm AS backend
WORKDIR /app
ARG BUILD_SHA
ENV BUILD_SHA=${BUILD_SHA}
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app && useradd --system --gid app --home /app app \
    && mkdir -p /app/data && chown -R app:app /app
WORKDIR /app
COPY --from=backend /app/target/release/rubric-revision-loop /usr/local/bin/rubric-revision-loop
COPY --from=frontend /app/dist ./dist
USER app
ENV PORT=8080 DATABASE_URL=sqlite://data/revision-loop.db?mode=rwc DIST_DIR=dist
EXPOSE 8080
CMD ["rubric-revision-loop"]
