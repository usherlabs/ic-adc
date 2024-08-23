# Use the base image from Usher Labs
FROM ghcr.io/usherlabs/verity:latest

# Set the working directory
WORKDIR /app

# Install certbot
RUN apt-get update && apt-get install -y certbot

# Copy the configuration file into the container
COPY ./config/prover.yaml /app/.verity/config/prover.yaml

# !NOT NEEDED FOR RUNNING PROVER
# Generate the TLS certificate and signing key
# RUN verity notary generate-certificate --domain example.com --email admin@example.com --config /app/config/notary.yaml && \
#     verity notary generate-signing-key --config /app/config/notary.yaml

# Expose the port that the Prover server will run on
EXPOSE 8080

# Command to start the Prover server with the right config file
CMD ["verity", "prover", "start", "--config", "/app/.verity/config/prover.yaml"]