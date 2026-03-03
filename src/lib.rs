use wasmtime::*;
use wasmtime_wasi::{
    p1::{self, WasiP1Ctx},
    p2::pipe::{MemoryInputPipe, MemoryOutputPipe},
    WasiCtxBuilder,
};

const PRECOMPILED_WASM: [u8; include_bytes!(concat!(env!("OUT_DIR"), "/faker_wasm.dat")).len()] =
    *include_bytes!(concat!(env!("OUT_DIR"), "/faker_wasm.dat"));

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(String);

impl From<wasmtime::Error> for Error {
    fn from(err: wasmtime::Error) -> Self {
        Error(format!("json-schema-faker runtime error: {err}"))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error(format!("json-schema-faker output decode error: {err}"))
    }
}

pub fn generate(schema: &serde_json::Value, count: u16) -> Result<Vec<serde_json::Value>, Error> {
    let wasi_command_json = serde_json::json!({
        "schema": schema,
        "count": count,
    });
    let wasi_command = wasi_command_json.to_string();

    let engine = Engine::default();
    let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
    p1::add_to_linker_sync(&mut linker, |s| s).expect("unexpected wasm linker error");

    let wasi_stdin = MemoryInputPipe::new(wasi_command.into_bytes());
    let wasi_stdout = MemoryOutputPipe::new(1024 * 1024 * 10);
    let wasi_stderr = MemoryOutputPipe::new(1024 * 1024);

    let mut wasi_ctx_builder = WasiCtxBuilder::new();
    wasi_ctx_builder
        .stdin(wasi_stdin)
        .stdout(wasi_stdout.clone())
        .stderr(wasi_stderr.clone());
    let wasi_ctx = wasi_ctx_builder.build_p1();
    let mut store = Store::new(&engine, wasi_ctx);

    // SAFETY: Module is trusted and comes from our build process
    let module = unsafe { Module::deserialize(&engine, &PRECOMPILED_WASM) }
        .expect("was compiled and embedded in the build script");

    let call_result = (|| -> wasmtime::Result<()> {
        linker.module(&mut store, "", &module)?;
        let f = linker.get_default(&mut store, "")?;
        let f = f.typed::<(), ()>(&store)?;
        f.call(&mut store, ())?;
        Ok(())
    })();

    if let Err(err) = call_result {
        let stderr = wasi_stderr.contents();
        if !stderr.is_empty() {
            if let Ok(stderr_text) = std::str::from_utf8(stderr.as_ref()) {
                return Err(Error(format!(
                    "json-schema-faker runtime error: {err}. wasi stderr: {}",
                    stderr_text.trim()
                )));
            }
        }
        return Err(err.into());
    }

    drop(store);
    let module_out = wasi_stdout.contents();
    let str_out = std::str::from_utf8(module_out.as_ref()).expect("must be a valid UTF8");
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
