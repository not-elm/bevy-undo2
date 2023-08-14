use bevy::ecs::system::SystemParam;
use bevy::prelude::{Event, EventWriter};

#[derive(Event, Default, PartialEq, Debug, Copy, Clone, Hash)]
pub struct RequestUndoEvent;


#[derive(SystemParam)]
pub struct UndoRequester<'w> {
    ew: EventWriter<'w, RequestUndoEvent>
}


impl<'w> UndoRequester<'w> {
    #[inline(always)]
    pub fn undo(&mut self) {
        self.ew.send(RequestUndoEvent);
    }
}