FROM alpine:latest
EXPOSE 6666
WORKDIR /root/
COPY target/x86_64-unknown-linux-musl/release/coordination .

CMD ["./coordination", "run"]
