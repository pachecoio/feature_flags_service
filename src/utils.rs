pub mod date_format {
    use chrono::{DateTime, Utc, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";
    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub mod mongo_id_format {
    use mongodb::bson::oid::ObjectId;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(
        id: &Option<ObjectId>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", id.unwrap().to_hex().to_string());
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<ObjectId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Option::from(ObjectId::parse_str(&s).unwrap()))
    }
}