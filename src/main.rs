use crate::module_loader::JsModuleLoader;
use boa_engine::{
    Context, JsResult, JsValue, Module, NativeFunction, Source, js_string,
    module::{ModuleLoader, SyntheticModuleInitializer},
    object::{FunctionObjectBuilder, builtins::JsArray},
    property::{Attribute, PropertyKey},
};
use boa_runtime::{Console, TextDecoder, TextEncoder};
use std::{path::Path, rc::Rc, time::Instant};

mod module_loader;

fn main() -> JsResult<()> {
    let start = Instant::now();

    let module_loader = Rc::new(JsModuleLoader::new());
    let context = &mut Context::builder()
        .module_loader(Rc::clone(&module_loader))
        .build()
        .unwrap();
    module_loader.register_module(js_string!("native-api"), get_native_api(context));

    init_runtime(context);

    let source = Source::from_filepath(Path::new("./js/main.js")).unwrap();
    let module = Module::parse(source, None, context)?;

    let promise = module.load_link_evaluate(context);
    promise
        .await_blocking(context)
        .map_err(|e| e.display().to_string())
        .unwrap();

    let default = get_default_export(context, &module)?;
    let default = default.as_object().unwrap();
    let name = default
        .get(PropertyKey::String(js_string!("name")), context)?
        .as_string()
        .unwrap()
        .to_std_string()
        .unwrap();
    let hook = default
        .get(PropertyKey::String(js_string!("hook")), context)?
        .as_function()
        .unwrap();

    println!("Plugin name: {}", name);
    let result = hook
        .call(&JsValue::Undefined, &[JsValue::Integer(42)], context)
        .unwrap()
        .as_number()
        .unwrap();

    println!("Hook result: {}", result);

    println!("Time cost: {:?}", start.elapsed());
    Ok(())
}

fn init_runtime(context: &mut Context) {
    let console = Console::init(context);

    // Register the console as a global property to the context.
    context
        .register_global_property(Console::NAME, console, Attribute::all())
        .expect("the console object shouldn't exist yet");
    context
        .register_global_class::<TextDecoder>()
        .expect("the TextDecoder class should be registered");
    context
        .register_global_class::<TextEncoder>()
        .expect("the TextEncoder class should be registered");
}

pub fn get_default_export(context: &mut Context, module: &Module) -> JsResult<JsValue> {
    let namespace = module.namespace(context);
    namespace.get(js_string!("default"), context)
}

fn get_native_api(context: &mut Context) -> Module {
    let echo = FunctionObjectBuilder::new(context.realm(), unsafe {
        NativeFunction::from_closure(move |_this, args, context| {
            Ok(JsArray::from_iter(args.iter().cloned(), context).into())
        })
    })
    .name("echo")
    .build();

    let add = FunctionObjectBuilder::new(context.realm(), unsafe {
        NativeFunction::from_closure(move |_this, args, _context| {
            Ok(args
                .iter()
                .map(|v| v.as_number().unwrap_or(0.0))
                .sum::<f64>()
                .into())
        })
    })
    .name("add")
    .build();

    Module::synthetic(
        &[
            js_string!("builtin_str"),
            js_string!("echo"),
            js_string!("add"),
        ],
        SyntheticModuleInitializer::from_copy_closure_with_captures(
            |module, fns, _| {
                module.set_export(&js_string!("builtin_str"), fns.0.clone().into())?;
                module.set_export(&js_string!("echo"), fns.1.clone().into())?;
                module.set_export(&js_string!("add"), fns.2.clone().into())?;
                Ok(())
            },
            (js_string!("this is built-in string"), echo, add),
        ),
        None,
        None,
        context,
    )
}
