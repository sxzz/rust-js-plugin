use rquickjs::{
    Ctx, Function, Result, Value,
    function::{IntoJsFunc, ParamRequirement, Params},
    module::{Declarations, Exports, ModuleDef},
};

struct AddFn;

impl<'js> IntoJsFunc<'js, ()> for AddFn {
    fn param_requirements() -> ParamRequirement {
        ParamRequirement::any()
    }

    fn call<'a>(&self, params: Params<'a, 'js>) -> Result<Value<'js>> {
        let count = params.len();

        let mut sum = 0.0;
        for i in 0..count {
            sum += params.arg(i).unwrap().as_number().unwrap_or(0.0);
        }

        Ok(Value::new_number(params.ctx().clone(), sum))
    }
}

pub struct RustModule;

impl ModuleDef for RustModule {
    fn declare(define: &Declarations) -> Result<()> {
        define.declare("builtin_str")?;
        define.declare("add")?;
        Ok(())
    }

    fn evaluate<'js>(ctx: &Ctx<'js>, exports: &Exports<'js>) -> Result<()> {
        exports.export("builtin_str", "this is built-in string")?;
        let function = Function::new(ctx.clone(), AddFn).unwrap();
        exports.export("add", function)?;
        Ok(())
    }
}
