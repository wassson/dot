use crossterm::event::*;
use crossterm::terminal::ClearType;
use crossterm::{cursor, event, execute, queue, terminal};
use std::io::{stdout, Write, self};
use std::time::Duration;

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not disable raw mode");
        Output::clear_screen().expect("Error");
    }
}

struct Output {
    win_size: (usize, usize),
    editor_contents: EditorContents,
    cursor_controller: CursorController,
}

impl Output {
    fn new() -> Self {
        let win_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap();
        Self { 
            win_size,
            editor_contents: EditorContents::new(),
            cursor_controller: CursorController::new(),
        }
    }

    fn clear_screen() -> std::result::Result<(), std::io::Error> {
        execute!(stdout(), terminal::Clear(ClearType::UntilNewLine))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    fn draw_rows(&mut self) {
        let screen_rows = self.win_size.1;
        let screen_columns = self.win_size.0;
        for i in 0..screen_rows {
            if i == screen_rows / 3 {
                let mut welcome = format!("Pound Editor --- Version {}", "0.0.1");
                if welcome.len() > screen_columns {
                    welcome.truncate(screen_columns)
                }
                /* add the following*/
                let mut padding = (screen_columns - welcome.len()) / 2;
                if padding != 0 {
                    self.editor_contents.push('~');
                    padding -= 1
                }
                (0..padding).for_each(|_| self.editor_contents.push(' '));
                self.editor_contents.push_str(&welcome);
                /* end */
            } else {
                self.editor_contents.push('~');
            }
            queue!(
                self.editor_contents,
                terminal::Clear(ClearType::UntilNewLine)
            )
            .unwrap();
            if i < screen_rows - 1 {
                self.editor_contents.push_str("\r\n");
            }
        }
    }

    fn refresh_screen(&mut self) -> std::result::Result<(), std::io::Error> {
        queue!(
            self.editor_contents, 
            cursor::Hide,
            terminal::Clear(ClearType::All), 
            cursor::MoveTo(0, 0)
        )?;
        self.draw_rows();
        let cursor_x = self.cursor_controller.cursor_x;
        let cursor_y = self.cursor_controller.cursor_y;
        queue!(
            self.editor_contents, 
            cursor::MoveTo(0, 0),
            cursor::Show
        )?;
        self.editor_contents.flush()
    }

    fn move_cursor(&mut self,direction:char) {
        self.cursor_controller.move_cursor(direction);
    }
}

struct Reader;

impl Reader {
    fn read_key(&self) -> std::result::Result<KeyEvent, std::io::Error> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
    }
}

struct CursorController {
    cursor_x: usize,
    cursor_y: usize,
}

impl CursorController {
    fn new() -> CursorController {
        Self {
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    fn move_cursor(&mut self, direction: char) {
        match direction {
            'w' => {
                self.cursor_y -= 1;
            }
            'a' => {
                self.cursor_x -= 1;
            }
            's' => {
                self.cursor_y += 1;
            }
            'd' => {
                self.cursor_x += 1;
            }
            _ => unimplemented!(),
        }
    }
}

struct EditorContents {
    content: String,
}

impl EditorContents {
    fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn push(&mut self, ch: char) {
        self.content.push(ch)
    }

    fn push_str(&mut self, string: &str) {
        self.content.push_str(string)
    }
}

impl io::Write for EditorContents {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.content.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(io::ErrorKind::WriteZero.into()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let out = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}

struct Editor {
    reader: Reader,
    output: Output,
}

impl Editor {
    fn new() -> Self {
        Self {
            reader: Reader,
            output: Output::new(),
        }
    }

    fn process_keypress(&mut self) -> std::result::Result<bool, std::io::Error> { /* modify*/
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                kind: _,
                state: _
            } => return Ok(false),
            /* add the following*/
            KeyEvent {
                code: KeyCode::Char(val @ ('w' | 'a' | 's' | 'd')),
                modifiers: KeyModifiers::NONE,
                kind: _,
                state: _
            } => self.output.move_cursor(val),
            // end
            _ => {}
        }
        Ok(true)
    }

    fn run(&mut self) -> std::result::Result<bool, std::io::Error> {
        self.output.refresh_screen()?;
        self.process_keypress()
    }
}

fn main() -> std::result::Result<(), std::io::Error> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;

    let mut editor = Editor::new();
    while editor.run()? {}

    Ok(())
}