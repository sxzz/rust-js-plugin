use boa_engine::{
    JsNativeError, JsString, Module, Source,
    module::{ModuleLoader, Referrer},
};
use oxc_resolver::{ResolveOptions, Resolver};
use rustc_hash::FxHashMap;
use std::{cell::RefCell, env::current_dir, fs::read_to_string};

pub struct JsModuleLoader {
    modules: RefCell<FxHashMap<JsString, Module>>,
}

impl JsModuleLoader {
    pub fn new() -> Self {
        Self {
            modules: Default::default(),
        }
    }
}

impl ModuleLoader for JsModuleLoader {
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(boa_engine::JsResult<Module>, &mut boa_engine::Context)>,
        context: &mut boa_engine::Context,
    ) {
        if let Some(module) = self.modules.borrow().get(&specifier) {
            finish_load(Ok(module.clone()), context);
            return;
        }

        let specifier = specifier.to_std_string_lossy();
        let cwd = current_dir().unwrap();
        let base_dir = referrer
            .path()
            .and_then(|path| path.parent())
            .unwrap_or_else(|| cwd.as_path());

        let options = ResolveOptions {
            ..Default::default()
        };

        let resolver = Resolver::new(options);
        match resolver.resolve(&base_dir, &specifier) {
            Ok(resolution) => {
                let source = read_to_string(resolution.full_path());
                match source {
                    Ok(source) => {
                        let source = Source::from_bytes(source.as_bytes());
                        let module = Module::parse(source, None, context);
                        finish_load(module, context);
                    }
                    Err(err) => finish_load(
                        Err(JsNativeError::error().with_message(err.to_string()).into()),
                        context,
                    ),
                }
            }
            Err(err) => finish_load(
                Err(JsNativeError::error().with_message(err.to_string()).into()),
                context,
            ),
        }
    }

    fn register_module(&self, specifier: JsString, module: Module) {
        self.modules.borrow_mut().insert(specifier, module);
    }

    fn get_module(&self, specifier: JsString) -> Option<Module> {
        self.modules.borrow().get(&specifier).cloned()
    }
}
