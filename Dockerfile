# Base image for Rust Engine
FROM rust:1.80-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Base image for Python NLP Server
FROM python:3.11-slim as python-env
WORKDIR /app
COPY subconscious/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
RUN python -m spacy download en_core_web_sm

# Final Runtime Image
FROM debian:bookworm-slim
WORKDIR /app

# Install Python for the final image
RUN apt-get update && apt-get install -y python3 python3-pip && rm -rf /var/lib/apt/lists/*

# Copy Rust Engine
COPY --from=builder /app/target/release/axiom_engine /app/axiom_engine

# Copy Python Environment and Scripts
COPY --from=python-env /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY subconscious /app/subconscious
COPY dashboard /app/dashboard

# Expose Dashboard and NLP API Ports
EXPOSE 8080 8081

# Startup script to run both Engine and NLP Server
RUN echo '#!/bin/bash\n\
python3 /app/subconscious/nlp_server.py &\n\
/app/axiom_engine\n\
' > /app/start.sh
RUN chmod +x /app/start.sh

CMD ["/app/start.sh"]
