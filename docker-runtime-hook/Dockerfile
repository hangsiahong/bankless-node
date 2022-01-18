FROM ubuntu:focal-20211006

RUN apt update && \
 apt install curl -y && \
 apt install unzip -y && \
 apt install git -y && \
 apt install jq -y

WORKDIR bankless-runtime

COPY local-tests/send-runtime/target/release/send_runtime /bankless-runtime/send_runtime
RUN chmod +x /bankless-runtime/send_runtime

COPY docker-runtime-hook/entrypoint.sh /bankless-runtime/entrypoint.sh
RUN chmod +x /bankless-runtime/entrypoint.sh

ENTRYPOINT ["./entrypoint.sh"]