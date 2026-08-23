use std::fmt;
use std::num::IntErrorKind;
use std::str::Utf8Error;

use chrono::{NaiveTime, Timelike, Weekday, WeekdaySet};

use iced::{Background, Color, Element, Length, Padding};
use iced::advanced::Widget;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{
    checkbox,
    Column, column,
    Container, container,
    opaque,
    pick_list,
    Row, row,
    space,
    Stack, stack,
    Text, text,
    text_input,
    tooltip,
};

use crate::make_semi_transparent;
use crate::core::block::time_range::{self, CustomWeekday, CustomWeekdays, UniformWeekdays, WeekdayRuleMode};
use crate::gui::state::grid::Grid;
use crate::gui::state::{bold_text, DARK_BEIGE, LENGTH_UNIT, Pad, RedBackground, semi_bold_text};
use crate::gui::state::modal::{horizontal_seperator, horizontal_seperator2, radio_with_border, title};

#[derive(Clone, Copy)]
enum Field {
    Hour,
    Minute,
}

#[derive(Clone, Copy)]
enum Bound {
    Start,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Custom,
    Uniform,
}

impl Mode {
    const ALL: [Mode; 2] = [Mode::Custom, Mode::Uniform];

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

#[derive(Clone)]
pub enum Message {
    ModeSelected(Mode),
    ToggleWeekday(Weekday, bool),
    TimeInputCustom(Weekday, Bound, Field, u8),
    TimeInputUniform(Bound, Field, u8),
    InvalidInput(String),
}

pub struct State {
    mode: Mode,
    uniform: UniformWeekdays,
    custom: CustomWeekdays,
}

// Constructor
impl State {
    pub fn new() -> Self {
        let uniform = {
            UniformWeekdays {
                time_range: time_range::TimeRange::min(),
                weekdays: WeekdaySet::single(Weekday::Mon),
            }
        };

        let custom = CustomWeekdays::default();

        Self {
            mode: Mode::Uniform,
            uniform,
            custom
        }
    }
}

// Update
impl State {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ModeSelected(mode) => self.mode = mode,
            Message::ToggleWeekday(weekday, flag) => {
                self.toggle_weekday(weekday, flag);
            },
            
            Message::TimeInputCustom(weekday, bound, field, input) => {
                let time_range = self.custom[weekday]
                    .enabled_time_range_mut()
                    .expect("Should always be Some(...) if user is able to input");

                change_time_range(time_range, bound, field, input);
            }
            Message::TimeInputUniform(bound, field, input) => {
                change_time_range(&mut self.uniform.time_range, bound, field, input);
            }
            Message::InvalidInput(input) => {
                eprintln!("Invalid input! (\"{input}\")");
            }
        }
    }

    fn toggle_weekday(&mut self, weekday: Weekday, flag: bool) -> Result<(), &'static str> {
        match self.mode {
            Mode::Uniform => {
                let weekdays = &mut self.uniform.weekdays;

                let is_single_day = weekdays.single_day().is_some();

                if !flag && is_single_day {
                    return Err("At least one weekday must be selected for any meaningful effect");
                }

                if flag {
                    weekdays.insert(weekday);
                } else {
                    weekdays.remove(weekday);
                } 

                Ok(())
            }
            Mode::Custom => {
                let custom_weekday = &mut self.custom[weekday];

                custom_weekday.enabled = flag;
                
                Ok(())
            }
        }
    }
}


// Helper functions for update
fn change_time_range(
    time_range: &mut time_range::TimeRange,
    bound: Bound,
    field: Field,
    input: impl Into<u32>,
) {
    let input = input.into();

    match bound {
        Bound::Start => change_clock(&mut time_range.start.time(), field, input),
        Bound::End => change_clock(&mut time_range.end.time(), field, input),
    }
}

fn change_clock(time: &mut NaiveTime, field: Field, input: u32) {
    match field {
        Field::Hour => {
            *time = NaiveTime::from_hms_opt(input, time.minute(), 0)
                .expect("Input (hour) should always be valid from earlier error handling");
        }
        Field::Minute => {
            *time = NaiveTime::from_hms_opt(time.hour(), input, 0)
                .expect("Input (minute) should always be valid from earlier error handling")
        }
    }
}

// View
impl State {
    pub fn view(&self) -> Element<'_, Message> {
        let mode_selection = self.mode_selection()
            .width(Length::Fill)
            .height(Length::Fill);

        let mode_view = match self.mode {
            Mode::Custom => self.custom_mode2(),
            Mode::Uniform => self.uniform_mode(),
        };

        let mode_view = container(mode_view)
            .width(Length::Fill)
            .height(Length::Fill);

        row![mode_selection, mode_view].into()
    }

    fn mode_selection(&self) -> Column<'_, Message> {
        let title_with_tooltip = {
            let title = title("Mode", 20.0).align_x(Horizontal::Left);
            let tooltip = container(tooltip(
                "🛈",
                container(
                    "Custom Mode allows you to have different time ranges on different days,
wheras Uniform Mode has the same time range on all days",
                ),
                tooltip::Position::Right,
            ));

            row![title, tooltip].align_y(Vertical::Center)
        };

        let radio_list = {
            let mode_radio = |text: &'static str, to| {
                radio_with_border(text, to, Some(self.mode), Message::ModeSelected)
            };
            let uniform_radio = mode_radio("Uniform", Mode::Uniform);
            let custom_radio = mode_radio("Custom", Mode::Custom);          

            column![uniform_radio, custom_radio]
        };

        column![title_with_tooltip, radio_list]
    }

    fn uniform_mode(&self) -> Element<'_, Message> {
        let weekday_elements = weekdays(|weekday| {
            let is_toggled = self.uniform.weekdays.contains(weekday);
            weekday_checkbox(weekday, is_toggled)
                .into()
        });

        let weekdays_x_list = Row::with_children(weekday_elements)
            .align_y(Vertical::Center)
            .spacing(*LENGTH_UNIT * 1.25)
            .pad(*LENGTH_UNIT);

        let horizontal_seperator = horizontal_seperator(100.0);

        let time_range_pick_list = {
            let on_selected = |bound, field, selected| Message::TimeInputUniform(bound, field, selected);

            time_range_pick_list(&self.uniform.time_range, on_selected, true).pad(*LENGTH_UNIT)
        };

        column![
            weekdays_x_list,
            horizontal_seperator,
            time_range_pick_list
        ]
        .align_x(Horizontal::Center)
        .into()
    }

    fn custom_mode(&self) -> Element<'_, Message> {
        let mut weekday_elements = column![];

        let start_end_header = {
            let start = semi_bold_text("Start");
            let end = semi_bold_text("End");

            row![start, end, space().width(Length::Fill)]
                .spacing(*LENGTH_UNIT)
        };

        weekday_elements = weekday_elements.extend([start_end_header.into(), horizontal_seperator2().into()]);

        for weekday in WeekdaySet::ALL.iter(Weekday::Mon) {
            let custom_weekday = &self.custom[weekday];
            let is_toggled = custom_weekday.enabled;

            let weekday_checkbox = weekday_checkbox(weekday, is_toggled)
                .align_y(Vertical::Top);

            let weekday_time_range_selection = weekday_time_range_selection(custom_weekday, weekday);

            let weekday_element = row![weekday_checkbox, weekday_time_range_selection]
                .align_y(Vertical::Center)
                .spacing(*LENGTH_UNIT)
                .pad_x(*LENGTH_UNIT);

            weekday_elements = weekday_elements.push(weekday_element);

            if weekday != Weekday::Sun {
                let horizontal_seperator = horizontal_seperator2();

                weekday_elements = weekday_elements.push(horizontal_seperator);
            }
        };

        weekday_elements
            .width(Length::Shrink)
            .align_x(Horizontal::Right)
            .spacing(*LENGTH_UNIT)
            .into()
    }

    fn custom_mode2(&self) -> Element<'_, Message> {
        let padding = Padding::ZERO.right(*LENGTH_UNIT);

        let headers: [Element<_>; 4] = {
            let start_time_range = semi_bold_text("Start").pad(padding).into();
            let end_time_range = semi_bold_text("End");

            // println!("{:?}", <Text<'_> as Widget<Message, _, _>>::state(&end_time_range));

            [space().into(), start_time_range, space().into(), end_time_range.into()]
        };

        let mut grid: Grid<'_, Message> = Grid::with(headers, 4);
            
        grid.align_row(0, Vertical::Center);

        for (i, weekday) in WeekdaySet::ALL.iter(Weekday::Mon).enumerate() {
            let custom_weekday = &self.custom[weekday];
            let is_toggled = custom_weekday.enabled;

            let weekday_checkbox = weekday_checkbox(weekday, is_toggled)
                .align_y(Vertical::Center)
                .padding(padding)
                .into();

            let on_selected = move |bound, field, input| {
                Message::TimeInputCustom(weekday, bound, field, input)
            };

            let [mut start, mut hyphen_seperator, mut end] = 
                time_range_pick_list2(&custom_weekday.time_range, on_selected);

            if !custom_weekday.enabled {
                (start, hyphen_seperator, end) = make_semi_transparent!(
                    start, hyphen_seperator, end; Element<_>
                );
            }

            let start = start.pad(padding).into();
            let hyphen_seperator = hyphen_seperator.pad(padding).into();

            let weekday = [weekday_checkbox, start, hyphen_seperator, end];

            grid.push_row(weekday)
                .align_row(i + 1, Vertical::Center);
        };

        grid
            .set_spacing_y(*LENGTH_UNIT)
            .align_column(0, Horizontal::Right)
            .align_column(1, Horizontal::Center)
            .align_column(3, Horizontal::Center);

        grid.into()
    }

    fn select_mode(&self) -> Element<'_, Message> {
        let title = "Mode";

        let mode = pick_list(Mode::ALL, Some(&self.mode), Message::ModeSelected);

        let mode_tooltip = tooltip(
            "🛈",
            container(
                "Custom Mode allows you to have different time ranges on different days,
wheras Uniform Mode has the same time range on all days",
            ),
            tooltip::Position::Right,
        );

        column![title, row![mode, mode_tooltip]].into()
    }
}

// Helper functions for view
const fn weekday_to_str(weekday: Weekday) -> &'static str {
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

const HOURS_IN_DAY: [&str; 24] = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23"];
const MINUTES_IN_HOUR: [&str; 60] = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59"];


fn weekdays<'a, T>(
    f: impl Fn(Weekday) -> T
) -> impl Iterator<Item = T> 
{
    WeekdaySet::ALL.iter(Weekday::Mon).map(f)
}

fn weekday_checkbox<'a>(weekday: Weekday, is_toggled: bool) -> Row<'a, Message> {
    let weekday_text = semi_bold_text(weekday_to_str(weekday));

    let checkbox = checkbox(is_toggled)
        .on_toggle(move |toggle| Message::ToggleWeekday(weekday, toggle));

    row![weekday_text, checkbox]
        .align_y(Vertical::Center)
        .spacing(*LENGTH_UNIT / 2.0)
}
/* 

    let element_with_title = |
        clock_pick_lists: Element<'a, _>, 
        description: &'static str
    | -> Column<'a, _>
    {
        column![
            semi_bold_text(description),
            clock_pick_lists
        ]
        .align_x(Horizontal::Center)
        .spacing(*LENGTH_UNIT * 0.9)
    };

*/

fn titled_with_seperator<'a>(to_title: impl Into<Element<'a, Message>>, title_text: &'a str) -> Column<'a, Message> {
    column![
        semi_bold_text(title_text),
        horizontal_seperator2(),
        to_title.into()
    ]
    .align_x(Horizontal::Center)
}

fn time_range_pick_list2<'a>(
    time_range: &time_range::TimeRange,
    on_selected: impl Fn(Bound, Field, u8) -> Message + Clone + 'a,
) -> [Element<'a, Message>; 3]
{
    let start_input = {
        let start_hour = time_range.start.time().hour() as u8;
        let start_min = time_range.start.time().minute() as u8;

        let on_start_hour_selected = on_selected.clone();
        let on_start_min_selected = on_selected.clone();

        clock_pick_list(
            start_hour,
            start_min,
            move |selected| on_start_hour_selected(Bound::Start, Field::Hour, selected),
            move |selected| on_start_min_selected(Bound::Start, Field::Minute, selected),
        )
        .into()
    };

    let hyphen_seperator = bold_text("-").into();

    let end_input = {
        let end_hour = time_range.end.time().hour() as u8;
        let end_min = time_range.end.time().minute() as u8;

        let on_end_hour_selected = on_selected.clone();
        let on_end_min_selected = on_selected;

        clock_pick_list(
            end_hour,
            end_min,
            move |input| on_end_hour_selected(Bound::End, Field::Hour, input),
            move |input| on_end_min_selected(Bound::End, Field::Minute, input),
        )
        .into()
    };

    [start_input, hyphen_seperator, end_input]
}
fn weekday_time_range_selection<'a>(custom_weekday: &CustomWeekday, weekday: Weekday) -> Element<'a, Message> {
    let time_range = &custom_weekday.time_range;

    let on_selected = move |bound, field, input| {
        Message::TimeInputCustom(weekday, bound, field, input)
    };

    let time_range_pick_list = time_range_pick_list(time_range, on_selected, false);

    if custom_weekday.enabled {
        time_range_pick_list.into()
    } else {
        let semi_transparent = container(
            space().width(Length::Fill).height(Length::Fill)
        )                           
        .style(|_theme| {
            use container::Style;

            Style {
                background: Some(
                    Background::Color(Color {
                        a: 0.4,
                        ..Color::WHITE
                    })
                ),
                ..Style::default()
            }
        });

        let opaque = opaque(semi_transparent);

        stack![
            time_range_pick_list,
            opaque
        ]
        .into()
    }
}

fn time_range_pick_list<'a>(
    time_range: &time_range::TimeRange,
    on_selected: impl Fn(Bound, Field, u8) -> Message + Clone + 'a,
    with_title: bool
) -> Element<'a, Message>
{
    let maybe_titled = |
        clock_pick_list: Element<'a, _>, 
        title: &'static str
    | -> Element<'_, Message>
    {
        if with_title {
            column![
                semi_bold_text(title),
                clock_pick_list
            ]
            .align_x(Horizontal::Center)
            .spacing(*LENGTH_UNIT * 0.9)
            .into()          
        } else {
            clock_pick_list
        }
    };

    let (start_clock, end_clock) = {
        let (start_clock, end_clock) = two_clock_pick_lists(time_range, on_selected);

        (maybe_titled(start_clock, "Start"), maybe_titled(end_clock, "End"))
    };

    let hyphen_seperator = maybe_titled(bold_text("-").into(), "");

    row![start_clock, hyphen_seperator, end_clock]
        .align_y(Vertical::Center)
        .spacing(*LENGTH_UNIT)
        .into()    
}

fn two_clock_pick_lists<'a>(
    time_range: &time_range::TimeRange,
    on_selected: impl Fn(Bound, Field, u8) -> Message + Clone + 'a,
) -> (Element<'a, Message>, Element<'a, Message>) 
{
    let start_clock = {
        let start_hour = time_range.start.time().hour() as u8;
        let start_min = time_range.start.time().minute() as u8;

        let on_start_hour_selected = on_selected.clone();
        let on_start_min_selected = on_selected.clone();

        clock_pick_list(
            start_hour,
            start_min,
            move |selected| on_start_hour_selected(Bound::Start, Field::Hour, selected),
            move |selected| on_start_min_selected(Bound::Start, Field::Minute, selected),
        )
    };

    let end_clock = {
        let end_hour = time_range.end.time().hour() as u8;
        let end_min = time_range.end.time().minute() as u8;

        let on_end_hour_selected = on_selected.clone();
        let on_end_min_selected = on_selected;

        clock_pick_list(
            end_hour,
            end_min,
            move |input| on_end_hour_selected(Bound::End, Field::Hour, input),
            move |input| on_end_min_selected(Bound::End, Field::Minute, input),
        )
    };

    (start_clock, end_clock)
}

fn clock_pick_list<'a>(
    current_hour: u8, 
    current_min: u8,
    on_hour_selected: impl Fn(u8) -> Message + 'a,
    on_min_selected: impl Fn(u8) -> Message + 'a
) -> Element<'a, Message> 
{
    let parse = |s: &str| s.parse::<u8>().expect("´s´ is always a integer within bounds 0-255");

    let hour_pick_list = pick_list(
        HOURS_IN_DAY, 
        Some(HOURS_IN_DAY[current_hour as usize]), 
        move |s| on_hour_selected(parse(s))
    );
    let min_pick_list = pick_list(
        MINUTES_IN_HOUR, 
        Some(MINUTES_IN_HOUR[current_min as usize]), 
        move |s| on_min_selected(parse(s))
    );

    row![hour_pick_list, semi_bold_text(":"), min_pick_list]
        .align_y(Vertical::Center)
        .spacing(*LENGTH_UNIT / 3.0)
        .into()
}



// Other items
impl From<&State> for WeekdayRuleMode {
    fn from(state: &State) -> Self {
        match state.mode {
            Mode::Custom => WeekdayRuleMode::Custom(state.custom.clone()),
            Mode::Uniform => WeekdayRuleMode::Uniform(state.uniform.clone()),
        }
    }
}


fn test(
    time_range: &time_range::TimeRange, 
    on_selected: impl Fn(Bound, Field, u8) -> Message + Clone, 
    is_toggled: bool
) {
    let weekday_checkbox = weekday_checkbox(Weekday::Mon, is_toggled)
        .align_y(Vertical::Top);

    let (start_clock, end_clock) = {
        let (start_clock, end_clock) = two_clock_pick_lists(time_range, on_selected);

        (start_clock.pad_x(*LENGTH_UNIT * 0.5), end_clock.pad_x(*LENGTH_UNIT * 0.5))
    };

    let (start, end) = (
        titled_with_seperator(start_clock, "Start"),
        titled_with_seperator(end_clock, "End")
    );

    let hyphen_seperator = titled_with_seperator(bold_text("-"), "");

    let _time_range = row![weekday_checkbox, start, hyphen_seperator, end]
        .align_y(Vertical::Center)
        .spacing(*LENGTH_UNIT);
}




// Legacy stuff, in case if needed 

/* 
fn time_range_text_input<'a, Message: Clone + 'a>(
    time_range: &time_range::TimeRange,
    on_input: impl Fn(Bound, Field, u32) -> Message + Clone + 'a,
    on_err: impl Fn(String) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let start_input = {
        let start_hour = time_range.start.hour().to_string();
        let start_min = time_range.start.minute().to_string();

        let on_input_start_hour = on_input.clone();
        let on_input_start_min = on_input.clone();

        let on_err_start = on_err.clone();

        clock_text_input(
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

        clock_text_input(
            &end_hour,
            &end_min,
            move |input| on_input_end_hour(Bound::End, Field::Hour, input),
            move |input| on_input_end_min(Bound::End, Field::Minute, input),
            on_err_end,
        )
    };

    row![start_input, "to", end_input].into()
}

fn clock_text_input<'a, Message: Clone + 'a>(
    hour: &str,
    minute: &str,
    on_hour_input: impl Fn(u32) -> Message + 'a,
    on_min_input: impl Fn(u32) -> Message + 'a,
    on_err: impl Fn(String) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let on_err_hour = on_err.clone();

    let hour_input = text_input("", hour).on_input(move |input| {
        match input.parse::<u32>() {
            Ok(value) if value < 24 => on_hour_input(value),
            Err(e) if let IntErrorKind::Empty = e.kind() => on_hour_input(0),
            _error => on_err_hour(input),
        }
    });

    let on_err_min = on_err;

    let minute_input = text_input("", minute).on_input(move |input| {
        match input.parse::<u32>() {
            Ok(value) if value < 60 => on_min_input(value),
            Err(e) if let IntErrorKind::Empty = e.kind() => on_min_input(0),
            _error => on_err_min(input)
        }
    });

    row![hour_input, ":", minute_input,].into()
}
*/

fn semi_transperent<'a>(
    element: Element<'a, Message>,
) -> Stack<'a, Message>
{
    let semi_transparent_layer = container(
        space().width(Length::Fill).height(Length::Fill)
    )                           
    .style(|_theme| {
        use container::Style;

        Style {
            background: Some(
                Background::Color(Color {
                    a: 0.4,
                    ..Color::WHITE
                })
            ),
            ..Style::default()
        }
    });

    let opaque = opaque(semi_transparent_layer);

    stack![
        element,
        opaque
    ]
}