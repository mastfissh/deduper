FROM rustembedded/cross:arm-unknown-linux-gnueabihf

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install libgtk-3-dev build-essential -y