# dvdsrc2

Vapoursynth mpeg2 source filter for DVD-Video.

See https://github.com/Jaded-Encoding-Thaumaturgy/vs-jetpack/tree/main/vssource for usage examples.
It is not ment to be used on its own.

libdvdread: release 7.0.0 has problems with ISO files,
Current master depends on 7.1.0 (libdvdread-git) which has the fix but has no offical release yet.

macos and musllinux builds are provided with best effort only

CI builds have libdvdcss disabled,
if you you want to use you're distro provided libdvdread you can build the package from git or sdist

# how to install
pip install vapoursynth-dvdsrc2

or from git

pip install git+https://github.com/jsaowji/dvdsrc2.git

or

uv build
and install the wheel


# how to build under linux
install [a52dec mpeg2dec libdvdread cargo rust pkg-config] from your package manager
cargo build --release

WARNING: on some distros a52dec does not ship with a pkg-config file
