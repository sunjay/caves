# Special Docker image for cross compiling building Rust on GameShell

FROM japaric/armv7-unknown-linux-gnueabihf:latest

# Build SDL2 for 32-bit ARMv7
ADD https://www.libsdl.org/release/SDL2-2.0.8.tar.gz /tmp/sdl2.tar.gz
RUN cd /tmp && \
    tar -pxzf sdl2.tar.gz && \
    cd SDL2-2.0.8 && \
    mkdir build && \
    cd build && \
    #export SDL2_SRC=/tmp/SDL2-2.0.8/src && \
    export TARGETMACH=arm-none-linux-gnueabi && \
    export CC=arm-linux-gnueabihf-gcc && \
    export CXX=arm-linux-gnueabihf-g++ && \
    ../configure --host=$TARGETMACH --disable-pulseaudio && \
    make -j 8 && \
    make -j 8 install && \
    echo "SDL2 installed successfully."

# Build SDL2_image for 32-bit ARMv7
ADD https://www.libsdl.org/projects/SDL_image/release/SDL2_image-2.0.3.tar.gz /tmp/sdl2_image.tar.gz
RUN cd /tmp && \
    tar -pxzf sdl2_image.tar.gz && \
    cd SDL2_image-2.0.3 && \
    mkdir build && \
    cd build && \
    export TARGETMACH=arm-none-linux-gnueabi && \
    export CC=arm-linux-gnueabihf-gcc && \
    export CXX=arm-linux-gnueabihf-g++ && \
    ../configure --host=$TARGETMACH && \
    make -j 8 && \
    make -j 8 install && \
    echo "SDL2_image installed successfully."

RUN apt-get install -y libclang-dev

ENV RUSTFLAGS="-L /usr/local/lib"
