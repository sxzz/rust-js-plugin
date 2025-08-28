use crate::{
    combine::{CombineLoader, CombineResolver},
    js_plugin::JsPlugin,
    native_api::RustModule,
    resolver::OxcResolver,
};
use anyhow::Ok;
use llrt_modules::module_builder::{GlobalAttachment, ModuleBuilder};
use rolldown::{BundlerBuilder, BundlerOptions};
use rolldown_common::Output;
use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Module, Object, Persistent, async_with,
    loader::{BuiltinResolver, ModuleLoader, ScriptLoader},
};
use std::{fs::read_to_string, path::Path, sync::Arc, time::Instant, vec};

mod combine;
mod js_plugin;
mod native_api;
mod resolver;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let start = Instant::now();

    let (_, context, global_attachment) = init_js().await?;

    let mut options = BundlerOptions::default();
    options.input = Some(vec!["/virtual".to_string().into()]);

    let (name, plugin_object) = async_with!(context => |ctx| {
        global_attachment.attach(&ctx)?;

        let path = Path::new("./js/plugin.js").canonicalize()?;
        let code = read_to_string(&path)?;
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
        let x = Persistent::save(&ctx, default_export);
        Ok((name, x))
    })
    .await?;

    let rolldown_start = Instant::now();

    let context = Arc::new(context);
    let plugin = JsPlugin::new(name, context.clone(), plugin_object);
    let mut bundler = BundlerBuilder::default()
        .with_options(options)
        .with_plugins(vec![Arc::new(plugin)])
        .build();
    let output = bundler.generate().await.unwrap();
    let first_output = match output.assets.get(0) {
        Some(Output::Chunk(chunk)) => Some(&chunk.code),
        _ => None,
    }
    .unwrap();
    println!("Bundling output: {:}", first_output);

    bundler.close().await?;
    context.runtime().run_gc().await;

    println!("Rolldown time cost: {:?}", rolldown_start.elapsed());
    println!("Time cost: {:?}", start.elapsed());

    Ok(())
}

async fn init_js() -> anyhow::Result<(AsyncRuntime, AsyncContext, GlobalAttachment)> {
    let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;

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
    runtime.set_loader(combined_resolver, loader).await;

    Ok((runtime, context, global_attachment))
}
