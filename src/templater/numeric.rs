use gtmpl::FuncError;

pub trait TmplToF64 {
    fn to_float(&self) -> Result<f64, FuncError>;
}

impl TmplToF64 for gtmpl::Value {
    fn to_float(&self) -> Result<f64, FuncError> {
        match self {
            gtmpl::Value::Number(ref n) => {
                let v = n.as_f64();
                if v.is_some() {
                    return Ok(v.unwrap());
                }

                let v = n.as_i64().map(|e| e as f64);
                if v.is_some() {
                    return Ok(v.unwrap());
                }

                let v = n.as_i64().map(|e| e as f64);
                if v.is_some() {
                    return Ok(v.unwrap());
                }
                return Err(FuncError::UnableToConvertFromValue);
            }
            _ => Err(FuncError::UnableToConvertFromValue),
        }
    }
}

pub trait NumberToTmpl {
    fn to_tmpl(&self) -> gtmpl::Value;
}

impl NumberToTmpl for f64 {
    fn to_tmpl(&self) -> gtmpl::Value {
        gtmpl::Value::Number(gtmpl_value::Number::from(*self))
    }
}
