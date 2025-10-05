FROM ubuntu:latest
ARG PATH_TO_EXEC
WORKDIR /app

# ensure rust connections to web works
RUN apt-get update && apt-get install -y openssl ca-certificates

COPY ${PATH_TO_EXEC} ./binary

RUN chmod +x ./*

ENTRYPOINT ["./binary"]
