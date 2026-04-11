use std::{
    fmt,

};

use chrono::{
    NaiveTime,
    Timelike,
    Weekday,
    WeekdaySet,
};

use iced::{
    Element,
    widget::{
        checkbox,
        Column, column,
        container,
        pick_list,
        Row,
        row,
        space,
        text,
        text_input,
        tooltip,
    }
};


use engine::{
    block_config::{
        time_range::{
            self,
            CustomWeekdays,
            UniformWeekdays,
            WeekdayRuleMode
        }
    }
};

#[derive(Clone, Copy)]
enum Field {
    Hour,
    Minute
}

#[derive(Clone, Copy)]
enum Bound {
    Start,
    End
}

#[derive(Clone)]
pub enum Message {
    ModeSelected(Mode),
    ToggleWeekday(bool, Weekday),
    TimeInputCustom(Weekday, Bound, Field, u32),
    TimeInputUniform(Bound, Field, u32),
    InvalidInput(String),
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Custom,
    Uniform,
}

impl Mode {
    const ALL: [Mode; 2] = [
        Mode::Custom,
        Mode::Uniform,
    ];

    const fn mode_str(self) -> &'static str {
        match self {
            Mode::Custom => "Custom",
            Mode::Uniform => "Uniform",
        }
    }
    
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = {
            match self {
                Mode::Custom => "Custom",
                Mode::Uniform => "Uniform",
            }
        };

        write!(f, "{s}")
    }
}

pub struct State {
    mode: Mode,
    custom: CustomWeekdays,
    uniform: UniformWeekdays,
}

impl State {
    pub fn new() -> Self {
        let custom = CustomWeekdays {
            monday: None,
            tuesday: None,
            wednesday: None,
            thursday: None,
            friday: None,
            saturday: None,
            sunday: None
        };

        let uniform = {
            UniformWeekdays {
                time_range: time_range::TimeRange::min(),
                weekdays: WeekdaySet::EMPTY
            }
        };

        State {
            mode: Mode::Uniform,
            custom,
            uniform
        }
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ModeSelected(mode) => self.mode = mode,
            Message::ToggleWeekday(toggle, weekday) => {
                match self.mode {
                    Mode::Custom => {
                        let time_range = self.custom.weekday_time_range_mut(weekday);

                        if toggle {
                            *time_range = Some(time_range::TimeRange::min());
                        } else {
                            *time_range = None;
                        }
                    },
                    Mode::Uniform => {
                        let weekdays = &mut self.uniform.weekdays;

                        if toggle {
                            weekdays.insert(weekday);
                        } else {
                            weekdays.remove(weekday);
                        }
                    }
                }

            },
            Message::TimeInputCustom(weekday, bound, field, input) => {
                let time_range = self.custom
                    .weekday_time_range_mut(weekday)
                    .as_mut()
                    .expect("should always be Some(...) if user is able to input");

                change_time_range(time_range, bound, field, input);
            },
            Message::TimeInputUniform(bound, field, input) => {
                change_time_range(&mut self.uniform.time_range, bound, field, input);
            },
            Message::InvalidInput(input) => {
                eprintln!("Invalid input! (\"{input}\")");
            }
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        let mode_view = match self.mode {
            Mode::Custom => self.custom_mode(),
            Mode::Uniform => self.uniform_mode(),
        };

        column![
            self.select_mode(),
            mode_view,
        ]
        .into()
    }
}

impl From<&State> for WeekdayRuleMode {
    fn from(state: &State) -> Self {
       match state.mode {
            Mode::Custom => WeekdayRuleMode::Custom(state.custom.clone()),
            Mode::Uniform => WeekdayRuleMode::Uniform(state.uniform.clone()),
       } 
    }
}

fn change_time_range(
    time_range: &mut time_range::TimeRange, 
    bound: Bound, 
    field: Field,
    input: u32
) {
    match bound {
        Bound::Start => change_clock(&mut time_range.start, field, input),
        Bound::End => change_clock(&mut time_range.end, field, input)
    }
}

fn change_clock(time: &mut NaiveTime, field: Field, input: u32) {
    match field {
        Field::Hour => {
            *time = NaiveTime::from_hms_opt(input, time.minute(), 0)
                .expect("input (hour) should always be valid from earlier error handling");
        },
        Field::Minute => {
            *time = NaiveTime::from_hms_opt(time.hour(), input, 0)
                .expect("input (minute) should always be valid from earlier error handling")
        }
    }
}

// Helper functions for view
impl State {
    fn custom_mode(&self) -> Element<'_, Message> {
        let weekdays_time_range_iter = WeekdaySet::ALL
            .iter(Weekday::Mon)
            .map(|weekday| {
                let option_time_range = self.custom.weekday_time_range(weekday);
                let is_toggled = option_time_range.is_some();

                let row = row![
                    weekday_str(weekday), 
                    checkbox(is_toggled)
                        .on_toggle(move |toggle| Message::ToggleWeekday(toggle, weekday))
                ];

                if is_toggled {
                    let time_range = option_time_range.unwrap();

                    return row.push(
                        enter_time_range(
                            &time_range, 
                            move |bound, field, input| Message::TimeInputCustom(weekday, bound, field, input), 
                            Message::InvalidInput,
                        )
                    )
                    .into()
                }

                row.into()
            });


        Column::with_children(weekdays_time_range_iter).into()      
    }

    fn uniform_mode(&self) -> Element<'_, Message> {
        let weekdays_iter = WeekdaySet::ALL
            .iter(Weekday::Mon)
            .map(|weekday| {
                let is_marked = self.uniform.weekdays.contains(weekday);

                row![
                    weekday_str(weekday), 
                    checkbox(is_marked)
                        .on_toggle(move |toggle| Message::ToggleWeekday(toggle, weekday))
                ]
                .into()
            });

        let weekdays_list = Column::with_children(weekdays_iter);

       row![
        weekdays_list, 
        enter_time_range(
            &self.uniform.time_range, 
            |bound, field, input| Message::TimeInputUniform(bound, field, input), 
            Message::InvalidInput
        )]
        .into()
    }

    fn select_mode(&self) -> Element<'_, Message> {
        let title = "Mode";

        let mode = pick_list(
            Mode::ALL, 
            Some(&self.mode), 
            Message::ModeSelected
        );

        let mode_tooltip = tooltip(
            "Info",
            container(
                    "Custom Mode allows you to have different time ranges on different days,
wheras Uniform Mode has the same time range on all days"
            ),
            tooltip::Position::Right
        );

        column![
            title,
            row![
                mode,
                mode_tooltip
            ]
        ]
        .into()
    }


}

const fn weekday_str(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
}

fn enter_time_range<'a, Message: Clone + 'a>(
    time_range: &time_range::TimeRange,
    on_input: impl Fn(Bound, Field, u32) -> Message + Clone + 'a,
    on_err: impl Fn(String) -> Message + Clone + 'a,
) -> Element<'a, Message> 
{
    let start_input = {
        let start_hour = time_range.start.hour().to_string();
        let start_min = time_range.start.minute().to_string();

        let on_input_start_hour = on_input.clone();
        let on_input_start_min = on_input.clone();

        let on_err_start = on_err.clone();

        enter_clock(
            &start_hour,
            &start_min,
            move |input| on_input_start_hour(Bound::Start, Field::Hour, input),
            move |input| on_input_start_min(Bound::Start, Field::Minute, input),
            on_err_start, 
        )
    };
    let end_input = {
        let end_hour = time_range.end.hour().to_string();
        let end_min = time_range.end.minute().to_string();

        let on_input_end_hour = on_input.clone();
        let on_input_end_min = on_input;

        let on_err_end = on_err;

        enter_clock(
            &end_hour,
            &end_min,
            move |input| on_input_end_hour(Bound::End, Field::Hour, input),
            move |input| on_input_end_min(Bound::End, Field::Minute, input),
            on_err_end, 
        )
    };

    row![start_input, "to", end_input].into()
}

fn enter_clock<'a, Message: Clone + 'a>(
    hour: &str, 
    minute: &str, 
    on_hour_input: impl Fn(u32) -> Message + 'a,
    on_min_input: impl Fn(u32) -> Message + 'a,
    on_err: impl Fn(String) -> Message + Clone + 'a
) -> Element<'a, Message> 
{
    let on_err_hour = on_err.clone();

    let hour_input = text_input("", hour)
        .on_input(move |input| {
            match input.parse::<u32>() {
                Ok(value) if value < 24 => on_hour_input(value),
                _error => on_err_hour(input)
            }
        });
    
    let on_err_min = on_err;

    let minute_input = text_input("", minute)
        .on_input(move |input| {
            match input.parse::<u32>() {
                Ok(value) if value < 60 => on_min_input(value),
                _error => on_err_min(input)
            }
        });

    row![
        hour_input,
        ":",
        minute_input,
    ]
    .into()
}
