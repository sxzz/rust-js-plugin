use oxc_resolver::{ResolveOptions, Resolver as OxcBaseResolver};
use rquickjs::{Ctx, Error, Result, loader::Resolver};
use std::path::Path;

pub struct OxcResolver {
    oxc_resolver: OxcBaseResolver,
}

impl OxcResolver {
    pub fn new() -> Self {
        let oxc_resolver = OxcBaseResolver::new(ResolveOptions::default());
        Self { oxc_resolver }
    }
}

impl Resolver for OxcResolver {
    fn resolve<'js>(&mut self, _ctx: &Ctx<'js>, base: &str, name: &str) -> Result<String> {
        let base_dir = Path::new(base).parent().unwrap();
        let result = self.oxc_resolver.resolve(base_dir, name);
        match result {
            Ok(resolved) => Ok(resolved.full_path().to_str().unwrap().to_string()),
            Err(_) => Err(Error::new_resolving(base, name)),
        }
    }
}
