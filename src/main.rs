use std::path::Path;

use wasmer::{
    imports, Function, FunctionEnv, FunctionEnvMut, FunctionType, Instance, Memory, Module, Store,
    Type, WasmPtr,
};

fn main() -> anyhow::Result<()> {
    let mut store = Store::default();
    let module = Module::from_file(&store, Path::new("wasm/script.wasm"))?;

    struct Env {
        // the memory field is an Option because it cant be directly initialized to the memory of our script
        // since we need to declare our environment before initializing the script, but we need the script
        // to be inicialized to access it's memory
        memory: Option<Memory>,
    }

    let print_def = |mut _env: FunctionEnvMut<Env>, str_ptr: WasmPtr<u8>| {
        let (env, mut_store) = _env.data_and_store_mut();

        // This shouldn't ever fail. We initialize the memory variable as soon as our script runs. So this is always avaliable when the script calls this function
        let memory = env.memory.as_ref().unwrap();

        let memory_view = memory.view(&mut_store);
        let string = str_ptr.read_utf8_string_with_nul(&memory_view).unwrap();
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

    // Finally access the script's memory
    let memory = instance.exports.get_memory("memory")?;
    let mut env_mut = env.as_mut(&mut store);
    env_mut.memory = Some(memory.clone());

    let main_fn = instance.exports.get_function("main")?;
    main_fn.call(&mut store, &[])?;

    Ok(())
}
