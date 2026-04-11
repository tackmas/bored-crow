mod timer;
mod time_range;

use std::{
    fmt,
    mem,
    sync::{
        Arc,
    },
};

use iced::{
    self,
    alignment::{
        Horizontal::*,
    },
    Color,
    Element,
    Length,
    widget::{
        Button,
        Column,
        center,
        Container,
        container,
        Responsive,
        Row,
        Scrollable,
        Space,
        Stack,
        Text,
        text,
    },
    Task,
};

use timer::{
    Timer,
};

use engine::{
    block_config::{
        Group,
        BlockRule,
        BlockRuleKind,
        time_range::{
            WeekdayRuleMode,
        }
    },
};

use platform::{
    Blocker,
};

use crate::iced_gui::{
    button_with_text,
};

const COLOR_RED: Color = Color::from_rgb(1.0, 0.0, 0.0);

pub enum Instruction {
    None,
    Close,
    Block(Task<Message>),
    Task(Task<Message>),
}

impl Instruction {
    fn hide_error_after_task() -> Instruction {
        let task = 
            Task::future(async {
                use tokio::time;

                time::sleep(time::Duration::from_secs(3)).await;

                Message::HideError
            });

        Instruction::Task(task)
    }
    fn block_task(group: Arc<Group>, block_rule: BlockRule) -> Instruction {
        let task =
            Task::future(async move {
                group.block(block_rule).await;
            })
            .discard();
        
        Instruction::Block(task)
    }
}

#[derive(Clone, Copy)] 
enum Tab {
    Timer,
    TimeRange,
}

#[derive(Clone)]
pub enum Message {
    Close,

    BlockWithLock,
    BlockWithoutLock,

    HideError,

    NavigateTo(Tab),

    Timer(timer::Message),
    TimeRange(time_range::Message),
}

enum Screen {
    Timer(Timer),
    TimeRange(time_range::State),
}

impl From<Tab> for Screen {
    fn from(tab: Tab) -> Self {
        match tab {
            Tab::Timer => Screen::Timer(Timer::new()),
            Tab::TimeRange => Screen::TimeRange(time_range::State::new()),
        }
    }
}

impl PartialEq<Tab> for Screen {
    fn eq(&self, other: &Tab) -> bool {
        match (self, other) {
            (Screen::Timer(_), Tab::Timer) => true,
            (Screen::TimeRange(_), Tab::Timer) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let s = 
            match self {
                Screen::Timer(_) => "Screen::Timer",
                Screen::TimeRange(_) => "Screen::TimeRange",
            };

        write!(f, "{s}")
    }
}

pub struct BlockConfig {
    blocker: Blocker,
    group: Arc<Group>,
    screen: Screen,
    error: Option<&'static str>
}

impl BlockConfig {
    pub fn new(blocker: Blocker, group: Arc<Group>) -> Self {
        BlockConfig {
            blocker,
            group,
            screen: Screen::Timer(Timer::new()),
            error: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Instruction {
        match message {
            Message::Close => Instruction::Close,
            Message::BlockWithoutLock => self.block(false),
            Message::BlockWithLock => self.block(true),
            Message::HideError => {
                self.error = None;

                Instruction::None
            },
            Message::NavigateTo(tab) => {
                self.navigate_to(tab)
            },
            Message::Timer(msg) => {
                if let Screen::Timer(timer) = &mut self.screen {
                    timer.update(msg);
                } else {
                    panic!("{} was emitted when screen is {}, and not {}",
                    "Message::Timer", self.screen, "Screen::Timer")
                }

                Instruction::None
            },
            Message::TimeRange(msg) => {
                if let Screen::TimeRange(state) = &mut self.screen {
                    state.update(msg);
                } else {
                    panic!("{} was emitted when screen is {}, and not {}",
                    "Message::TimeRange", self.screen, "Screen::TimeRange")
                }

                Instruction::None
            }

        }
    }
    pub fn view(&self) -> Element<'_ , Message> {
        let tabs = Row::with_children([
            button_with_text("Timer", Message::NavigateTo(Tab::Timer)),
            button_with_text("Time Range", Message::NavigateTo(Tab::TimeRange))
        ]);

        let tab_content = 
            center(
                match &self.screen {
                    Screen::Timer(timer) => {
                        timer
                            .view()
                            .map(Message::Timer)
                    },
                    Screen::TimeRange(state) => {
                        state
                            .view()
                            .map(Message::TimeRange)
                    },

                }               
            )
            .padding([50, 100]);

        Column::with_children([
            tabs.into(),
            tab_content.into(),
            self.display_error(),
            finish_buttons(),
        ])
        .into()
    }

}

// Helper functions for update
impl BlockConfig {
    fn block(&mut self, lock_when_blocked: bool) -> Instruction {
        let kind = 
            match &mut self.screen {
                Screen::Timer(timer) => {
                    let duration =
                        match timer.try_duration() {
                            Ok(ok) => ok,
                            Err(error_str) => {
                                self.error = Some(error_str);

                                return Instruction::hide_error_after_task();
                            }
                        };

                    BlockRuleKind::Timer(duration)

                },
                Screen::TimeRange(state) => {
                    BlockRuleKind::TimeRange(WeekdayRuleMode::from(&*state))
                }
            };
        
        let blocker = self.blocker.clone();

        let block_rule = 
            BlockRule {
                blocker,
                kind,
                lock_when_blocked,
            };

        let group = self.group.clone();

        Instruction::block_task(group, block_rule)    
    }

    fn navigate_to(&mut self, tab: Tab) -> Instruction {
        if self.screen != tab {
            self.screen = Screen::from(tab);
        }

        Instruction::None
    }

    fn update_screen(&mut self, msg: Message) {

    }
}

// Helper functions for view
impl BlockConfig {
    fn display_error(&self) -> Element<'_, Message> {
        let error_str = 
            match self.error {
                Some(error_str) => error_str,
                None => " "
            };

        container(
            text(error_str)
                .color(COLOR_RED)
                .size(15)
        )
        .center_x(Length::Fill)
        .into()
    }
}


fn finish_buttons<'a>() -> Element<'a, Message> {
    Container::new(
        Row::with_children([
            button_with_text("Close", Message::Close),
            button_with_text("Block without lock", Message::BlockWithoutLock),
            button_with_text("Block and lock", Message::BlockWithLock),
            Space::new().into(),
        ])
        .spacing(20)
    )
    .align_right(Length::Fill)
    .into()   
}
