use gtmpl::{Func, FuncError};

use super::numeric::{NumberToTmpl, TmplToF64};

fn func_add(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    let mut result = 0.0;
    for a in args {
        result += a.to_float()?;
    }

    Ok(result.to_tmpl())
}

pub fn funcs() -> Vec<(&'static str, Func)> {
    vec![("add", func_add)]
}

#[cfg(test)]
mod test {
    pub use super::*;
    use gtmpl::{Context, Template, Value};

    fn render(template: &str, state: Value) -> String {
        let mut tmpl = Template::default();
        tmpl.add_funcs(&funcs());

        tmpl.parse(template).unwrap();

        tmpl.render(&Context::from(state)).unwrap()
    }

    // Test a simple rendering
    #[test]
    fn test_add() {
        assert_eq!(render(r#"{{ add 1 2 3 }}"#, Value::NoValue), "6");
        assert_eq!(render(r#"{{ add 1.2 2.4 3.4 }}"#, Value::NoValue), "7");
        // 32 bits int overflow
        assert_eq!(
            render(r#"{{ add 4294967295 1 }}"#, Value::NoValue),
            "4294967296"
        );
        // Float
        assert_eq!(render(r#"{{ add 1.1 2.2 3.3 }}"#, Value::NoValue), "6.6");
    }
}
