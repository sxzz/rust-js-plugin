use crate::{loader::MyLoader, native_api::RustModule, resolver::OxcResolver};
use log::{LevelFilter, info};
use rquickjs::{
    CatchResultExt, Context, Function, Module, Object, Runtime, loader::BuiltinResolver,
};
use rquickjs_extra::console::init as init_console;
use std::{fs::read_to_string, path::Path, time::Instant};

mod loader;
mod native_api;
mod resolver;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .env()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let start = Instant::now();

    let runtime = Runtime::new().unwrap();
    let context = Context::full(&runtime).unwrap();
    let built_resolver = BuiltinResolver::default().with_module("native-api");
    let resolver = OxcResolver::new(built_resolver);
    let loader = MyLoader::default().with_module("native-api", RustModule);
    let path = Path::new("./js/main.js").canonicalize().unwrap();
    let code = read_to_string(&path).unwrap();

    runtime.set_loader(resolver, loader);

    context.with(|ctx| {
        init_console(&ctx).unwrap();

        let (decl, promise) = Module::declare(ctx.clone(), path.to_str().unwrap(), code)
            .catch(&ctx)
            .unwrap()
            .eval()
            .catch(&ctx)
            .unwrap();
        promise.finish::<()>().unwrap();

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

// fn get_native_api(context: &mut Context) -> Module {
//     let echo = FunctionObjectBuilder::new(context.realm(), unsafe {
//         NativeFunction::from_closure(move |_this, args, context| {
//             Ok(JsArray::from_iter(args.iter().cloned(), context).into())
//         })
//     })
//     .name("echo")
//     .build();

//     let add = FunctionObjectBuilder::new(context.realm(), unsafe {
//         NativeFunction::from_closure(move |_this, args, _context| {
//             Ok(args
//                 .iter()
//                 .map(|v| v.as_number().unwrap_or(0.0))
//                 .sum::<f64>()
//                 .into())
//         })
//     })
//     .name("add")
//     .build();

//     Module::synthetic(
//         &[
//             js_string!("builtin_str"),
//             js_string!("echo"),
//             js_string!("add"),
//         ],
//         SyntheticModuleInitializer::from_copy_closure_with_captures(
//             |module, fns, _| {
//                 module.set_export(&js_string!("builtin_str"), fns.0.clone().into())?;
//                 module.set_export(&js_string!("echo"), fns.1.clone().into())?;
//                 module.set_export(&js_string!("add"), fns.2.clone().into())?;
//                 Ok(())
//             },
//             (js_string!("this is built-in string"), echo, add),
//         ),
//         None,
//         None,
//         context,
//     )
// }
