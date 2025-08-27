use rquickjs::{
    Ctx, Module, Result,
    loader::{Loader, ModuleLoader, ScriptLoader},
    module::ModuleDef,
};

#[derive(Debug)]
pub struct MyLoader {
    module_loader: ModuleLoader,
    script_loader: ScriptLoader,
}

impl Default for MyLoader {
    fn default() -> Self {
        let script_loader = ScriptLoader::default();
        let module_loader = ModuleLoader::default();

        Self {
            script_loader,
            module_loader,
        }
    }
}

impl MyLoader {
    pub fn with_module<N: Into<String>, M: ModuleDef>(mut self, name: N, module: M) -> Self {
        self.module_loader.add_module(name, module);
        self
    }
}

impl Loader for MyLoader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, path: &str) -> Result<Module<'js>> {
        self.module_loader
            .load(ctx, path)
            .or_else(|_| self.script_loader.load(ctx, path))
    }
}
