use anyhow::Result;
use rolldown::plugin::{
    HookLoadArgs, HookLoadOutput, HookLoadReturn, HookResolveIdArgs, HookResolveIdOutput,
    HookResolveIdReturn, HookUsage, Plugin, PluginContext,
};
use rquickjs::{AsyncContext, CatchResultExt, Ctx, Function, Object, Persistent, Value};
use std::{borrow::Cow, fmt::Debug, sync::Arc};

pub struct JsPlugin {
    name: String,
    context: Arc<AsyncContext>,
    plugin_object: Persistent<Object<'static>>,
}

impl JsPlugin {
    pub fn new(
        name: String,
        context: Arc<AsyncContext>,
        plugin_object: Persistent<Object<'static>>,
    ) -> Self {
        Self {
            name,
            context,
            plugin_object,
        }
    }

    #[inline]
    fn get_plugin_object<'js>(&self, ctx: &Ctx<'js>) -> Object<'js> {
        self.plugin_object.clone().restore(ctx).unwrap()
    }
}

impl Debug for JsPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsPlugin")
            .field("name", &self.name)
            .finish()
    }
}

impl Plugin for JsPlugin {
    fn name(&self) -> Cow<'static, str> {
        self.name.clone().into()
    }

    fn register_hook_usage(&self) -> HookUsage {
        HookUsage::ResolveId | HookUsage::Load
    }

    async fn resolve_id(
        &self,
        _ctx: &PluginContext,
        args: &HookResolveIdArgs<'_>,
    ) -> HookResolveIdReturn {
        let s: Result<Option<String>> = self
            .context
            .with(|ctx| {
                let plugin_object = self.get_plugin_object(&ctx);
                let load_fn = plugin_object.get::<_, Function>("resolveId")?;
                let result: Value = load_fn
                    .call((args.specifier, args.importer))
                    .catch(&ctx)
                    .unwrap();

                if result.is_string() {
                    Ok(Some(result.as_string().unwrap().to_string()?))
                } else {
                    Ok(None)
                }
            })
            .await;

        return s.map(|s| {
            s.map(|s| HookResolveIdOutput {
                id: s.into(),
                ..Default::default()
            })
        });
    }

    async fn load(&self, _ctx: &PluginContext, args: &HookLoadArgs<'_>) -> HookLoadReturn {
        let s: Result<Option<String>> = self
            .context
            .with(|ctx| {
                let plugin_object = self.get_plugin_object(&ctx);
                let load_fn = plugin_object.get::<_, Function>("load")?;
                let result: Value = load_fn.call((args.id,)).catch(&ctx).unwrap();

                if result.is_string() {
                    Ok(Some(result.as_string().unwrap().to_string()?))
                } else {
                    Ok(None)
                }
            })
            .await;

        return s.map(|s| {
            s.map(|s| HookLoadOutput {
                code: s.into(),
                ..Default::default()
            })
        });
    }
}
