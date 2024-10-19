RUSTFLAGS='-Ctarget-feature=+atomics,+bulk-memory,+mutable-globals -Clink-arg=--max-memory=4294967296' wasm-pack build --out-dir www/pkg/threads --target web -- --features threads -Z build-std=panic_abort,std
cp -R ./snippets/* $(ls -d -1 ./www/pkg/threads/snippets/* | sed -n '1p')/src
RUSTUP_TOOLCHAIN=stable wasm-pack build --out-dir www/pkg --target web