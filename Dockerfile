FROM alpine:latest
EXPOSE 6666
WORKDIR /root/
COPY target/x86_64-unknown-linux-musl/release/coordinate .

CMD ["./coordinate", "run"]
