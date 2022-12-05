####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=feature_flags_service
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /usr/src/feature_flags_service

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /usr/src/feature_flags_service

# Copy our build
COPY --from=builder /usr/src/feature_flags_service/target/x86_64-unknown-linux-musl/release/feature_flags_service ./

# Use an unprivileged user.
USER feature_flags_service:feature_flags_service

EXPOSE 8080

CMD ["/usr/src/feature_flags_service/feature_flags_service"]