use rquickjs::{
    Ctx, Error, Module, Result,
    loader::{Loader, Resolver},
};

pub struct CombineLoader {
    loaders: Vec<Box<dyn Loader>>,
}

impl Default for CombineLoader {
    fn default() -> Self {
        Self {
            loaders: Vec::new(),
        }
    }
}

impl CombineLoader {
    pub fn add_loader<L: Loader + 'static>(&mut self, loader: L) {
        self.loaders.push(Box::new(loader));
    }

    pub fn with_loader<L: Loader + 'static>(mut self, loader: L) -> Self {
        self.add_loader(loader);
        self
    }
}

impl Loader for CombineLoader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, path: &str) -> Result<Module<'js>> {
        for loader in &mut self.loaders {
            if let Ok(module) = loader.load(ctx, path) {
                return Ok(module);
            }
        }
        Err(Error::new_loading(path))
    }
}

pub struct CombineResolver {
    resolvers: Vec<Box<dyn Resolver>>,
}

impl CombineResolver {
    pub fn new() -> Self {
        Self {
            resolvers: Vec::new(),
        }
    }

    pub fn add_resolver<R: Resolver + 'static>(&mut self, resolver: R) {
        self.resolvers.push(Box::new(resolver));
    }

    pub fn with_resolver<R: Resolver + 'static>(mut self, resolver: R) -> Self {
        self.add_resolver(resolver);
        self
    }
}

impl Resolver for CombineResolver {
    fn resolve<'js>(&mut self, ctx: &Ctx<'js>, base: &str, name: &str) -> Result<String> {
        for resolver in &mut self.resolvers {
            if let Ok(resolved) = resolver.resolve(ctx, base, name) {
                return Ok(resolved);
            }
        }
        Err(Error::new_resolving(base, name))
    }
}
