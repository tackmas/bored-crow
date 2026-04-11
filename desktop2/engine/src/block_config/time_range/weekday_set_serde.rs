use chrono::{
    Weekday,
    WeekdaySet
};

use serde::{
    Deserialize,

    Serializer, 
    Deserializer,

    ser::SerializeSeq
};

pub fn serialize<S>(set: &WeekdaySet, serializer: S) -> Result<S::Ok, S::Error> 
where 
    S: Serializer
{
    let vec: Vec<Weekday> = set.iter(Weekday::Mon).collect();

    let mut seq = serializer.serialize_seq(Some(7))?;

    for weekday in vec {
        seq.serialize_element(&weekday)?;
    }
    seq.end()
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<WeekdaySet, D::Error>
where 
    D: Deserializer<'de>
{
    let vec = Vec::<Weekday>::deserialize(deserializer)?;
    Ok(WeekdaySet::from_iter(vec))    
}