
# how to build linux
install a52dec mpeg2dec libdvdread rust 
exec dvdsrccommon/gen.sh for bindings
cargo build 


#build on windows
git archive --format=tar.gz -o myapp.tar.gz  main
docker build . -o winbuild
