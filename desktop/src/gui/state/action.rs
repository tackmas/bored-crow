use iced::Task;

#[derive(Clone, Copy, Debug)]
pub enum ModalAction {
    None,
    Close,
    Open,
}

pub struct Action<C, M> {
    pub custom_opt: Option<C>,
    pub task: Task<M>,
    pub modal: ModalAction,
    pub save: bool,
}

impl<C, M> Action<C, M> {
    pub fn none() -> Self {
        Self {
            custom_opt: None,
            task: Task::none(),
            modal: ModalAction::None,
            save: false,
        }
    }
    pub fn with_custom<C2>(self, custom: Option<C2>) -> Action<C2, M> {
        Action {
            custom_opt: custom,
            task: self.task,
            modal: self.modal,
            save: self.save,
        }
    }
    pub fn none_with_custom(custom: C) -> Self {
        Self::none().with_custom(Some(custom))
    }
    pub fn map_custom<C2, F>(self, mapper: F) -> Action<C2, M>
    where
        F: FnOnce(Option<C>) -> Option<C2>,
    {
        Action {
            custom_opt: mapper(self.custom_opt),
            task: self.task,
            modal: self.modal,
            save: self.save,
        }
    }
    pub fn with_task(self, task: Task<M>) -> Self {
        Self {
            custom_opt: self.custom_opt,
            task,
            modal: self.modal,
            save: self.save,
        }
    }
    pub fn map_task<M2, F>(self, mut mapper: F) -> Action<C, M2>
    where
        M: Send + 'static,
        M2: Send + 'static,
        F: FnMut(M) -> M2 + Send + 'static,
    {
        Action {
            custom_opt: self.custom_opt,
            task: self.task.map(move |msg| mapper(msg)),
            modal: self.modal,
            save: self.save,
        }
    }
    pub fn close_modal(self) -> Self {
        Self {
            custom_opt: self.custom_opt,
            task: self.task,
            modal: ModalAction::Close,
            save: self.save,
        }
    }
    pub fn open_modal(self) -> Self {
        Self {
            custom_opt: self.custom_opt,
            task: self.task,
            modal: ModalAction::Open,
            save: self.save,
        }
    }
    pub fn save(self) -> Self {
        Self {
            custom_opt: self.custom_opt,
            task: self.task,
            modal: self.modal,
            save: true,
        }
    }
}
