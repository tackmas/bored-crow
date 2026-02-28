use iced::{
    self,
    Element,
    Length,
    widget::{
        Button,
        Column,
        Container,
        Row,
        Space,
        Text,
    }
};

use crate::platform::App;

#[derive(Clone)]
pub enum BlockEvent {
    NewGroup
}

struct Group {
    inner: Vec<App>
}

impl Group {
    fn new() -> Self {
        Group { inner: Vec::new() }
    }
}

pub struct BlockState {
    groups: Vec<Group>
}

impl BlockState {
    pub fn new() -> Self {
        BlockState { groups: Vec::new() }
    }

    pub fn update(&mut self, event: BlockEvent) {
        match event {
            BlockEvent::NewGroup => ()
        }

    }   

    pub fn view(&self) -> Element<'_, BlockEvent> {
        let first_row = self.first_row();
        let group_list = self.group_list();

        Column::with_children([
            first_row.into(),
            group_list.into()
        ])
        .into()
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
                .on_press(BlockEvent::NewGroup)
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
