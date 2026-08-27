
use std::ops::{Index, IndexMut};

use chrono::{Weekday, WeekdaySet};

use serde::{Deserialize, Serialize};

use super::{TimeRange, WeekScheduleT};
/* 

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CustomWeek {
    pub monday: CustomWeekday,
    pub tuesday: CustomWeekday,
    pub wednesday: CustomWeekday,
    pub thursday: CustomWeekday,
    pub friday: CustomWeekday,
    pub saturday: CustomWeekday,
    pub sunday: CustomWeekday,
}

impl Index<u8> for CustomWeek {
    type Output = CustomWeekday;

    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0 => &self.monday,
            1 => &self.tuesday,
            2 => &self.wednesday,
            3 => &self.thursday,
            4 => &self.friday,
            5 => &self.saturday,
            6 => &self.sunday,
            _ => panic!("Index must be between the range 0-6 (inclusive)")
        }
    }
}

impl IndexMut<u8> for CustomWeek {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        match index {
            0 => &mut self.monday,
            1 => &mut self.tuesday,
            2 => &mut self.wednesday,
            3 => &mut self.thursday,
            4 => &mut self.friday,
            5 => &mut self.saturday,
            6 => &mut self.sunday,
            _ => panic!("Index must be between the range 0-6 (inclusive)")
        }        
    }
}

impl Index<Weekday> for CustomWeek {
    type Output = CustomWeekday;

    fn index(&self, index: Weekday) -> &Self::Output {
        match index {
            Weekday::Mon => &self.monday,
            Weekday::Tue => &self.tuesday,
            Weekday::Wed => &self.wednesday,
            Weekday::Thu => &self.thursday,
            Weekday::Fri => &self.friday,
            Weekday::Sat => &self.saturday,
            Weekday::Sun => &self.sunday,
        }        
    }
}

impl IndexMut<Weekday> for CustomWeek {
    fn index_mut(&mut self, index: Weekday) -> &mut Self::Output {
        match index {
            Weekday::Mon => &mut self.monday,
            Weekday::Tue => &mut self.tuesday,
            Weekday::Wed => &mut self.wednesday,
            Weekday::Thu => &mut self.thursday,
            Weekday::Fri => &mut self.friday,
            Weekday::Sat => &mut self.saturday,
            Weekday::Sun => &mut self.sunday,
        }          
    }
}

impl WeekScheduleT for CustomWeek {
    fn enabled_weekdays(&self) -> WeekdaySet {
        let arr = [
            &self.monday,
            &self.tuesday,
            &self.wednesday,
            &self.thursday,
            &self.friday,
            &self.saturday,
            &self.sunday,
        ];

        arr.into_iter()
            .enumerate()
            .filter(|(_, weekday)| weekday.enabled)
            .map(|(i, _)| Weekday::try_from(i as u8).unwrap())
            .collect()
    }
    fn time_range_on_weekday(&self, weekday: Weekday) -> Option<&TimeRange> {
        self[weekday]
            .enabled_time_range()      
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomWeekday {
    pub time_range: TimeRange,
    pub enabled: bool,
}

impl CustomWeekday {
    pub fn enabled_time_range(&self) -> Option<&TimeRange> {
        self.enabled.then_some(&self.time_range)
    }
    pub fn enabled_time_range_mut(&mut self) -> Option<&mut TimeRange> {
        self.enabled.then_some(&mut self.time_range)
    }
}

impl Default for CustomWeekday {
    fn default() -> Self {
        Self {
            time_range: TimeRange::min(),
            enabled: false
        }
    }
}

*/

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CustomWeek {
    time_ranges_on_week: TimeRangesOnWeek,
    #[serde(with = "super::weekday_set_serde")]
    enabled_weekdays: WeekdaySet,
}

impl CustomWeek {
    pub fn from_parts(time_ranges_on_week: TimeRangesOnWeek, enabled_weekdays: WeekdaySet) -> Self {
        Self {
            time_ranges_on_week,
            enabled_weekdays
        }
    }
    pub fn time_range_on_weekday(&self, weekday: Weekday) -> Option<&TimeRange> {
        self.enabled_weekdays
            .contains(weekday)
            .then_some(&self.time_ranges_on_week[weekday])
    }
    pub fn time_range_on_weekday_mut(&mut self, weekday: Weekday) -> Option<&mut TimeRange> {
        self.enabled_weekdays
            .contains(weekday)
            .then_some(&mut self.time_ranges_on_week[weekday])
    }
    pub fn enable_weekday(&mut self, weekday: Weekday) {
        self.enabled_weekdays.insert(weekday);
    }
    pub fn disable_weekday(&mut self, weekday: Weekday) {
        self.enabled_weekdays.remove(weekday);
    }
    pub fn weekday_is_enabled(&self, weekday: Weekday) -> bool {
        self.enabled_weekdays.contains(weekday)
    }
}

impl WeekScheduleT for CustomWeek {
    fn enabled_weekdays(&self) -> WeekdaySet {
        self.enabled_weekdays
    }
    fn time_range_on_weekday(&self, weekday: Weekday) -> Option<&TimeRange> {
        self.time_range_on_weekday(weekday)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TimeRangesOnWeek{
    on_monday: TimeRange,
    on_tuesday: TimeRange,
    on_wednesday: TimeRange,
    on_thursday: TimeRange,
    on_friday: TimeRange,
    on_saturday: TimeRange,
    on_sunday: TimeRange,
}

impl Index<Weekday> for TimeRangesOnWeek {
    type Output = TimeRange;

    fn index(&self, weekday: Weekday) -> &Self::Output {
        match weekday {
            Weekday::Mon => &self.on_monday,
            Weekday::Tue => &self.on_tuesday,
            Weekday::Wed => &self.on_wednesday,
            Weekday::Thu => &self.on_thursday,
            Weekday::Fri => &self.on_friday,
            Weekday::Sat => &self.on_saturday,
            Weekday::Sun => &self.on_sunday,
        }
    }
}

impl IndexMut<Weekday> for TimeRangesOnWeek {
    fn index_mut(&mut self, weekday: Weekday) -> &mut Self::Output {
        match weekday {
            Weekday::Mon => &mut self.on_monday,
            Weekday::Tue => &mut self.on_tuesday,
            Weekday::Wed => &mut self.on_wednesday,
            Weekday::Thu => &mut self.on_thursday,
            Weekday::Fri => &mut self.on_friday,
            Weekday::Sat => &mut self.on_saturday,
            Weekday::Sun => &mut self.on_sunday,
        }
    }
}
