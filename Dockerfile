FROM debian:8
EXPOSE 8080
CMD ["/redact-store"]
COPY target/release/ /
