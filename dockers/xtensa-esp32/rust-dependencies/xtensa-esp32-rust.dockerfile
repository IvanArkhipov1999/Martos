FROM ubuntu:latest

RUN /bin/bash -c cargo install espup
RUN espup install