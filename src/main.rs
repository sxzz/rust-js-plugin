use crate::{loader::MyLoader, native_api::RustModule, resolver::OxcResolver};
use log::{LevelFilter, info};
use rquickjs::{CatchResultExt, Context, Function, Module, Object, Runtime};
use rquickjs_extra::console::init as init_console;
use std::{fs::read_to_string, path::Path, time::Instant};

mod loader;
mod native_api;
mod resolver;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .env()
        .with_level(LevelFilter::Info)
        .init()?;

    let start = Instant::now();

    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;

    let mut resolver = OxcResolver::new();
    resolver.builtin_resolver.add_module("native-api");

    let mut loader = MyLoader::default();
    loader.module_loader.add_module("native-api", RustModule);

    let path = Path::new("./js/main.js").canonicalize()?;
    let code = read_to_string(&path)?;

    runtime.set_loader(resolver, loader);

    context.with(|ctx| {
        init_console(&ctx).unwrap();

        let (decl, promise) = Module::declare(ctx.clone(), path.to_str().unwrap(), code)
            .catch(&ctx)
            .unwrap()
            .eval()
            .catch(&ctx)
            .unwrap();
        promise.finish::<()>().catch(&ctx).unwrap();

        let ns = decl.namespace().unwrap();
        let default_export = ns.get::<_, Object>("default").unwrap();

        let name = default_export.get::<_, String>("name").unwrap();
        let hook = default_export.get::<_, Function>("hook").unwrap();

        info!("Plugin name: {}", name);
        let result: i32 = hook.call((42,)).catch(&ctx).unwrap();
        info!("Hook result: {}", result);
    });

    info!("Time cost: {:?}", start.elapsed());
    Ok(())
}
