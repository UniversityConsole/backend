//! Deserialize an HashMap into a Rust data structure.

use std::collections::HashMap;

use aws_sdk_dynamodb::model::AttributeValue;

use serde::de::IntoDeserializer;

use crate::error::{Error, Result};

macro_rules! impl_deserialize_n {
    ($type:ty, $method:ident, $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: serde::de::Visitor<'de>,
        {
            let val = self
                .read
                .get_attribute_value(&self.current_field)
                .ok_or_else(|| Error {
                    message: format!("missing integer for field {:?}", &self.current_field),
                })?
                .clone();
            if let AttributeValue::N(val) = val {
                visitor.$visit(val.parse::<$type>().map_err(|_| Error {
                    message: "Invalid type".to_owned(),
                })?)
            } else {
                Err(Error {
                    message: format!("missing integer for field {:?}", &self.current_field),
                })
            }
        }
    };
}

#[derive(Debug)]
enum Index {
    String(String),
    Number(usize),
    None,
}

trait Read: Clone {
    fn get_attribute_value(&self, index: &Index) -> Option<&AttributeValue>;
    fn get_keys(&self) -> Vec<String>;
}

#[derive(Clone)]
struct HashMapRead<S: ::std::hash::BuildHasher + Clone> {
    hashmap: HashMap<String, AttributeValue, S>,
}
impl<S: ::std::hash::BuildHasher + Clone> HashMapRead<S> {
    fn new(hm: HashMap<String, AttributeValue, S>) -> Self {
        HashMapRead { hashmap: hm }
    }
}
impl<S: ::std::hash::BuildHasher + Clone> Read for HashMapRead<S> {
    fn get_attribute_value(&self, index: &Index) -> Option<&AttributeValue> {
        match *index {
            Index::String(ref key) => self.hashmap.get(key),
            _ => None,
        }
    }
    fn get_keys(&self) -> Vec<String> {
        self.hashmap.keys().cloned().collect()
    }
}

#[derive(Clone)]
struct VecRead {
    vec: Vec<AttributeValue>,
}

impl Read for VecRead {
    fn get_attribute_value(&self, index: &Index) -> Option<&AttributeValue> {
        match *index {
            Index::Number(key) => self.vec.get(key),
            _ => None,
        }
    }
    fn get_keys(&self) -> Vec<String> {
        return vec![];
    }
}

#[derive(Debug)]
struct Deserializer<R> {
    read: R,
    current_field: Index,
    as_key: bool,
}
impl<'de, R> Deserializer<R>
where
    R: Read,
{
    pub fn new(read: R) -> Self {
        Deserializer {
            read,
            current_field: Index::None,
            as_key: false,
        }
    }
}

impl<'de, 'a, R: Read> serde::de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let f = self
            .read
            .get_attribute_value(&self.current_field)
            .ok_or_else(|| Error {
                message: format!("missing for field {:?}", &self.current_field),
            })?
            .clone();

        match f {
            AttributeValue::B(_) => self.deserialize_byte_buf(visitor),
            AttributeValue::Bool(_) => self.deserialize_bool(visitor),
            AttributeValue::L(_) => self.deserialize_bool(visitor),
            AttributeValue::M(_) => self.deserialize_map(visitor),
            AttributeValue::N(_) => self.deserialize_f64(visitor),
            AttributeValue::Null(_) => self.deserialize_unit(visitor),
            AttributeValue::S(_) => self.deserialize_str(visitor),
            _ => unreachable!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let val = self
            .read
            .get_attribute_value(&self.current_field)
            .ok_or_else(|| Error {
                message: "Missing field".to_owned(),
            })?
            .clone();
        if let AttributeValue::Bool(val) = val {
            visitor.visit_bool(val)
        } else {
            Err(Error {
                message: "Invalid type".to_owned(),
            })
        }
    }

    impl_deserialize_n!(i8, deserialize_i8, visit_i8);
    impl_deserialize_n!(i16, deserialize_i16, visit_i16);
    impl_deserialize_n!(i32, deserialize_i32, visit_i32);
    impl_deserialize_n!(i64, deserialize_i64, visit_i64);

    impl_deserialize_n!(u8, deserialize_u8, visit_u8);
    impl_deserialize_n!(u16, deserialize_u16, visit_u16);
    impl_deserialize_n!(u32, deserialize_u32, visit_u32);
    impl_deserialize_n!(u64, deserialize_u64, visit_u64);

    impl_deserialize_n!(f32, deserialize_f32, visit_f32);
    impl_deserialize_n!(f64, deserialize_f64, visit_f64);

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let val = self
            .read
            .get_attribute_value(&self.current_field)
            .ok_or_else(|| Error {
                message: format!("missing char for field {:?}", &self.current_field),
            })?
            .clone();
        if let AttributeValue::S(val) = val {
            let val = val.parse::<char>().map_err(|_| Error {
                message: "Invalid type".to_owned(),
            })?;
            visitor.visit_char(val)
        } else {
            Err(Error {
                message: format!("missing char for field {:?}", &self.current_field),
            })
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.as_key {
            match &self.current_field {
                Index::String(ref key) => visitor.visit_str(key),
                _ => visitor.visit_str(""),
            }
        } else if let Some(field) = self.read.get_attribute_value(&self.current_field) {
            if let AttributeValue::S(val) = field.clone() {
                visitor.visit_str(&val)
            } else {
                Err(Error {
                    message: format!("missing string for field {:?}", &self.current_field),
                })
            }
        } else {
            visitor.visit_str("")
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if let Some(field) = self.read.get_attribute_value(&self.current_field) {
            if let AttributeValue::B(val) = field.clone() {
                visitor.visit_bytes(val.as_ref())
            } else {
                Err(Error {
                    message: format!("missing bytes for field {:?}", &self.current_field),
                })
            }
        } else {
            visitor.visit_bytes(b"")
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.read.get_attribute_value(&self.current_field).is_none() {
            return visitor.visit_none();
        }
        match self
            .read
            .get_attribute_value(&self.current_field)
            .ok_or_else(|| Error {
                message: format!("missing option for field {:?}", &self.current_field),
            })? {
            AttributeValue::Null(true) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let val = self
            .read
            .get_attribute_value(&self.current_field)
            .ok_or_else(|| Error {
                message: "Missing field".to_owned(),
            })?;
        if let AttributeValue::Null(_) = val {
            visitor.visit_unit()
        } else {
            Err(Error {
                message: format!("Missing null for field {:?}", &self.current_field),
            })
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let list = self
            .read
            .get_attribute_value(&self.current_field)
            .ok_or_else(|| Error {
                message: format!("missing sequence for field {:?}", &self.current_field),
            })?
            .clone();
        let read = match list {
            AttributeValue::L(alist) => VecRead { vec: alist },
            AttributeValue::Ns(numlist) => VecRead {
                vec: numlist.into_iter().map(|n| AttributeValue::N(n)).collect(),
            },
            AttributeValue::Ss(slist) => VecRead {
                vec: slist.into_iter().map(|s| AttributeValue::S(s)).collect(),
            },
            AttributeValue::Bs(blist) => VecRead {
                vec: blist.into_iter().map(|s| AttributeValue::B(s)).collect(),
            },
            _ => {
                return Err(Error {
                    message: "No sequence input found".to_owned(),
                });
            }
        };

        let mut des = Deserializer::new(read);
        visitor.visit_seq(SeqAccess::new(&mut des))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.current_field {
            Index::None => {
                let mut des = Deserializer::new(self.read.clone());
                visitor.visit_seq(TupleAccess::new(&mut des))
            }
            _ => {
                let val = self
                    .read
                    .get_attribute_value(&self.current_field)
                    .ok_or_else(|| Error {
                        message: format!("missing hashmap for field {:?}", &self.current_field),
                    })?;
                match val {
                    AttributeValue::M(val) => {
                        let subread = HashMapRead {
                            hashmap: val.clone(),
                        };
                        let mut des = Deserializer::new(subread);
                        visitor.visit_seq(TupleAccess::new(&mut des))
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.current_field {
            Index::None => {
                let mut des = Deserializer::new(self.read.clone());
                visitor.visit_seq(TupleAccess::new(&mut des))
            }
            _ => {
                let val = self
                    .read
                    .get_attribute_value(&self.current_field)
                    .ok_or_else(|| Error {
                        message: format!("missing hashmap for field {:?}", &self.current_field),
                    })?;
                match val {
                    AttributeValue::M(val) => {
                        let subread = HashMapRead {
                            hashmap: val.clone(),
                        };
                        let mut des = Deserializer::new(subread);
                        visitor.visit_seq(TupleAccess::new(&mut des))
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.current_field {
            Index::None => visitor.visit_map(MapAccess::new(self, self.read.get_keys())),
            _ => {
                let map = self
                    .read
                    .get_attribute_value(&self.current_field)
                    .ok_or_else(|| Error {
                        message: format!("missing struct for field {:?}", &self.current_field),
                    })?;
                let hm = if let AttributeValue::M(val) = map.clone() {
                    val
                } else {
                    HashMap::new()
                };
                let keys = hm.keys().cloned().collect();
                let mut des = Deserializer::new(HashMapRead::new(hm));
                visitor.visit_map(MapAccess::new(&mut des, keys))
            }
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.current_field {
            Index::None => visitor.visit_map(MapAccess::new(self, self.read.get_keys())),
            _ => {
                let map = self
                    .read
                    .get_attribute_value(&self.current_field)
                    .ok_or_else(|| Error {
                        message: format!("missing struct for field {:?}", &self.current_field),
                    })?;
                if let AttributeValue::M(hm) = map.clone() {
                    let keys = hm.keys().cloned().collect();
                    let mut des = Deserializer::new(HashMapRead::new(hm));
                    visitor.visit_map(MapAccess::new(&mut des, keys))
                } else {
                    Err(Error {
                        message: "Missing struct fields".to_owned(),
                    })
                }
            }
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let (variant, values) = if let Index::None = self.current_field {
            let val = self
                .read
                .get_attribute_value(&Index::String(String::from("___enum_tag")))
                .ok_or_else(|| Error {
                    message: "Missing enum tag field".to_owned(),
                })?
                .clone();
            let variant = if let AttributeValue::S(val) = val {
                Ok(val)
            } else {
                Err(Error {
                    message: "Missing enum tag value".to_owned(),
                })
            }?;
            let values = self
                .read
                .get_attribute_value(&Index::String(String::from("___enum_values")))
                .and_then(|v| {
                    if let AttributeValue::M(val) = v {
                        Some(val.clone())
                    } else {
                        None
                    }
                });
            (variant, values)
        } else {
            let enum_field = self
                .read
                .get_attribute_value(&self.current_field)
                .ok_or_else(|| Error {
                    message: format!("missing enum for field {:?}", &self.current_field),
                })?;
            if let AttributeValue::S(shortstyle) = enum_field.clone() {
                return visitor.visit_enum(shortstyle.into_deserializer());
            }
            let base = if let AttributeValue::M(val) = enum_field.clone() {
                Ok(val)
            } else {
                Err(Error {
                    message: "Missing enum data".to_owned(),
                })
            }?;
            let enum_tag = if let AttributeValue::S(val) = base.get("___enum_tag").unwrap() {
                Ok(val.clone())
            } else {
                Err(Error {
                    message: "Missing enum tag value".to_owned(),
                })
            }?;

            (
                enum_tag,
                base.get("___enum_values").and_then(|v| {
                    if let AttributeValue::M(val) = v {
                        Some(val.clone())
                    } else {
                        None
                    }
                }),
            )
        };
        let mut des = Deserializer::new(HashMapRead::new(values.unwrap_or_else(HashMap::new)));
        visitor.visit_enum(EnumAccess::new(&mut des, variant))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.current_field {
            Index::String(ref value) => visitor.visit_str(&value.clone()),
            _ => Err(Error {
                message: "indentifier should be a string".to_string(),
            }),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct EnumAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
    variant_name: String,
}
impl<'a, R: 'a> EnumAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, variant_name: String) -> Self {
        EnumAccess { de, variant_name }
    }
}
impl<'de, 'a, R: Read + 'a> serde::de::EnumAccess<'de> for EnumAccess<'a, R> {
    type Error = Error;
    type Variant = VariantAccess<'a, R>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(self.variant_name.into_deserializer())?;
        Ok((value, VariantAccess::new(self.de)))
    }
}

struct VariantAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
}
impl<'a, R: 'a> VariantAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        VariantAccess { de }
    }
}
impl<'de, 'a, R: Read + 'a> serde::de::VariantAccess<'de> for VariantAccess<'a, R> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        self.de.current_field = Index::String(String::from("_0"));
        seed.deserialize(&mut *self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(TupleAccess::new(self.de))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_map(MapAccess::new(self.de, self.de.read.get_keys()))
    }
}

struct TupleAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
    current: usize,
}
impl<'a, R: 'a> TupleAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        TupleAccess { de, current: 0 }
    }
}
impl<'de, 'a, R: Read + 'a> serde::de::SeqAccess<'de> for TupleAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        self.de.current_field = Index::String(format!("_{}", self.current));
        self.current += 1;
        if self
            .de
            .read
            .get_attribute_value(&self.de.current_field)
            .is_none()
        {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct SeqAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
    current: usize,
}

impl<'a, R: 'a> SeqAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        SeqAccess { de, current: 0 }
    }
}

impl<'de, 'a, R: Read + 'a> serde::de::SeqAccess<'de> for SeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        self.de.current_field = Index::Number(self.current);
        self.current += 1;
        if self
            .de
            .read
            .get_attribute_value(&self.de.current_field)
            .is_none()
        {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct MapAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
    keys: Vec<String>,
    current: usize,
}

impl<'a, R: 'a> MapAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, keys: Vec<String>) -> Self {
        MapAccess {
            de,
            keys,
            current: 0,
        }
    }
}

impl<'de, 'a, R: Read + 'a> serde::de::MapAccess<'de> for MapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.current >= self.keys.len() {
            Ok(None)
        } else {
            self.de.current_field = Index::String(self.keys[self.current].to_string());
            self.de.as_key = true;
            self.current += 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        self.de.as_key = false;
        seed.deserialize(&mut *self.de)
    }
}

fn from_trait<'de, R, T>(read: R) -> Result<T>
where
    R: Read,
    T: serde::de::Deserialize<'de>,
{
    let mut de = Deserializer::new(read);
    let value = serde::de::Deserialize::deserialize(&mut de)?;

    Ok(value)
}

/// Deserialize an instance of type `T` from an `HashMap<String, AttributeValue>`.
///
/// # Errors
///
/// This conversion can fail if the structure of the input does not match the
/// structure expected by `T`, for example if `T` is a struct type but the input
/// contains something other than a JSON map. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the JSON map or some number is too big to fit in the expected primitive
/// type.
pub fn from_hashmap<'a, T, S: ::std::hash::BuildHasher + Clone>(
    hm: HashMap<String, AttributeValue, S>,
) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    from_trait(HashMapRead::new(hm))
}
