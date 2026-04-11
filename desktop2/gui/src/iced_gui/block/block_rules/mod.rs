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
    TimerMessage,
    TimerState,
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
        Modal,
        button_with_text,
    },
    platform::{
        Blocker,
    },

};

#[derive(Clone)]
pub enum BlockRulesMessage {
    BlockWithLock,
    BlockWithoutLock,
    Close,    
    Timer
}

enum BlockRulesScreen {
    Timer,
}

pub struct BlockRulesState {
    blocker: Blocker,
    group: Group,
    screen: BlockRulesScreen,
    timer: TimerState,
}

impl MyOption<BlockRulesState> {
    pub fn new_with(blocker: Blocker, group: Group) -> Self {
        let screen = BlockRulesScreen::Timer;
        let timer = TimerState::new();

        MyOption(Some(BlockRulesState {
            blocker,
            group,
            screen,
            timer,
        }))
    }
    pub fn update(&mut self, message: BlockRulesMessage) {
        use BlockRulesMessage::*;

        match message {
            Close => self.0 = None,
            BlockWithoutLock | BlockWithLock => {
                let state = self.0.as_ref().unwrap();
                let screen = &state.screen;
                
                let kind = 
                    match screen {
                        BlockRulesScreen::Timer => BlockRuleKind::Timer(state.timer.duration()) 
                    };

                let lock_when_blocked = 
                    match message {
                        BlockWithoutLock => false,
                        BlockWithLock => true,
                        _ => unreachable!()
                    };
                
                let blocker = state.blocker.clone();

                let block_rule = 
                    BlockRule {
                        blocker,
                        kind,
                        lock_when_blocked,
                    };

                state.group.clone().block(block_rule);
                self.0 = None;
            },   
            _ => ()
        }

    }
    pub fn view(&self) -> Element<'_ , BlockRulesMessage> {
        let block_rules = self.0
            .as_ref()
            .unwrap();

        let tabs = Row::with_children([
            button_with_text("Timer", BlockRulesMessage::Timer),
        ]);

        let tab_content = 
            match &block_rules.screen {
                BlockRulesScreen::Timer => block_rules.timer
                    .view()
                    .map(|_msg| BlockRulesMessage::Timer)
            };

        let finish_buttons =
            Container::new(
                Row::with_children([
                    button_with_text("Close", BlockRulesMessage::Close),
                    button_with_text("Block without lock", BlockRulesMessage::BlockWithoutLock),
                    button_with_text("Block and lock", BlockRulesMessage::BlockWithLock),
                    Space::new().into(),
                ])
                .spacing(20)
            )
            .align_right(Fill);


        let column = Column::with_children([
            tabs.into(),
            tab_content,
            finish_buttons.into(),
        ]);

        column.create_modal();

        todo!();
    }
}