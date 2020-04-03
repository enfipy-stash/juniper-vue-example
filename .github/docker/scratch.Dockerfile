FROM scratch
# COPY /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY app app
ENTRYPOINT ["./app"]
