# syntax=docker/dockerfile:1

# =============================================================================
# Stage 1: Frontend Build
# =============================================================================
FROM node:24-trixie-slim AS frontend-builder

WORKDIR /app/frontend

# Install dependencies
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci

# Copy source and build
COPY frontend/ ./
RUN npm run build

# =============================================================================
# Stage 2: Cargo Chef - Prepare dependencies recipe
# =============================================================================
FROM rust:1.93-slim-trixie AS chef

RUN cargo install cargo-chef
WORKDIR /app

# =============================================================================
# Stage 3: Prepare recipe
# =============================================================================
FROM chef AS planner

COPY backend/ ./
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 4: Build dependencies (cached layer)
# =============================================================================
FROM chef AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cook dependencies (this layer is cached unless dependencies change)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY backend/ ./
RUN cargo build --release --bin tsukota-server

# =============================================================================
# Stage 5: Final Runtime Image
# =============================================================================
FROM debian:trixie-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/tsukota-server /app/tsukota-server

# Copy frontend build output
COPY --from=frontend-builder /app/frontend/dist /app/public

# Set environment variables
# ENV GOOGLE_APPLICATION_CREDENTIALS # optional
ENV PORT=3000
# ENV PROJECT_ID # optional
ENV PUBLIC_DIR=/app/public
ENV SERVICE_ACCOUNT_EMAIL=your-service-account-email@example.com

# Expose port
EXPOSE 3000

# Run the server
CMD ["/app/tsukota-server"]
