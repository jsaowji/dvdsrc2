# dvdsrc2

Vapoursynth mpeg2 source filter for DVD-Video. 

See https://github.com/Jaded-Encoding-Thaumaturgy/vs-jetpack/tree/main/vssource for usage examples.
It is not ment to be used on its own.

libdvdread: release 7.0.0 has problems with ISO files, either extract to folder or downgrade to 6.1.3
Current master depends on 7.1.0 (libdvdread-git) which has the fix but has no offical release.

currently only windows x86_64, manylinux x86_64 and have wheels to pypi uploaded

CI builds have libdvdcss disabled,
if you you want to use you're distro provided libdvdread you can build the package from git or sdist

# how to install
pip install vapoursynth-dvdsrc2

or from git

pip install git+https://github.com/jsaowji/dvdsrc2.git

or

uv build

# how to build under linux
install [a52dec mpeg2dec libdvdread cargo rust pkg-config] from your package manager
cargo build --release

(on some distros a52dec has no pkg-config files ?)
