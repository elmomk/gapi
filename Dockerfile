FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
RUN groupadd -g 1000 garmin && useradd -u 1000 -g garmin garmin
WORKDIR /app
COPY target/release/garmin_api .
VOLUME /app/data
USER garmin
EXPOSE 3000
CMD ["./garmin_api"]
