use crate::{
    combine::{CombineLoader, CombineResolver},
    native_api::RustModule,
    resolver::OxcResolver,
};
use llrt_modules::module_builder::ModuleBuilder;
use rquickjs::{
    CatchResultExt, Context, Function, Module, Object, Runtime,
    loader::{BuiltinResolver, ModuleLoader, ScriptLoader},
};
use std::{fs::read_to_string, path::Path, time::Instant};

mod combine;
mod native_api;
mod resolver;

fn main() -> anyhow::Result<()> {
    let start = Instant::now();

    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    let builtin_resolver = BuiltinResolver::default().with_module("native-api");
    let (llrt_resolver, llrt_loader, global_attachment) = ModuleBuilder::default().build();
    let oxc_resolver = OxcResolver::new();
    let combined_resolver = CombineResolver::new()
        .with_resolver(builtin_resolver)
        .with_resolver(llrt_resolver)
        .with_resolver(oxc_resolver);

    let module_loader = ModuleLoader::default().with_module("native-api", RustModule);
    let script_loader = ScriptLoader::default();
    let loader = CombineLoader::default()
        .with_loader(module_loader)
        .with_loader(llrt_loader)
        .with_loader(script_loader);

    let path = Path::new("./js/main.js").canonicalize()?;
    let code = read_to_string(&path)?;

    runtime.set_loader(combined_resolver, loader);

    context.with(|ctx| -> anyhow::Result<()> {
        global_attachment.attach(&ctx)?;

        let (decl, promise) = Module::declare(ctx.clone(), path.to_str().unwrap(), code)
            .catch(&ctx)
            .unwrap()
            .eval()
            .catch(&ctx)
            .unwrap();
        promise.finish::<()>().catch(&ctx).unwrap();

        let ns = decl.namespace().unwrap();
        let default_export = ns.get::<_, Object>("default")?;

        let name = default_export.get::<_, String>("name")?;
        let hook = default_export.get::<_, Function>("hook")?;

        println!("Plugin name: {}", name);
        let result: i32 = hook.call((42,)).catch(&ctx).unwrap();
        println!("Hook result: {}", result);

        Ok(())
    })?;

    println!("Time cost: {:?}", start.elapsed());
    Ok(())
}
