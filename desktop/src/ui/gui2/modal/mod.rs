use iced::{
    Element,
};

pub mod new_guigroup;

pub enum Message {
    NewGUIGroup(new_guigroup::Message)
}

pub enum Modal {
    NewGUIGroup(new_guigroup::NewGUIGroup)
}


impl Modal {
    pub fn new() -> Self {
        todo!()
    }
    pub fn update(&mut self, message: Message) {
        todo!();
    }
    pub fn view(&self) -> Element<'_, Message> {
        todo!()
    }
}
