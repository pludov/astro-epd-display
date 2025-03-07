use gtmpl::{Func, FuncError};

use super::numeric::{NumberToTmpl, TmplToF64};
use super::string::{StringToTmpl, TmplToString};

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

fn func_div(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    let mut result = args[0].to_float()?;
    for a in &args[1..] {
        result /= a.to_float()?;
    }

    Ok(result.to_tmpl())
}
fn func_mul(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    let mut result = args[0].to_float()?;
    for a in &args[1..] {
        result *= a.to_float()?;
    }

    Ok(result.to_tmpl())
}

fn func_mod(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    let mut result = args[0].to_float()?;
    for a in &args[1..] {
        result %= a.to_float()?;
    }

    Ok(result.to_tmpl())
}

fn func_round(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    let v = args[0].to_float()?;
    Ok(v.round().to_tmpl())
}

fn func_floor(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    let v = args[0].to_float()?;
    Ok(v.floor().to_tmpl())
}

fn func_lpad(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    let v = args[0].to_str()?;
    let pad = args[1].to_float()? as usize;
    let pad_char = args[2].to_str()?;
    let mut result = v.to_string();
    while result.len() < pad {
        result = format!("{}{}", pad_char, result);
    }
    Ok(result.to_tmpl())
}

pub fn funcs() -> Vec<(&'static str, Func)> {
    vec![
        ("add", func_add),
        ("sub", func_sub),
        ("div", func_div),
        ("mul", func_mul),
        ("mod", func_mod),
        ("round", func_round),
        ("floor", func_floor),
        ("lpad", func_lpad),
    ]
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

    #[test]
    fn test_mod() {
        assert_eq!(render(r#"{{ mod 1 2 }}"#, Value::NoValue), "1");
        assert_eq!(render(r#"{{ mod 1.2 2.4 }}"#, Value::NoValue), "1.2");
        // 32 bits int overflow
        assert_eq!(render(r#"{{ mod 4294967295 1 }}"#, Value::NoValue), "0");
        // Float
        assert_eq!(render(r#"{{ mod 1.1 2.2 }}"#, Value::NoValue), "1.1");
    }
    #[test]
    fn test_round_fn() {
        assert_eq!(render(r#"{{ round 1.1 }}"#, Value::NoValue), "1");
        assert_eq!(render(r#"{{ round 1.5 }}"#, Value::NoValue), "2");
        assert_eq!(render(r#"{{ round 1.6 }}"#, Value::NoValue), "2");
        assert_eq!(render(r#"{{ round 1.9 }}"#, Value::NoValue), "2");
        assert_eq!(render(r#"{{ round -1.2 }}"#, Value::NoValue), "-1");
    }

    #[test]
    fn test_floor_fn() {
        assert_eq!(render(r#"{{ floor 1.1 }}"#, Value::NoValue), "1");
        assert_eq!(render(r#"{{ floor 1.5 }}"#, Value::NoValue), "1");
        assert_eq!(render(r#"{{ floor 1.6 }}"#, Value::NoValue), "1");
        assert_eq!(render(r#"{{ floor 1.9 }}"#, Value::NoValue), "1");
        assert_eq!(render(r#"{{ floor -1.2 }}"#, Value::NoValue), "-2");
    }

    #[test]
    fn test_lpad_fn() {
        assert_eq!(render(r#"{{ lpad "1" 5 "0" }}"#, Value::NoValue), "00001");
        assert_eq!(render(r#"{{ lpad "1" 5 "1" }}"#, Value::NoValue), "11111");
        assert_eq!(render(r#"{{ lpad  1  5 "x" }}"#, Value::NoValue), "xxxx1");
        assert_eq!(render(r#"{{ lpad 11  5 "x" }}"#, Value::NoValue), "xxx11");
        assert_eq!(render(r#"{{ lpad "1" 2 "00" }}"#, Value::NoValue), "001");
    }
}
