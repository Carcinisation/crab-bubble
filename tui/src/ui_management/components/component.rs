use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{prelude::Backend, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::state_store::{action::Action, State};

pub trait Component {
    fn new(state: &State, action_tx: UnboundedSender<Action>) -> Self
    where
        Self: Sized;
    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized;

    fn name(&self) -> &str;

    fn handle_key_event(&mut self, key: KeyEvent);

    fn handle_mouse_event(&mut self, mouse: MouseEvent);
}

pub trait ComponentRender<Props> {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, props: Props);
}
