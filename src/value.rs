use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'source> {
    /// null.
    Null,
    /// true or false.
    Bool(bool),
    /// Any floating point number.
    Float(f64),
    // And non floating point number
    Integer(i64),
    /// Any quoted string.
    String(&'source str),
    /// An array of values
    Array(Vec<Value<'source>>),
    /// An array of keys and values ( used to represent a dictionary )
    Object(Vec<(&'source str, Value<'source>)>),
}

impl serde::Serialize for Value<'_> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::String(s) => serializer.serialize_str(s),
            Value::Float(n) => serializer.serialize_f64(*n),
            Value::Integer(n) => serializer.serialize_i64(*n),
            Value::Array(arr) => arr.serialize(serializer),
            Value::Object(obj) => {
                // Flatten array to array of objects into a single JSON object
                let mut map = BTreeMap::new();
                for (key, value) in obj {
                    map.insert(key, value);
                }
                map.serialize(serializer)
            }
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Null => serializer.serialize_none(),
        }
    }
}
