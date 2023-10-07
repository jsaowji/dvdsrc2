
# how to build linux
install a52dec mpeg2dec libdvdread rust 
cargo build 


#msys
install all the stuff dvdread dvdcss for mingw64
make msys2 mingw64 see cargo by modifying msys2 mingw ini remove # before the inherit line
in a mingw64 shell
cargo build --release
