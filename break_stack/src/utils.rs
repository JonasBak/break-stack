pub use break_stack_macros::bundle_files;

pub mod serde {
    use serde::de::IntoDeserializer;
    use serde::Deserialize;

    pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: serde::Deserialize<'de>,
    {
        let opt = Option::<String>::deserialize(de)?;
        let opt = opt.as_ref().map(String::as_str);
        match opt {
            None | Some("") => Ok(None),
            Some(s) => T::deserialize(s.into_deserializer()).map(Some),
        }
    }

    pub fn from_string_empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let opt = Option::<String>::deserialize(de)?;
        let opt = opt.as_ref().map(String::as_str);
        match opt {
            None | Some("") => Ok(None),
            Some(s) => T::from_str(s).map(Some).map_err(serde::de::Error::custom),
        }
    }

    pub fn from_string<'de, D, T>(de: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let string = String::deserialize(de)?;
        T::from_str(string.as_str()).map_err(serde::de::Error::custom)
    }

    pub fn vec_from_string<'de, D, T>(de: D) -> Result<Vec<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let v: Result<Vec<T>, _> = Vec::<String>::deserialize(de)?
            .into_iter()
            .map(|elem| T::from_str(&elem))
            .collect();
        v.map_err(serde::de::Error::custom)
    }
}

pub mod askama {
    pub mod filters {
        pub fn string_or_empty<T: ToString>(t: &Option<T>) -> ::askama::Result<String> {
            Ok(t.as_ref()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "".to_string()))
        }
        pub fn some_matches<T: PartialEq>(t: &Option<T>, a: &T) -> ::askama::Result<bool> {
            Ok(t.as_ref().map(|t| t == a).unwrap_or(false))
        }
        pub fn string_if_true<'a>(b: &'a bool, s: &'a str) -> ::askama::Result<&'a str> {
            Ok(b.then(|| s).unwrap_or(""))
        }
    }
}
