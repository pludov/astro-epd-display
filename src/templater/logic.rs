use gtmpl::{Func, FuncError};

use super::boolean::{BoolToTmpl, TmplToBool};

fn func_ternary(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    if args.len() != 3 {
        return Err(FuncError::ExactlyXArgs("ternary".to_string(), 3));
    }
    let cond = args[0].to_bool()?;
    if cond {
        Ok(args[1].clone())
    } else {
        Ok(args[2].clone())
    }
}

fn func_bool(args: &[gtmpl::Value]) -> Result<gtmpl::Value, FuncError> {
    if args.len() != 1 {
        return Err(FuncError::ExactlyXArgs("bool".to_string(), 1));
    }
    Ok((args[0].to_bool()?.to_tmpl()))
}

pub fn funcs() -> Vec<(&'static str, Func)> {
    vec![
        ("ternary", func_ternary),
        ("test.Ternary", func_ternary),
        ("bool", func_bool),
        ("conv.Bool", func_bool),
    ]
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::templater::JsonToYaml;

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
    fn test_ternary() {
        assert_eq!(render(r#"{{ ternary 1 2 3 }}"#, Value::NoValue), "2");
        assert_eq!(
            render(r#"{{ ternary false "a" "b" }}"#, Value::NoValue),
            "b"
        );
    }

    // Test a simple rendering
    #[test]
    fn test_bool() {
        let mut hash = HashMap::new();
        hash.insert("nu".to_string(), Value::Nil);
        assert_eq!(render(r#"{{ bool 1 }}"#, Value::NoValue), "true");
        assert_eq!(render(r#"{{ bool "a" }}"#, Value::NoValue), "true");
        assert_eq!(render(r#"{{ bool "" }}"#, Value::NoValue), "false");
        assert_eq!(
            render(
                r#"{{ bool .nu }}"#,
                gtmpl::Value::from(JsonToYaml {
                    json: serde_json::json!({ "nu": null})
                })
            ),
            "false"
        );
        assert_eq!(render(r#"{{ bool 0 }}"#, Value::NoValue), "false");
        assert_eq!(
            render(
                r#"{{ bool .missing }}"#,
                gtmpl::Value::from(JsonToYaml {
                    json: serde_json::json!({ "nu": null})
                })
            ),
            "false"
        );
    }
}
