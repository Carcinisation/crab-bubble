use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseEvent};
use ratatui::{
    prelude::{Backend, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;
use unicode_width::UnicodeWidthChar;

use crate::state_store::{action::Action, State};

use super::{Component, ComponentRender};

pub struct InputBox {
    /// Current value of the input box
    text: String,
    /// 에디터 커서의 위치로, 문자 인덱스를 나타낸다. 바이트 인덱스가 아님에 주의.
    cursor_position: usize,
}

impl InputBox {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, new_text: &str) {
        self.text = String::from(new_text);
        self.cursor_position = self.get_max_cursor_position();
    }

    pub fn reset(&mut self) {
        self.cursor_position = 0;
        self.text.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// 커서 최대값(문자 수)를 얻는다. 참고로 text.len()는 실제 문자 수가 아니라 바이트 수를 반환한다.
    fn get_max_cursor_position(&self) -> usize {
        self.text.char_indices().count()
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.text.insert(self.get_cursor_byte_index(), new_char);

        self.move_cursor_right();
    }

    /// 현재 커서의 바이트 인덱스를 얻는다. 문자열은 UTF-8 형식이므로, 문자 인덱스 != 바이트 인덱스이다. 
    fn get_cursor_byte_index(&self) -> usize {
        self.text
            .char_indices() // 문자열의 각 문자의 시작 위치와 문자를 반환하는 반복자를 생성한다.
            .nth(self.cursor_position) // 반복자에서 n번째 문자의 시작 위치와 문자를 가져온다.
            .map(|(i, _)| i) // (usize, char) 튜플에서 usize만 가져온다.
            .unwrap_or(self.text.len()) // (안전하게 처리) Option<usize>에서 usize를 가져오고, None이면 문자열의 길이를 반환한다.
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.text.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.text.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.text = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.get_max_cursor_position())
    }

    /// 한글과 같은 double-byte 문자를 처리하기 위해 문자 너비를 고려하여 터미널 상의 커서 위치를 계산한다.
    fn get_terminal_cursor_position(&self) -> u16 {
        let mut terminal_cursor_position = 0;
        for (i, c) in self.text.chars().enumerate() {
            if i == self.cursor_position {
                break;
            }
            // 문자 너비 더하기
            terminal_cursor_position += c.width().unwrap_or(0) as u16;
        }
        terminal_cursor_position
    }
}

impl Component for InputBox {
    fn new(_state: &State, _action_tx: UnboundedSender<Action>) -> Self {
        Self {
            //
            text: String::new(),
            cursor_position: 0,
        }
    }

    fn move_with_state(self, _state: &State) -> Self
    where
        Self: Sized,
    {
        Self { ..self }
    }

    fn name(&self) -> &str {
        "Input Box"
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char(to_insert) => {
                self.enter_char(to_insert);
            }
            KeyCode::Backspace => {
                self.delete_char();
            }
            KeyCode::Left => {
                self.move_cursor_left();
            }
            KeyCode::Right => {
                self.move_cursor_right();
            }
            _ => {}
        }
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) {}
}

pub struct RenderProps {
    pub title: String,
    pub area: Rect,
    pub border_color: Color,
    pub show_cursor: bool,
}

impl ComponentRender<RenderProps> for InputBox {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, props: RenderProps) {
        let input = Paragraph::new(self.text.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .fg(props.border_color)
                    .title(props.title),
            );
        frame.render_widget(input, props.area);

        // Cursor is hidden by default, so we need to make it visible if the input box is selected
        if props.show_cursor {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            frame.set_cursor(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                props.area.x + self.get_terminal_cursor_position() as u16 + 1,
                // Move one line down, from the border to the input line
                props.area.y + 1,
            )
        }
    }
}
