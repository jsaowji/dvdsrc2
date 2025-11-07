# dvdsrc2
libdvdread: release 7.0.0 has problems with ISO files, either extract to folder or downgrade to 6.1.3

See https://github.com/Jaded-Encoding-Thaumaturgy/vs-jetpack/tree/main/vssource for usage examples. It is not ment to be used on its own.

# how to build under linux
install [a52dec mpeg2dec libdvdread rust] from your package manager
cargo build --release

# msys
see the github action for reference

install all the dependencies: (rust dvdread dvdcss libmpeg2 a52dec) for mingw64

in a mingw64 shell type:
cargo build --release

#Alternativly if you want to use your windows rust install, change the default target to mingw using rustup and make msys2 mingw64 see cargo by modifying msys2 mingw ini remove # before the inherit line
