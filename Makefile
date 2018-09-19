build: cargo
	find target/wasm32-unknown-emscripten/debug/deps -type f -name "*.wasm" | xargs -I {} cp {} public/simple.wasm
	find target/wasm32-unknown-emscripten/debug/deps -type f ! -name "*.asm.js" -name "*.js" | xargs -I {} cp {} public/simple.js
cargo:
	cargo build --target=wasm32-unknown-emscripten
clean:
	cargo clean
	rm public/simple.wasm
	rm public/simple.js

