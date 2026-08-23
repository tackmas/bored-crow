use iced::Color;
use iced::Element;
use iced::Length;
use iced::alignment::Vertical;
use iced::border::{Border, Radius};
use iced::font::{Font, Weight};
use iced::widget::{
    button,
    checkbox,
    Column, column,
    Container, container,
    radio,
    right,
    Row, row,
    Space, 
    Text,
    text
};

use crate::core::block::block_rule::{LockConfig as CoreLockConfig, MoreLockConfig};
use crate::core::block::timer::Timer;
use crate::gui::state::{LENGTH_UNIT, Pad};
use crate::gui::state::action;
use crate::gui::state::modal::{self, radio_with_border, title};

use super::Tab;
use super::timer;
use super::timer::Timer as GuiTimer;

pub enum CustomAction {
    BackToBlockConfig,
    StartBlock(CoreLockConfig),
}

pub type CA = CustomAction;
pub type Action = action::Action<CA, Message>;

#[derive(Clone)]
pub enum Message {
    // Footer actions
    Close,
    Back,
    StartBlock,

    // Forward message
    Timer(timer::Message),

    // Update
    LockModeSelected(LockMode),
    ToggleBlockTaskManager(bool),
    ToggleProhibitUninstall(bool)
}

pub struct LockConfig {
    lock_mode: LockMode,
    more_lock_config: MoreLockConfig,
    gui_timer: GuiTimer,
}

// Constructor
impl LockConfig {
    pub fn new() -> Self {
        Self {
            lock_mode: LockMode::NoLock,
            more_lock_config: MoreLockConfig::new(),
            gui_timer: super::timer::Timer::new(),
        }
    }
}

// Update
impl LockConfig {
    #[must_use]
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Close => Action::none().close_modal(),
            Message::Back => Action::none_with_custom(CA::BackToBlockConfig),
            Message::StartBlock => self.start_block_action(),
            Message::Timer(msg) => {
                self.gui_timer.update(msg);

                Action::none()
            },
            Message::LockModeSelected(lock_mode) => {
                self.lock_mode = lock_mode;

                Action::none()
            },
            Message::ToggleBlockTaskManager(toggled) => {
                self.more_lock_config
                    .block_task_manager = toggled;

                Action::none()
            },
            Message::ToggleProhibitUninstall(toggled) => {
                self.more_lock_config
                    .prohibit_uninstall = toggled;

                println!("Toggled Prohibit uninstall to {toggled}");

                Action::none()
            }
        }
    }

    fn start_block_action(&self) -> Action {
        let lock_rule = match self.lock_mode {
            LockMode::NoLock => CoreLockConfig::NoLock,
            LockMode::LockWhileBlocked => CoreLockConfig::LockWhenBlocked(self.more_lock_config),
            LockMode::LockWithTimer => {
                let duration = self.gui_timer.try_duration().unwrap();
                let timer = Timer::new(duration);

                CoreLockConfig::Timer(timer, self.more_lock_config)
            }
        };

        Action::none_with_custom(CA::StartBlock(lock_rule)).close_modal()
    }
}

const NO_LOCK: &str = "No lock";
const LOCK_WHILE_BLOCKED: &str = "Lock while blocked";
const LOCK_WITH_TIMER: &str = "Lock with timer";

// View
impl LockConfig {
    pub fn view(&self, block_mode: Tab) -> Element<'_, Message> {
        let primary_title = modal::primary_title("Lock configuration");

        let main_content = self
            .main_content(block_mode)
            .width(Length::Fill)
            .height(Length::FillPortion(8));

        let footer_actions = self.footer_actions().height(Length::FillPortion(1));

        let view = column![primary_title, main_content, footer_actions];

        view.into()
    }

    fn main_content(&self, block_mode: Tab) -> Row<'_, Message> {
        let left_half_content = self
            .lock_mode_selection(block_mode)
            .width(Length::Fill)
            .height(Length::Fill);

        let right_half_content: Element<'_, Message> = {
            let lock_mode_details = self.lock_mode_details();

            if let LockMode::NoLock = self.lock_mode {
                lock_mode_details.into()
            } else {
                let more_config = self
                    .more_lock_config()
                    .width(Length::Fill)
                    .height(Length::Fill);

                column![lock_mode_details, more_config].into()
            }
        };

        row![left_half_content, right_half_content]
    }

    fn lock_mode_selection(&self, block_mode: Tab) -> Column<'_, Message> {
        let title_bar = title("Lock mode", 20.0);
        let radio_list = self.lock_mode_radio_list(block_mode);

        column![title_bar, radio_list]
    }
    fn lock_mode_radio_list(&self, block_mode: Tab) -> Column<'_, Message> {
        let lock_mode_radio = |text: &'static str, to| {
            radio_with_border(text, to, Some(self.lock_mode), Message::LockModeSelected)
        };

        let no_lock = lock_mode_radio(NO_LOCK, LockMode::NoLock);
        let lock_while_blocked = lock_mode_radio(LOCK_WHILE_BLOCKED, LockMode::LockWhileBlocked);

        let content = column![no_lock, lock_while_blocked];

        match block_mode {
            Tab::Timer => content,
            _ => {
                let lock_with_timer = lock_mode_radio(LOCK_WITH_TIMER, LockMode::LockWithTimer);

                content.push(lock_with_timer)
            }
        }
    }

    fn lock_mode_details(&self) -> Column<'_, Message> {
        let lock_mode_info = |title_text: &'static str, description_text: &'static str| {
            let title_bar = title(title_text, 20.0);
            let description = description(description_text, 14.0);

            column![title_bar, description]
        };

        match self.lock_mode {
            LockMode::NoLock => lock_mode_info(
                NO_LOCK,
                "No lock will be applied to the block. You will be able to unblock whenever.",
            ),
            LockMode::LockWhileBlocked => lock_mode_info(
                LOCK_WHILE_BLOCKED,
                "While actively blocking, the block will be locked and you will be unable to unblock",
            ),
            LockMode::LockWithTimer => {
                let lock_mode_info = lock_mode_info(
                    LOCK_WITH_TIMER,
                    "While the timer is alive, the block is locked and you will be unable to unblock",
                );
                let timer = self.gui_timer
                    .view()
                    .map(Message::Timer)
                    .pad(*LENGTH_UNIT);

                column![lock_mode_info, timer]
            }
        }
    }

    fn more_lock_config(&self) -> Column<'_, Message> {
        let title_bar = title("Additional configurations if locked", 20.0);
        let content = {
            let indent = Space::new().width(*LENGTH_UNIT * 2.0);
            let content = self.more_lock_config.view();

            row![indent, content]
        };

        column![title_bar, content]
    }

    fn footer_actions(&self) -> Container<'_, Message> {
        let close = button(text("Close")).on_press(Message::Close);
        let back = button(text("Back")).on_press(Message::Back);
        let start_block = button(text("Start block")).on_press(Message::StartBlock);

        let buttons_row = row![close, back, start_block]
            .align_y(Vertical::Center)
            .spacing(10);

        let footer_actions = right(buttons_row);

        footer_actions
    }
}

// Helper functions for view
fn description<'a>(text_str: &'a str, text_size: f32) -> Container<'a, Message> {
    let description_text = text(text_str).size(text_size);

    let description = text_with_padding(description_text);

    description
}

fn text_with_padding<'a>(text: Text<'a>) -> Container<'a, Message> {
    container(text).align_left(Length::Fill).padding(*LENGTH_UNIT)
}

// other stuff
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    NoLock,
    LockWhileBlocked,
    LockWithTimer,
}

impl MoreLockConfig {
    pub fn new() -> Self {
        Self {
            block_task_manager: false,
            prohibit_uninstall: false,
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        let block_task_manager = checkbox_config(
            self.block_task_manager,
            Message::ToggleBlockTaskManager,
            "Block Task Manager"
        );

        let prohibit_uninstall = checkbox_config(
            self.prohibit_uninstall, 
            Message::ToggleProhibitUninstall,
            "Prohibit uninstalling"
        );

        column![
            block_task_manager,
            prohibit_uninstall
        ]
        .into()
    }
}

fn checkbox_config<'a>(
    toggled: bool, 
    on_toggle: impl Fn(bool) -> Message + 'a, 
    description: &'a str
) -> Row<'a, Message> 
{
    let checkbox = checkbox(toggled)
        .on_toggle(on_toggle)
        .pad(*LENGTH_UNIT);

    let description = text(description)
        .pad(*LENGTH_UNIT);

    row![checkbox, description]
        .align_y(Vertical::Center)
}