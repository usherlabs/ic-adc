#! Running the orchestrator in a docker file leads to issues with the verity-client
# initial buider image to compile the orchestrator into an executable binary
FROM rust:bookworm AS build
WORKDIR /verityprogram
COPY ./orchestrator ./
RUN cargo build --release


# our final base image
FROM debian:bookworm-slim
# Install pkg-config and libssl-dev for async-tungstenite to use
RUN apt-get update && apt-get -y upgrade && apt-get install -y --no-install-recommends \
  pkg-config \
  libssl-dev \
  ca-certificates \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*
# Copy default fixture folder for default usage
WORKDIR /verityprogram
# copy the build artifact and config from the build stage
COPY --from=build /verityprogram/target/release/orchestrator .
# !NOTE: the env variables from the docker compose file will override the .env file if both are provided
COPY --from=build /verityprogram/identity.pem .
# set the startup command to run your binary


# add tags for data dog
LABEL com.datadoghq.ad.check_names='["orchestrator"]'
LABEL com.datadoghq.ad.init_configs='[{}]'
LABEL com.datadoghq.ad.instances='[{"command": "./orchestrator"}]'
LABEL com.datadoghq.ad.logs='[{"source": "orchestrator", "service": "orchestrator"}]'

CMD ["./orchestrator"]