# dvdsrc2

See https://github.com/Jaded-Encoding-Thaumaturgy/vs-jetpack/tree/main/vssource for usage examples. It is not ment to be used on its own.

libdvdread: release 7.0.0 has problems with ISO files, either extract to folder or downgrade to 6.1.3
For now vendored and locked as git submodule libdvdread with patch


# how to build under linux
install [a52dec mpeg2dec libdvdread rust] from your package manager
cargo build --release

# msys
see the github action for reference

install all the dependencies: (rust dvdread dvdcss libmpeg2 a52dec) for ucrt64

in a ucrt64 shell type:
cargo build --release
