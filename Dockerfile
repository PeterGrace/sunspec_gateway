FROM docker.io/ubuntu:20.04
ARG TARGETARCH

RUN mkdir -p /opt/sunspec_gateway 
WORKDIR /opt/sunspec_gateway
COPY ./tools/target_arch.sh /opt/sunspec_gateway
COPY models/ /opt/sunspec_gateway/models/
COPY ui/ /opt/sunspec_gateway/ui/
RUN --mount=type=bind,target=/context \
 cp /context/target/$(/opt/sunspec_gateway/target_arch.sh)/release/sunspec_gateway /opt/sunspec_gateway/sunspec_gateway \
 && cp /context/tools/stats.so.${TARGETARCH} /opt/sunspec_gateway/stats.so
CMD ["/opt/sunspec_gateway/sunspec_gateway"]
EXPOSE 8443
