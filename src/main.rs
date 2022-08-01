// Text editor toy
use crossterm::event::*;
use crossterm::terminal::ClearType;
use crossterm::{queue, cursor, event, terminal, execute};
use std::io::stdout;
use std::io::Write;
use std::time::Duration;

static VERSION: &'static str = "0.0.1";

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not disable raw mode");
        Output::clear_screen().expect("Error");
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
}

struct Output {
    win_size: (usize, usize),
    editor_contents: EditorContents,
}

impl Output {
    fn new() -> Self {
        let win_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap();
        Self { 
            win_size,
            editor_contents: EditorContents::new(),
        }
    }

    fn clear_screen() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0,0))
    }

    fn draw_rows(&mut self) {
        let screen_rows = self.win_size.1;
        let screen_columns = self.win_size.0; 
        for i in 0..screen_rows {
            if i == screen_rows / 3 {
                let mut welcome = format!(
                    "Pound Editor --- Version {}", 
                    VERSION
                );
                if welcome.len() > screen_columns {
                    welcome.truncate(screen_columns)
                }
                let mut padding = (screen_columns - welcome.len()) / 2;
                if padding != 0 {
                    self.editor_contents.push('~');
                    padding -= 1
                }
                (0..padding).for_each(|_| self.editor_contents.push(' '));
                self.editor_contents.push_str(&welcome);
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

    fn refresh_screen(&mut self) -> crossterm::Result<()> {
        queue!(
            self.editor_contents, 
            cursor::Hide,
            terminal::Clear(ClearType::All), 
            cursor::MoveTo(0, 0)
        )?;
        self.draw_rows();
        queue!(
            self.editor_contents, 
            cursor::MoveTo(0, 0),
            cursor::Show,
        )?;
        self.editor_contents.flush()
    }
}

struct Reader;

impl Reader {
    fn read_key(&self) -> crossterm::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
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

    fn process_keypress(&self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
            } => return Ok(false),
            _ => {}
        }
        Ok(true)
    }
    
    fn run(&mut self) -> crossterm::Result<bool> {
        self.output.refresh_screen()?;
        self.process_keypress()
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

impl std::io::Write for EditorContents {

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.content.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(std::io::ErrorKind::WriteZero.into()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let out = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}

fn main() -> crossterm::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;

    let mut editor = Editor::new();
    while editor.run()? {}

    Ok(())
}
