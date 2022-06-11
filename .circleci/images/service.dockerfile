FROM ubuntu:22.04

ARG SERVICE_NAME

RUN apt update
RUN apt install -y ca-certificates
RUN mkdir -p /uc/bin
COPY $SERVICE_NAME /uc/bin/$SERVICE_NAME

EXPOSE 8080
CMD /uc/bin/$SERVICE_NAME
