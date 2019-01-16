use std::str::FromStr;
use std::fmt;
use chrono::prelude::*;
use serde::de;
use chrono::FixedOffset;
use failure::Error;


#[derive(Debug)]
pub struct WMIDateTime(pub DateTime<FixedOffset>);

impl FromStr for WMIDateTime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (datetime_part, tz_part) = s.split_at(21);

        let tz_min: i32= tz_part.parse()?;

        let tz = FixedOffset::east(tz_min * 60);

        let dt = tz.datetime_from_str(datetime_part, "%Y%m%d%H%M%S.%f")?;

        Ok(Self(dt))
    }
}

struct DateTimeVisitor;

impl<'de> de::Visitor<'de> for DateTimeVisitor {
    type Value = WMIDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a formatted date and time string or a unix timestamp"
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        value.parse().map_err(|err| E::custom(format!("{}", err)))
    }
}

impl<'de> de::Deserialize<'de> for WMIDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(DateTimeVisitor)
    }
}

#[allow(non_camel_case_types)]
mod tests {
    use super::*;

    #[test]
    fn it_works_with_negative_offset() {
        let dt: WMIDateTime = "20190113200517.500000-180".parse().unwrap();

        assert_eq!(dt.0.to_rfc3339(), "2019-01-13T20:05:17.000500-03:00");
    }

    #[test]
    fn it_works_with_positive_offset() {
        let dt: WMIDateTime = "20190113200517.500000+060".parse().unwrap();

        assert_eq!(dt.0.to_rfc3339(), "2019-01-13T20:05:17.000500+01:00");
    }
}