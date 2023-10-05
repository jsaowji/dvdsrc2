FROM ubuntu:22.04 as tarballs_raw
RUN apt update && apt -y upgrade
RUN apt-get install -y wget
RUN apt-get install -y build-essential curl xz-utils
RUN wget -nc https://libmpeg2.sourceforge.io/files/libmpeg2-0.5.1.tar.gz
RUN wget -nc https://codeload.github.com/vapoursynth/vapoursynth/tar.gz/refs/tags/R62
RUN wget -nc http://download.videolan.org/pub/videolan/libdvdread/last/libdvdread-6.1.3.tar.bz2
RUN wget -nc https://git.adelielinux.org/community/a52dec/-/archive/c388f3b6d911c246e0b2a7b2c436c3de2e79c74d/a52dec-c388f3b6d911c246e0b2a7b2c436c3de2e79c74d.tar.gz
RUN wget -nc https://github.com/vapoursynth/vapoursynth/releases/download/R64/VapourSynth64-Portable-R64.7z
RUN wget -nc https://ziglang.org/builds/zig-linux-x86_64-0.12.0-dev.706+62a0fbdae.tar.xz -O zig.tar

FROM tarballs_raw as tarballs
RUN mkdir zig-dl && tar xf zig.tar -C zig-dl
RUN mv /zig-dl/zig-linux-x86_64-0.12.0-dev.706+62a0fbdae/* /zig-dl/
RUN tar -xvf a52dec-c388f3b6d911c246e0b2a7b2c436c3de2e79c74d.tar.gz && mv a52dec-c388f3b6d911c246e0b2a7b2c436c3de2e79c74d a52dec
RUN tar -xvf libdvdread-6.1.3.tar.bz2 && mv libdvdread-6.1.3 libdvdread
RUN tar -xvf libmpeg2-0.5.1.tar.gz && mv libmpeg2-0.5.1 libmpeg2
RUN apt-get install -y p7zip-full
RUN mkdir vportable
WORKDIR vportable
RUN 7z x ../VapourSynth64-Portable-R64.7z


FROM ubuntu:22.04 as zigbuildbase
RUN apt update && apt -y upgrade
RUN apt-get install -y wget
RUN apt-get install -y build-essential curl
RUN apt-get install -y automake autoconf libtool
COPY --from=tarballs --link /zig-dl /zig-dl
ENV PATH="/zig-dl/:${PATH}"
ENV AR="zig ar"
ENV RANLIB="zig ranlib"
ENV CC="zig cc --target=x86_64-windows -D__CRT__NO_INLINE -mno-ms-bitfields"

FROM zigbuildbase as a52dec
COPY --from=tarballs /a52dec /a52dec
WORKDIR /a52dec
RUN pwd
RUN find /a52*
RUN ./bootstrap
RUN ./configure --host="x86_64-windows"
RUN make

FROM zigbuildbase as mpeg2
COPY --from=tarballs /libmpeg2 /libmpeg2
WORKDIR /libmpeg2
RUN ./configure --host="x86_64-windows" --disable-sdl --disable-directx
RUN make

FROM zigbuildbase as dvdread
COPY --from=tarballs /libdvdread /libdvdread
WORKDIR /libdvdread
RUN ./configure --host="x86_64-windows"
RUN make

FROM ubuntu:22.04 as external_libs
RUN apt update && apt -y upgrade
RUN apt-get install -y build-essential curl 
RUN mkdir windows_libs
COPY --from=tarballs /vportable/sdk/lib64 /windows_libs
RUN mv /windows_libs/VSScript.lib /windows_libs/vapoursynth-script.lib
RUN mv /windows_libs/VapourSynth.lib /windows_libs/vapoursynth.lib
COPY --from=mpeg2 /libmpeg2/libmpeg2/.libs/libmpeg2.a . /windows_libs
COPY --from=dvdread /libdvdread/.libs/libdvdread.a . /windows_libs
COPY --from=a52dec /a52dec/liba52/.libs/liba52.a . /windows_libs
COPY --from=tarballs /VapourSynth64-Portable-R64.7z .


FROM ubuntu:22.04 as rustbase
RUN apt update && apt -y upgrade
RUN apt-get install -y build-essential curl 
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add x86_64-pc-windows-gnu
RUN apt install wget
RUN cargo install bindgen-cli
RUN apt-get install -y  libdvdread-dev libmpeg2-4-dev liba52-dev libclang-dev gcc-mingw-w64
COPY --link --from=external_libs /windows_libs /windows_libs
COPY myapp.tar.gz /
RUN tar -xvf myapp.tar.gz
WORKDIR /dvdsrccommon
RUN sh gen.sh
WORKDIR /
RUN cargo build --target x86_64-pc-windows-gnu
RUN cargo build --target x86_64-pc-windows-gnu --release

FROM scratch AS binaries
COPY --from=rustbase /target/x86_64-pc-windows-gnu/release/*.dll /
