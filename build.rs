use std::{
    env, fs,
    path::Path,
    process::{Command, ExitStatus},
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=js/src/main.js");

    let out_dir = env::var_os("OUT_DIR").unwrap();

    dircpy::copy_dir("js", Path::new(&out_dir).join("js")).expect("cannot copy JS sources");

    // Run `npm install` for all JS dependencies including bundler
    if !Command::new("npm")
        .arg("install")
        .arg("--prefix")
        .arg("js")
        .current_dir(&out_dir)
        .status()
        .as_ref()
        .map(ExitStatus::success)
        .unwrap_or(false)
    {
        panic!("Error running `npm install`");
    }

    // Build a JS wrapper library as a file
    if !Command::new("npm")
        .arg("run")
        .arg("build")
        .arg("--prefix")
        .arg("js")
        .current_dir(&out_dir)
        .status()
        .as_ref()
        .map(ExitStatus::success)
        .unwrap_or(false)
    {
        panic!("Error running `npm run build`");
    }

    // Build a WASM binary from the JS bundle
    if !Command::new("javy")
        .arg("compile")
        .arg("js/dist/bundle.mjs")
        .current_dir(out_dir.clone())
        .status()
        .as_ref()
        .map(ExitStatus::success)
        .unwrap_or(false)
    {
        panic!("Error running `javy compile`");
    }

    wasmtime_precompile(Path::new(&out_dir));
}

fn wasmtime_precompile(out_dir: &Path) {
    let engine = wasmtime::Engine::default();

    let bytes = engine
        .precompile_module(&fs::read(out_dir.join("index.wasm")).expect("missing WASM file"))
        .expect("cannot precompile WASM with Wasmtime");
    fs::write(out_dir.join("faker_wasm.dat"), bytes).expect("cannot write precompiled WASM module");
}
