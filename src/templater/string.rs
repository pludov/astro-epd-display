use gtmpl::FuncError;

pub trait TmplToString {
    fn to_str(&self) -> Result<String, FuncError>;
}

impl TmplToString for gtmpl::Value {
    fn to_str(&self) -> Result<String, FuncError> {
        match self {
            gtmpl::Value::String(ref n) => Ok(n.clone()),
            gtmpl::Value::Number(ref n) => {
                let v = n.as_f64();
                if v.is_some() {
                    return Ok(v.unwrap().to_string());
                }

                let v = n.as_i64().map(|e| e as f64);
                if v.is_some() {
                    return Ok(v.unwrap().to_string());
                }

                let v = n.as_i64().map(|e| e as f64);
                if v.is_some() {
                    return Ok(v.unwrap().to_string());
                }
                return Err(FuncError::UnableToConvertFromValue);
            }
            _ => Err(FuncError::UnableToConvertFromValue),
        }
    }
}

pub trait StringToTmpl {
    fn to_tmpl(&self) -> gtmpl::Value;
}

impl StringToTmpl for String {
    fn to_tmpl(&self) -> gtmpl::Value {
        gtmpl::Value::String(self.clone())
    }
}
