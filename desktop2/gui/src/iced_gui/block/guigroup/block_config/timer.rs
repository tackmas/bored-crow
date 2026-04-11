use std::{
    time::{
        Duration,
    }
};

use iced::{
    self,
    alignment::{
        Horizontal,
    },
    Length,
    Element,
    widget::{
        center,
        Column,
        PickList,
        Row,
        Text,
        TextInput,
    },
};


#[derive(Clone)]
pub enum Message {
    SelectTimeUnit(TimeUnit),
    TextInputChanged(String),
}

#[derive(Clone, Copy, PartialEq)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
}

impl TimeUnit {
    const TIME_UNITS: [TimeUnit; 5] = [
        TimeUnit::Seconds,
        TimeUnit::Minutes,
        TimeUnit::Hours,
        TimeUnit::Days,
        TimeUnit::Weeks,
    ];

    pub fn duration(&self, num: u64) -> Duration {
        match self {
            TimeUnit::Seconds => Duration::from_secs(num),
            TimeUnit::Minutes => Duration::from_mins(num),
            TimeUnit::Hours => Duration::from_hours(num),
            TimeUnit::Days => Duration::from_hours(num * 24),
            TimeUnit::Weeks => Duration::from_hours(num * 24 * 7),
        }
    }
}

impl std::fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = 
            match self {
                TimeUnit::Seconds => "Seconds",
                TimeUnit::Minutes => "Minutes",
                TimeUnit::Hours => "Hours",
                TimeUnit::Days => "Days",
                TimeUnit::Weeks => "Weeks",
            };

        write!(f, "{s}")
    }
}

pub struct Timer {
    text_input: String,
    time_unit: TimeUnit,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            text_input: String::new(),
            time_unit: TimeUnit::Seconds,
        }
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SelectTimeUnit(time_unit) => self.time_unit = time_unit,
            Message::TextInputChanged(text_input) => self.text_input = text_input,
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        let selected_time_unit = 
            PickList::new(
                TimeUnit::TIME_UNITS, 
                Some(self.time_unit), 
                Message::SelectTimeUnit
            );

        let instruction = Text::new("Enter a positive integer, with your selected time unit");
        let text_input = TextInput::new("Enter how long you want to block the group", &self.text_input)
            .on_input(Message::TextInputChanged);

        let row = 
            Row::with_children([
                text_input.into(),
                selected_time_unit.into()
            ]);

        let column = 
            Column::with_children([
                instruction.into(),
                row.into(),
            ])
            .align_x(Horizontal::Left);
        
        center(column).into()
    }

    pub fn try_duration(&self) -> Result<Duration, &'static str> {
        let parse_result = self
            .text_input
            .parse::<u64>();

        match parse_result {
            Ok(duration) => Ok(self.time_unit.duration(duration)),
            Err(_) => Err("The input must be a valid positive integer!")
        } 
    }
}