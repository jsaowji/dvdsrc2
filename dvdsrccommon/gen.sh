ARGS=--no-layout-tests

bindgen $ARGS dvdread.h > src/bindings/dvdread_.rs
bindgen $ARGS mpeg2.h > src/bindings/mpeg2_.rs
bindgen $ARGS a52.h > src/bindings/a52_.rs