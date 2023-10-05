ARGS=--no-layout-tests

bindgen $ARGS dvdread.h > src/bindings/dvdread.rs
bindgen $ARGS mpeg2.h > src/bindings/mpeg2.rs
bindgen $ARGS a52.h > src/bindings/a52.rs