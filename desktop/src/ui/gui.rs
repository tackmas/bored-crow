use iced::{
    self, 
    advanced::text::IntoFragment,
    Application,
    Element, 
    widget::{
        self, button, column, container, Row, row, text
    }  
};

use crate::platform::{App, Blocker};

#[derive(Clone, Debug)]
enum Message {
    Increment,
    Refresh
}


struct State {
    apps: Vec<App>
}

impl State {
    fn initialize() -> State {
        let apps = App::all_apps().unwrap();

        State { apps }
    } 

    fn update(state: &mut State, message: Message) {
        match message {
            Message::Refresh => { state.apps =  App::all_apps().unwrap(); },
            _ => ()
        };
    }

    fn view(state: &State) -> Element<'_, Message> {
        column![
            vec_into_text_row(App::map_into_names(&state.apps)),
            button(text("Refresh")).on_press(Message::Refresh)
        ].into()
    }
}

fn vec_into_text_row<'a>(v: Vec<impl IntoFragment<'a>>) -> Row<'a, Message> {
    let mut row = Row::new();

    for e in v {
        row = row.push(text(e));
    }

    row
}

pub fn run() -> iced::Result {
    iced::application(State::initialize, State::update, State::view).run()
}
