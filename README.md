# json-schema-faker-rs
JSONschema faker not in Rust, but for Rust.

_The project is a prototype and lacks support for custom keywords and refs, use at your
own risk_

## Motivation

Who wants a data generator for JSON schemas in Rust? And who wants to write it in their
free time? Me neither. However, we have a working generator in JavaScript, and this
library aims to fill the gap and deliver the flagship JSON schema data generator
experience to the Rust world.

## Requirements

1. Rust with Cargo
2. [Javy](https://github.com/bytecodealliance/javy)
3. npm

## How it works
1. We use [json-schema-faker](https://github.com/json-schema-faker/json-schema-faker) to
handle the task.
2. The WASI module should follow the "command pattern" as described in [Javy's
README](https://github.com/bytecodealliance/javy?tab=readme-ov-file#invoking-javy-generated-modules-programatically). That's
why there is a wrapper around the library.
3. [Rollup](https://rollupjs.opg) bundles it into an ES module.
4. [Javy](https://github.com/bytecodealliance/javy), mentioned above, compiles it into a
WASI module.
5. [Wasmtime](https://github.com/bytecodealliance/wasmtime) performs its own compilation
and dumps the bytecode into a file at `build.rs` time.
6. The build script embeds Wasmtime's output into the binary, with no runtime dependencies
needed afterward!