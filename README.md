# sunspec_gateway

This app will monitor a SunSpec-compliant device and emit metrics from said device to MQTT.

# Usage

This app is built into a docker image which is pushed to the docker hub with both amd64 (normal PC's) and arm64 (raspberry pi 3-onwards).

## Running straight from docker
`docker run -ti -v ./config.yaml:/opt/sunspec_gateway/config.yaml docker.io/petergrace/sunspec-gateway:latest`


## using docker-compose
`docker compose up -d` and then `docker-compose logs` to watch logs.


## Building it yourself
The tool should build easily with a `cargo build` instantiation.  The only gotcha is that I use a sqlite extension for part of the data storage backend and so you will need to manually copy ./tools/stats.so.XXX to ./stats.so for the program to execute.  The XXXX depends on your processor architecture (amd64 for normal PC or arm64 for raspberry pis 3 and onwards)
