FROM ubuntu:latest

WORKDIR /app

# ensure rust connections to web works
RUN apt-get update && apt-get install -y openssl ca-certificates

COPY ./mounted/SP2Any-Linux .

RUN chmod +x ./*

ENTRYPOINT ["./SP2Any-Linux"]
