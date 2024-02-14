use wasi_common::pipe::{ReadPipe, WritePipe};
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

const PRECOMPILED_WASM: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/faker_wasm.dat")).len()] =
    *include_bytes!(concat!(env!("OUT_DIR"), "/faker_wasm.dat"));

#[derive(Debug, thiserror::Error)]
#[error("json-schema-faker can't produce an output for the schema")]
pub struct Error;

impl From<wasmtime::Error> for Error {
    fn from(_: wasmtime::Error) -> Self {
        Error
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        Error
    }
}

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
        .call(&mut store, ())?;

    drop(store);
    let module_out = wasi_stdout.try_into_inner().expect("unique ownership");
    let str_out = std::str::from_utf8(&module_out.get_ref()).expect("must be a valid UTF8");
    Ok(serde_json::from_str(str_out)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn good() {
        const SCHEMA: &'static str = r#"
{
    "type": "object",
    "required": ["name", "credits"],
    "properties": {
        "name": { "type": "string" },
        "credits": { "type": "integer", "minimum": 0, "maximum": 13371337 }
    }
}"#;

        let json_schema: serde_json::Value =
            serde_json::from_str(SCHEMA).expect("must be a valid JSON");
        let documents = generate(&json_schema, 100).expect("schema must be correct");

        assert!(!documents.is_empty());

        let simple_check = documents
            .iter()
            .flat_map(|v| v.as_object())
            .map(|object| {
                object.contains_key("name")
                    && object
                        .get("credits")
                        .map(|credits| {
                            credits
                                .as_number()
                                .map(|n| n.as_u64())
                                .flatten()
                                .map(|c| c > 0)
                                .unwrap_or_default()
                        })
                        .unwrap_or_default()
            })
            .fold(true, |acc, x| acc && x);

        assert!(simple_check);
    }

    #[test]
    fn bad() {
        const SCHEMA: &'static str = r#"
{
    "type": "object",
    "required": ["name"],
    "properties": {
        "name": { "type": "kek" }
    }
}"#;

        let json_schema: serde_json::Value =
            serde_json::from_str(SCHEMA).expect("must be a valid JSON");
        assert!(generate(&json_schema, 10).is_err());
    }
}
