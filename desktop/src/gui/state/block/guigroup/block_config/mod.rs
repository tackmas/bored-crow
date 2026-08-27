mod lock_config;
mod time_range;
mod timer;

use std::{fmt, mem, sync::Arc};

use iced::{
    self, Background, Color, Element, Length, Task,
};
use iced::alignment::Horizontal::*;
use iced::widget::{
    Button, button, 
    Column, Container, Responsive, 
    Row, row, 
    Scrollable, Space, Stack, Text, text, center,
    container,
};

use lock_config::{self as l_c, LockConfig};
use time_range::GUITimeRange;
use timer::Timer;

use crate::{
    core::block::{time_range::WeekSchedule, timer::Timer as CoreTimer},
    platform::Blocker,
};

use crate::core::block::{BlockConfig as CoreBlockConfig, BlockRuleKind, Group, LockWhenBlocked};
use crate::core::block::block_rule::{LockConfig as CoreLockConfig};
use crate::core::block::time_range::WeekScheduleT;
use crate::gui::state::modal;
use crate::gui::state::{action, bold_text, button_with_text, DARK_BEIGE, LENGTH_UNIT, Pad, RedBackground, semi_bold_text};
use crate::unwrap_variant;

const COLOR_RED: Color = Color::from_rgb(1.0, 0.0, 0.0);

pub enum CustomAction {
    Close,
    Block(BlockRuleKind),
}

pub type CA = CustomAction;
pub type Action = action::Action<CA, Message>;

#[derive(Clone)]
pub enum Message {
    Close,
    Next,

    HideError,

    NavigateTo(Tab),

    LockConfig(lock_config::Message),
    Timer(timer::Message),
    TimeRange(time_range::Message),
}

#[derive(Debug)]
enum Screen {
    BlockConfig,
    LockConfig,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tab {
    Timer,
    TimeRange,
}

pub struct BlockConfig {
    screen: Screen,
    lock_config: LockConfig,
    tab: Tab,
    timer: Timer,
    time_range: GUITimeRange,
    error: Option<&'static str>,
}

impl BlockConfig {
    pub fn new() -> Self {
        BlockConfig {
            screen: Screen::BlockConfig,
            tab: Tab::Timer,
            lock_config: LockConfig::new(),
            timer: Timer::new(),
            time_range: GUITimeRange::new(),
            error: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Close => Action::none_with_custom(CustomAction::Close),
            Message::Next => {
                

                self.screen = Screen::LockConfig;

                Action::none()
            }
            Message::HideError => {
                self.error = None;

                Action::none()
            }
            Message::NavigateTo(tab) => {
                self.tab = tab;

                Action::none()
            }
            Message::LockConfig(msg) => {
                let action = self.lock_config.update(msg);

                self.handle_lock_config_action(action)
            }
            Message::Timer(msg) => {
                if let Tab::Timer = self.tab {
                    self.timer.update(msg);
                } else {
                    panic!(
                        "{} was emitted when screen is {:?}, and not {}",
                        "Message::Timer", self.screen, "Screen::Timer"
                    );
                }

                Action::none()
            }
            Message::TimeRange(msg) => {
                if let Tab::TimeRange = self.tab {
                    self.time_range.update(msg);
                } else {
                    panic!(
                        "{} was emitted when screen is {:?}, and not {}",
                        "Message::TimeRange", self.screen, "Screen::TimeRange"
                    );
                }

                Action::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::BlockConfig => self.block_config_view(),
            Screen::LockConfig => self.lock_config
                .view(self.tab)
                .map(Message::LockConfig),
        }
    }
}

// Helper functions for update
impl BlockConfig {
    /*fn block(&mut self, lock_when_blocked: bool) -> Action {
        let kind = match mut &self.screen {
            Screen::Timer(timer) => {
                let duration = match timer.try_duration() {
                    Ok(ok) => ok,
                    Err(error_str) => {
                        self.error = Some(error_str);

                        return Action::none()
                            .with_task(error_task());
                    }
                };

                BlockRuleKind::Timer(BlockTimer::new(duration))

            },
            Screen::TimeRange(state) => {
                BlockRuleKind::TimeRange(WeekdayRuleMode::from(&*state))
            }
        };


        Action::none_with_custom(CA::Block(kind, lock_when_blocked))
    }*/

    fn handle_lock_config_action(&mut self, mut action: l_c::Action) -> Action {
        match action.custom_opt.take() {
            None => action.with_custom(None),
            Some(l_c::CA::BackToBlockConfig) => {
                self.screen = Screen::BlockConfig;

                action.with_custom(None)
            }
            Some(l_c::CA::StartBlock(core_lock_config)) => {
                self.screen = Screen::BlockConfig;

                let block_rule_kind = match self.tab {
                    Tab::Timer => {
                        let duration = self.timer.try_duration().unwrap();
                        let timer = CoreTimer::new(duration);
                        let lock_when_blocked = LockWhenBlocked::from_lock_config(&core_lock_config);

                        BlockRuleKind::Timer { timer, lock_when_blocked }
                    }
                    Tab::TimeRange => {
                        self.time_range.into_block_rule_kind(core_lock_config)
                    }
                };

                action.with_custom(Some(CA::Block(block_rule_kind)))
            }
        }
        .map_task(Message::LockConfig)
    }

}

// Helper methods for view
impl BlockConfig {
    fn block_config_view(&self) -> Element<'_, Message> {
        let primary_title = modal::primary_title("Block configuration");

        let tab_content = // center(
            match self.tab {
                Tab::Timer => self.timer.view().map(Message::Timer),
                Tab::TimeRange => self.time_range.view().map(Message::TimeRange),
            };
       // )
        // .padding([50, 100]);

        Column::with_children([
            primary_title.into(),
            self.tab_buttons(),
            tab_content.into(),
            self.display_error(),
            self.footer_actions(),
        ])
        .into()
    }
    fn tab_buttons(&self) -> Element<'_, Message> {
        let tab_button = |tab_name, navigate_to_tab| {
            button(semi_bold_text(tab_name))  
                .on_press(Message::NavigateTo(navigate_to_tab))
                .style(move |_theme, _status| {
                    use button::Style;

                    let background = {
                        let color = if self.tab == navigate_to_tab {
                            // Beige
                            DARK_BEIGE
                        } else {
                            Color::BLACK
                        };

                        Background::Color(color)
                    };

                    Style {
                        background: Some(background),
                        text_color: Color::WHITE,
                        ..Style::default()
                    }
                })
        };

        let timer = tab_button("Timer", Tab::Timer);
        let time_range = tab_button("Time range", Tab::TimeRange);
            
        row![timer, time_range]
            .pad(*LENGTH_UNIT)
            .into()
    }

    fn display_error(&self) -> Element<'_, Message> {
        let error_str = match self.error {
            Some(error_str) => error_str,
            None => " ",
        };

        container(text(error_str).color(COLOR_RED).size(15))
            .center_x(Length::Fill)
            .into()
    }

    fn footer_actions(&self) -> Element<'_, Message> {
        Container::new(
            Row::with_children([
                button_with_text("Close", Message::Close),
                button_with_text("Next", Message::Next),
                Space::new().into(),
            ])
            .spacing(20),
        )
        .align_right(Length::Fill)
        .into()
    }
}

fn error_task() -> Task<Message> {
    Task::future(async {
        use tokio::time::{Duration, sleep};

        sleep(Duration::from_secs(3)).await;

        Message::HideError
    })
}

