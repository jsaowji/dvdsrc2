
# how to build linux
install a52dec mpeg2dec libdvdread rust 
cargo build 


#msys
install all the stuff: (rust dvdread dvdcss libmpeg2 a52dec) for mingw64

in a mingw64 shell
cargo build --release

#Alternativly if you want to use your windows rust install, change the default target to mingw using rustup and make msys2 mingw64 see cargo by modifying msys2 mingw ini remove # before the inherit line
