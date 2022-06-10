FROM arm64v8/ubuntu:22.04

ARG CARGO_TARGET_DIR
ARG SERVICE_NAME

RUN apt update
RUN apt install -y ca-certificates
RUN mkdir -p /uc/bin
COPY $CARGO_TARGET_DIR/$SERVICE_NAME /uc/bin/$SERVICE_NAME

EXPOSE 8080
CMD /uc/bin/$SERVICE_NAME
