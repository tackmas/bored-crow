mod timer;

use std::{
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
    Length::*,
    widget::{
        Button,
        Column,
        Container,
        container,
        Responsive,
        Row,
        Scrollable,
        Space,
        Stack,
        Text,
    },
};

use timer::{
    Timer,
};

use crate::{
    core::{
        block_config::{
            Group,
            BlockRule,
            BlockRuleKind,
        },
    },
    MyOption,
    ui::gui2::{
        button_with_text,
    },
    platform::{
        Blocker,
    },

};

pub enum Action {
    None,
    Close,
}

#[derive(Clone)]
pub enum Message {
    BlockWithLock,
    BlockWithoutLock,
    Close,
    NavigateTimer,    
    Timer(timer::Message)
}

enum Screen {
    Timer(Timer),
    Nothing,
}

impl std::fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = 
            match self {
                Screen::Timer(_) => "Screen::Timer",
                Screen::Nothing => "Screen::Nothing",
            };

        write!(f, "{s}")
    }
}

pub struct BlockConfig {
    blocker: Blocker,
    group: Arc<Group>,
    screen: Screen,
}

impl BlockConfig {
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Close => Action::Close,
            Message::BlockWithoutLock | Message::BlockWithLock => {
                let kind = 
                    match &self.screen {
                        Screen::Timer(timer) => BlockRuleKind::Timer(timer.duration()),
                        Screen::Nothing => unreachable!(), 
                    };

                let lock_when_blocked = 
                    match message {
                        Message::BlockWithoutLock => false,
                        Message::BlockWithLock => true,
                        _ => unimplemented!()
                    };
                
                let blocker = self.blocker.clone();

                let block_rule = 
                    BlockRule {
                        blocker,
                        kind,
                        lock_when_blocked,
                    };

                self.group.block(block_rule);

                Action::Close
            },   
            Message::Timer(msg) => {
                if let Screen::Timer(timer) = &mut self.screen {
                    timer.update(msg);
                } else {
                    panic!("{} was emitted when screen is {}, and not {}",
                    "Message::Timer", self.screen, "Screen::Timer")
                }

                Action::None
            }
            Message::NavigateTimer => {
                let timer = Timer::new();
                self.screen = Screen::Timer(timer);

                Action::None
            }
        }

    }
    pub fn view(&self) -> Element<'_ , Message> {
        let tabs = Row::with_children([
            button_with_text("Timer", Message::NavigateTimer),
        ]);

        let tab_content = 
            match &self.screen {
                Screen::Timer(timer) => timer
                    .view()
                    .map(Message::Timer),
                Screen::Nothing => !unimplemented!()
            };

        let finish_buttons =
            Container::new(
                Row::with_children([
                    button_with_text("Close", Message::Close),
                    button_with_text("Block without lock", Message::BlockWithoutLock),
                    button_with_text("Block and lock", Message::BlockWithLock),
                    Space::new().into(),
                ])
                .spacing(20)
            )
            .align_right(Fill);


        Column::with_children([
            tabs.into(),
            tab_content,
            finish_buttons.into(),
        ])
        .into()
    }
}

impl From<(Blocker, Arc<Group>)> for BlockConfig {
    fn from((blocker, group): (Blocker, Arc<Group>)) -> Self {
        BlockConfig {
            blocker,
            group,
            screen: Screen::Timer(Timer::new()),
        }
    }
}