mod new_group;

// Dependencies
use iced::{
    self,
    alignment::Horizontal,
    Element,
    Length,
    widget::{
        Button,
        Column,
        Container,
        Row,
        Space,
        Stack,
        Text,
    }
};

// Local
use crate::gui2::Route;
use crate::platform::App;
use new_group::{NewGroupEvent, NewGroupState, OptionNewGroupState};

#[derive(Clone)]
pub enum BlockEvent {
    NewGroup(Route<NewGroupEvent>),
}

struct Group {
    inner: Vec<App>,
}

impl Group {
    fn new() -> Self {
        let inner = Vec::new();

        Group { inner }
    }
}

pub struct BlockState {
    groups: Vec<Group>,
    option_new_group: Option<NewGroupState>,
}

impl BlockState {
    pub fn new() -> Self {
        let groups = Vec::new();
        let option_new_group = None;

        BlockState { 
            groups, 
            option_new_group 
        }
    }

    pub fn update(&mut self, event: BlockEvent) {
        match event {
            BlockEvent::NewGroup(route) => {
                match route {
                    Route::Forward(event) => self.option_new_group.update(event),
                    Route::Open =>{
                        if self.option_new_group.is_none() {
                            self.option_new_group = OptionNewGroupState::new();
                        }
                    } 
                }
            }
        }
    }   

    pub fn view(&self) -> Element<'_, BlockEvent> {
        let first_row = self.first_row();
        let group_list = self.group_list();

        let column = 
            Column::with_children([
                first_row.into(),
                group_list.into()
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center);

        let stack_base = 
            Stack::with_children([
                column.into()
            ]);

        if self.option_new_group.is_some() {
            let new_group_view = self.option_new_group
                .view()
                .map(|event| BlockEvent::NewGroup(Route::Forward(event)));

            let stack = stack_base.push(new_group_view);

            return stack.into();
        }

        stack_base.into()

        /* 

        let stack = {
            let base = Stack::with_children([
                column.into()
            ]);

            if self.new_group.is_some() {
                let new_group = self.new_group
                    .view()
                    .map(|event| BlockEvent::NewGroup(Route::Forward(event)));

                base.push(new_group)
            } else {
                base
            }
        };
            
        stack.into()

        */
    }

    fn first_row(&self) -> Container<'_, BlockEvent> {
        let (_, screen_height) = crate::gui2::screen_size();

        let title = 
            Container::new(
                Text::new("App Blocks")
            )
            .center_x(Length::FillPortion(1))
            .center_y(Length::Fill);

        let new_group_button = 
            Container::new(
                Button::new(
                    Text::new("New Group")
                )
                .on_press(BlockEvent::NewGroup(Route::Open))
            )
            .center_x(Length::FillPortion(1))
            .center_y(Length::Fill);

        let space = Space::new()
            .width(Length::FillPortion(5));

        let first_row = 
            Container::new(
                Row::with_children([
                    title.into(), 
                    space.into(),
                    new_group_button.into()
                ])
            )
            .center_x(Length::Fill)
            .center_y(screen_height / 15);

        first_row
    }

    fn group_list(&self) -> Container<'_, BlockEvent> {
        if self.groups.is_empty() { 
            Container::new(
                Text::new("No groups currently exist")
            )
            .center(Length::Fill)
        } else {
            todo!()
        }
    }
}
