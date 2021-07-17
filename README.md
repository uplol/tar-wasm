# tar-wasm

Provides a simple streaming tarball library for environments with Readable/Writable streams (web browser, deno, nodejs, etc) via a WASM module.

See [test.ts](test.ts) for a usage example in Deno.

### Install Deps
```
cargo install wasm-pack
```

### Build
```
wasm-pack build --target web
```

### Run Example
```
deno run --allow-all ./test.ts
tar -xvf ./test.tar
```