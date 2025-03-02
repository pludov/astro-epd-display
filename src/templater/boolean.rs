use gtmpl::FuncError;

pub trait TmplToBool {
    fn to_bool(&self) -> Result<bool, FuncError>;
}

impl TmplToBool for gtmpl::Value {
    fn to_bool(&self) -> Result<bool, FuncError> {
        match self {
            gtmpl::Value::Number(ref n) => {
                let v = n.as_f64();
                if v.is_some() {
                    return Ok(v.unwrap() != 0.0);
                }

                let v = n.as_i64().map(|e| e as f64);
                if v.is_some() {
                    return Ok(v.unwrap() != 0.0);
                }

                let v = n.as_i64().map(|e| e as f64);
                if v.is_some() {
                    return Ok(v.unwrap() != 0.0);
                }
                println!("unrecognized boolean = {:?}", n);
                return Err(FuncError::UnableToConvertFromValue);
            }
            gtmpl::Value::Bool(b) => Ok(*b),
            gtmpl::Value::String(s) => Ok(!s.is_empty()),
            gtmpl::Value::Map(m) => Ok(!m.is_empty()),
            gtmpl::Value::Array(a) => Ok(!a.is_empty()),
            gtmpl::Value::Nil => Ok(false),
            gtmpl::Value::NoValue => Ok(false),
            a => {
                println!("unrecognized boolean = {:?}", a);
                Err(FuncError::UnableToConvertFromValue)
            }
        }
    }
}

pub trait BoolToTmpl {
    fn to_tmpl(&self) -> gtmpl::Value;
}

impl BoolToTmpl for bool {
    fn to_tmpl(&self) -> gtmpl::Value {
        gtmpl::Value::Bool(*self)
    }
}
