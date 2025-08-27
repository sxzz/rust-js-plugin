use oxc_resolver::{ResolveOptions, Resolver as OxcBaseResolver};
use rquickjs::{
    Ctx, Error, Result as QuickResult,
    loader::{BuiltinResolver, Resolver},
};
use std::{path::Path, result::Result::Ok};

#[derive(Debug, Default)]
pub struct OxcResolver {
    oxc_resolver: OxcBaseResolver,
    builtin_resolver: BuiltinResolver,
}

impl OxcResolver {
    pub fn new(builtin_resolver: BuiltinResolver) -> Self {
        let oxc_resolver = OxcBaseResolver::new(ResolveOptions::default());
        Self {
            oxc_resolver,
            builtin_resolver,
        }
    }
}

impl Resolver for OxcResolver {
    fn resolve<'js>(&mut self, ctx: &Ctx<'js>, base: &str, name: &str) -> QuickResult<String> {
        if let QuickResult::Ok(x) = self.builtin_resolver.resolve(ctx, base, name) {
            return QuickResult::Ok(x);
        };

        let base_dir = Path::new(base).parent().unwrap();
        let result = self.oxc_resolver.resolve(base_dir, name);
        match result {
            Ok(resolved) => Result::Ok(resolved.full_path().to_str().unwrap().to_string()),
            Err(_) => Err(Error::new_resolving(base, name)),
        }
    }
}
