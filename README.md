# json-schema-faker-rs

JSON Schema faker not written in Rust, but made for Rust.

# Why?

There was no way to generate test data in Rust that complied with a provided JSON Schema,
and no time to build one. Luckily, everything we need already exists: a JS library, a WASM
runtime, and a JS-to-WASM compiler. This crate simply packs them together so you can plug
and play.

## Requirements

1. Rust with Cargo
2. [Javy](https://github.com/bytecodealliance/javy) installed
3. npm installed

## How it works
1. We use [json-schema-faker](https://github.com/json-schema-faker/json-schema-faker) to
handle the task.
2. The WASI module should follow the "command pattern" as described in [Javy's
README](https://github.com/bytecodealliance/javy?tab=readme-ov-file#invoking-javy-generated-modules-programatically). That's
why there is a wrapper around the library.
3. [Rollup](https://rollupjs.org) bundles it into an ES module.
4. [Javy](https://github.com/bytecodealliance/javy), mentioned above, compiles it into a
WASI module.
5. [Wasmtime](https://github.com/bytecodealliance/wasmtime) performs its own compilation
and dumps the bytecode into a file at `build.rs` time.
6. The build script embeds Wasmtime's output into the binary, with no runtime dependencies
needed afterward!
