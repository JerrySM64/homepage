.PHONY: build

build:
	rm -rf pkg
	cd wasm-module && wasm-pack build --target web
	mv wasm-module/pkg ./pkg
	cd wasm-module && cargo clean
