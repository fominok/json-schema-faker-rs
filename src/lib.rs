use wasi_common::pipe::{ReadPipe, WritePipe};
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

const PRECOMPILED_WASM: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/faker_wasm.dat")).len()] =
    *include_bytes!(concat!(env!("OUT_DIR"), "/faker_wasm.dat"));

#[derive(Debug, thiserror::Error)]
pub enum Error {}

pub fn generate(schema: &serde_json::Value, count: u16) -> Result<Vec<serde_json::Value>, Error> {
    let wasi_command_json = serde_json::json!({
        "schema": schema,
        "count": count,
    });
    let wasi_command = wasi_command_json.to_string();

    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s).expect("unexpected wasm linker error");

    let wasi_stdin = Box::new(ReadPipe::from(wasi_command));
    let wasi_stdout = Box::new(WritePipe::new_in_memory());

    let wasi_ctx = WasiCtxBuilder::new()
        .stdin(wasi_stdin)
        .stdout(wasi_stdout.clone())
        .build();
    let mut store = Store::new(&engine, wasi_ctx);

    // SAFETY: Module is trusted and comes from our build process
    let module = unsafe { Module::deserialize(&engine, &PRECOMPILED_WASM) }
        .expect("was compiled and embedded in the build script");

    linker
        .module(&mut store, "", &module)
        .expect("unexpected wasmtime error");
    linker
        .get_default(&mut store, "")
        .expect("unexpected wasmtime error")
        .typed::<(), ()>(&store)
        .expect("unexpected wasmtime error")
        .call(&mut store, ())
        .expect("unexpected wasmtime error");

    drop(store);
    let module_out = wasi_stdout.try_into_inner().expect("unique ownership");
    let str_out = std::str::from_utf8(&module_out.get_ref()).expect("must be a valid UTF8");
    Ok(serde_json::from_str(str_out).expect("unexpectedly malformed JSON"))
}
