use embedded_graphics::text::Alignment;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use Alignment::*;

pub fn default() -> Option<Alignment> {
    None
}

pub fn serialize<S>(v: &Option<Alignment>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if v.is_none() {
        return Option::<String>::None.serialize(s);
    }

    let v: &str = match v.unwrap() {
        Left => "left",
        Right => "right",
        Center => "center",
    };

    v.to_string().serialize(s)
}

pub fn deserialize<'de, D>(d: D) -> Result<Option<Alignment>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Option::<String>::deserialize(d)?.map(|e| e.to_lowercase());

    if v.is_none() {
        return Ok(None);
    } else {
        match v.unwrap().as_str() {
            "left" => Ok(Some(Alignment::Left)),
            "right" => Ok(Some(Alignment::Right)),
            "center" => Ok(Some(Alignment::Center)),
            v => {
                println!("Invalid alignment : {:?}", v);
                Ok(None)
            }
        }
    }
}
