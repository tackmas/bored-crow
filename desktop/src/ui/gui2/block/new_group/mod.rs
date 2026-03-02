// Std
use std::collections::HashSet;

// Dependencies 
use iced::{
    self,
    Element,
    Length,
    widget::{
        Button,
        Checkbox,
        Column,
        Container,
        Float,
        Row,
        Scrollable,
        Space,
        Text,
    }
};


// Local
use crate::gui2::Modal;
use crate::platform::App;

#[derive(Clone)]
pub enum NewGroupEvent {
    Exit,

    Select(App),
    Unselect(App),
}

pub trait OptionNewGroupState {
    fn new() -> Self;
    fn update(&mut self, event: NewGroupEvent);
    fn view(&self) -> Element<'_ , NewGroupEvent>;
}

pub struct NewGroupState {
    apps: Vec<App>,
    selected: HashSet<App>,
}

impl OptionNewGroupState for Option<NewGroupState> {
    fn new() -> Self {
        let apps = App::all_apps().unwrap();
        let selected = HashSet::new();

        Some(NewGroupState { 
            apps,
            selected
        })
    }

    fn update(&mut self, event: NewGroupEvent) {
        if let NewGroupEvent::Exit = event {
            *self = None;
            return;
        } 

        let new_group = self
            .as_mut()
            .expect(r#"Should always be Some, otherwise "fn update()" should not be called"#);

        if let NewGroupEvent::Select(app) = event {
            new_group.selected.insert(app);
        } else if let NewGroupEvent::Unselect(app) = event {
            new_group.selected.remove(&app);
        }

        println!{"{:?}", new_group.selected};
    }   

    fn view(&self) -> Element<'_, NewGroupEvent> {
        let new_group = self
            .as_ref()
            .expect(r#"Should always be Some, otherwise "fn view()" should not be called"#);

        let rows = new_group.apps
            .iter()
            .map(|app| {   
                let is_selected = new_group.selected.contains(app);

                let checkbox = Checkbox::new(is_selected).on_toggle(|toggled| {
                    match toggled {
                        true => NewGroupEvent::Select(app.clone()),
                        false => NewGroupEvent::Unselect(app.clone())
                    }
                });

                let text = Text::new(app.name());

                Row::with_children([
                    checkbox.into(),
                    text.into(),
                ])
                .into()
            });

        let column: Column<'_ , NewGroupEvent> = Column::with_children(rows);

        let scrollable = Scrollable::new(column)
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill);


        let container = Container::new(scrollable)
            .center(Length::Fill);

        let modal = container
            .create_modal(NewGroupEvent::Exit);

        modal.into()
    }
}