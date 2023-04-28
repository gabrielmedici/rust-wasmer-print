use std::path::Path;

use wasmer::{
    imports, Function, FunctionEnv, FunctionEnvMut, FunctionType, Instance, Memory, Module, Store,
    Type, WasmPtr,
};

fn main() -> anyhow::Result<()> {
    let mut store = Store::default();
    let module = Module::from_file(&store, Path::new("wasm/script.wasm"))?;

    struct Env {
        memory: Option<Memory>,
    }

    let print_def = |mut env: FunctionEnvMut<Env>, ptr: WasmPtr<u8>| {
        let (e, sstore) = env.data_and_store_mut();
        let mem = e.memory.as_ref().unwrap();

        let mem_view = mem.view(&sstore);
        let string = ptr.read_utf8_string_with_nul(&mem_view).unwrap();
        println!("Print from WASM: {:?}", string);
    };

    let env = FunctionEnv::new(&mut store, Env { memory: None });

    let print = Function::new_typed_with_env(&mut store, &env, print_def);

    let import_object = imports! {
        "env" => {
            "print" => print,
            // AssemblyScript needs this import. Doesn't do anything
            "abort" => Function::new(
                &mut store,
                &FunctionType::new(vec![Type::I32, Type::I32, Type::I32, Type::I32], vec![]),
                |args| Ok(vec![]),
            )
        }
    };

    let instance = Instance::new(&mut store, &module, &import_object)?;
    let memory = instance.exports.get_memory("memory")?;

    let mut env_mut = env.as_mut(&mut store);
    env_mut.memory = Some(memory.clone());

    let mainfn = instance.exports.get_function("main")?;
    mainfn.call(&mut store, &[])?;

    Ok(())
}
