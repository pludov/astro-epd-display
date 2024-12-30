use gtmpl::{Func, FuncError};

use super::numeric::{NumberToTmpl, TmplToF64};

fn func_add(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    let mut result = 0.0;
    for a in args {
        result += a.to_float()?;
    }

    Ok(result.to_tmpl())
}

fn func_sub(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    let mut result = args[0].to_float()?;
    for a in &args[1..] {
        result -= a.to_float()?;
    }

    Ok(result.to_tmpl())
}

pub fn funcs() -> Vec<(&'static str, Func)> {
    vec![("add", func_add), ("sub", func_sub)]
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
    #[test]

    fn test_round() {
        let v: f64 = -4.4;
        println!("v.fract() = {}", v.fract());
        let n = gtmpl::Value::Number(gtmpl_value::Number::from(v));
        assert_eq!(n.to_float().unwrap(), -4.4);
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

    #[test]
    fn test_sub() {
        assert_eq!(render(r#"{{ sub 1 2 3 }}"#, Value::NoValue), "-4");
        assert_eq!(render(r#"{{ sub 1.2 2.4 3 }}"#, Value::NoValue), "-4");
        // 32 bits int overflow
        assert_eq!(
            render(r#"{{ sub -4294967296 -1 }}"#, Value::NoValue),
            "-4294967295"
        );
        // Float
        assert_eq!(render(r#"{{ sub 1.1 2.2 3.3 }}"#, Value::NoValue), "-4.4");
    }
}
