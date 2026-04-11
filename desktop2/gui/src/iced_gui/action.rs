
pub enum ModalAction {
    None,
    Close,
    Open,
}

pub struct Action<I> {
    pub instruction: I,
    pub modal_action: ModalAction,
    pub save: bool,
}

impl<I> Action<I> {
    pub fn new_with(instruction: I, modal_action: ModalAction, save: bool) -> Self {
        Self {
            instruction,
            modal_action,
            save,
        }
    }
    pub fn map_instruction<T>(self, mapper: impl FnOnce(I) -> T) -> Action<T> {
        Action {
            instruction: mapper(self.instruction),
            modal_action: self.modal_action,
            save: self.save
        }
    }
}


