use std::{
    fmt::{
        Display,
        Result,
    },
    time::{
        Duration,
    }
};

use iced::{
    self,
    alignment::{
        Horizontal,
    },
    Element,
    widget::{
        Column,
        PickList,
        Row,
        Text,
        TextInput,
    },
};

#[derive(Clone)]
pub enum TimerMessage {
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

impl Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        let s = match self {
            TimeUnit::Seconds => "Seconds",
            TimeUnit::Minutes => "Minutes",
            TimeUnit::Hours => "Hours",
            TimeUnit::Days => "Days",
            TimeUnit::Weeks => "Weeks",
        };

        write!(f, "{s}")
    }
}

pub struct TimerState {
    text_input: String,
    time_unit: TimeUnit,
}

impl TimerState {
    pub fn new() -> Self {
        todo!();
    }
    pub fn update(&mut self, message: TimerMessage) {
        match message {
            TimerMessage::SelectTimeUnit(time_unit) => self.time_unit = time_unit,
            TimerMessage::TextInputChanged(text_input) => self.text_input = text_input,
        }
    }
    pub fn view(&self) -> Element<'_, TimerMessage> {
        let selected_time_unit = 
            PickList::new(
                TimeUnit::TIME_UNITS, 
                None::<TimeUnit>, 
                TimerMessage::SelectTimeUnit
            );

        let instruction = Text::new("Enter a positive integer, with your selected time unit");
        let text_input = TextInput::new("Enter how long you want to block the group", &self.text_input)
            .on_input(TimerMessage::TextInputChanged);

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
        
        column.into()
    }

    pub fn duration(&self) -> Duration {
        let duration_num = self
            .text_input
            .parse::<u64>()
            .expect("input must be a positive integer");

        self.time_unit.duration(duration_num)
    }
}