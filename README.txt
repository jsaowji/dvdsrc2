# dvdsrc2

See https://github.com/Jaded-Encoding-Thaumaturgy/vs-jetpack/tree/main/vssource for usage examples. It is not ment to be used on its own.

libdvdread: release 7.0.0 has problems with ISO files, either extract to folder or downgrade to 6.1.3
Current master depends on 7.1.0 (libdvdread-git) which has the fix but has no offical release.

CI builds are probably broken because of this


# how to build under linux
install [a52dec mpeg2dec libdvdread rust pkg-config] from your package manager
cargo build --release

# how to build the new shiny vapoursynth plugin wheel
uv build