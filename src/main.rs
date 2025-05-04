use std::{io, time::{Duration, Instant}};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug, Default)]
pub struct App {
    counter: i8,
    binary_digits: Vec<u8>,
    result: Option<u32>,
    flash: Option<Instant>,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(&*self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            }
        }
        // clear flash after short duration
        if let Some(start) = self.flash {
            if start.elapsed() > Duration::from_millis(150) {
                self.flash = None;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('j') => {
                self.decrement_counter();
                self.flash = Some(Instant::now());
            }
            KeyCode::Char('k') => {
                self.increment_counter();
                self.flash = Some(Instant::now());
            }
            KeyCode::Enter => self.handle_enter(),
            KeyCode::Char('r') => self.reset(),
            KeyCode::Backspace => self.backspace(),
            _ => {}
        }
    }

    fn increment_counter(&mut self) {
        self.counter = (self.counter + 1) % 2;
    }

    fn decrement_counter(&mut self) {
        self.counter = (self.counter - 1).rem_euclid(2);
    }

    fn handle_enter(&mut self) {
        if self.binary_digits.len() < 10 {
            if self.binary_digits.is_empty() && self.result.is_some() {
                self.result = None;
            }
            self.binary_digits.push(self.counter as u8);
        } else {
            self.result = Some(self.calculate_decimal());
            self.binary_digits.clear();
        }
    }

    fn backspace(&mut self) {
        if !self.binary_digits.is_empty() {
            self.binary_digits.pop();
            self.result = None;
        }
    }

    fn reset(&mut self) {
        self.counter = 0;
        self.binary_digits.clear();
        self.result = None;
        self.flash = None;
    }

    fn calculate_decimal(&self) -> u32 {
        self.binary_digits
            .iter()
            .enumerate()
            .map(|(i, &bit)| (bit as u32) << (9 - i))
            .sum()
    }

    fn calculate_live_preview(&self) -> Option<u32> {
        if self.binary_digits.is_empty() {
            None
        } else {
            Some(self.calculate_decimal())
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::styled(
            " Paijo's Religions Converter ",
            Style::default().fg(Color::Yellow).bold(),
        );

        let instructions = Line::from(vec![
            Span::styled(" [K] ", Style::default().fg(Color::Blue).bold()),
            Span::styled("Toggle 1 ", Style::default().fg(Color::White).bold()),
            Span::styled("[J] ", Style::default().fg(Color::Blue).bold()),
            Span::styled("Toggle 0 ", Style::default().fg(Color::White).bold()),
            Span::styled("[Enter] ", Style::default().fg(Color::Blue).bold()),
            Span::styled("Push/Convert ", Style::default().fg(Color::White).bold()),
            Span::styled("[Q] ", Style::default().fg(Color::Red).bold()),
            Span::styled("Quit ", Style::default().fg(Color::Red).bold()),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK)
            .border_style(Style::default().fg(Color::Rgb(254, 128, 25)));

        let mut lines = vec![];

        // Counter with flash effect
        let counter_style = if self.flash.is_some() {
            Style::default().black().on_yellow().bold()
        } else {
            Style::default().yellow()
        };

        lines.push(Line::from(vec![
            "Counter: ".into(),
            Span::styled(self.counter.to_string(), counter_style),
        ]));

        lines.push(Line::from("")); // spacing

        // Result or live preview
        lines.push(Line::from(vec![
            Span::styled("Result: ", Style::default().fg(Color::Magenta).bold()),
            match self.result {
                Some(val) => Span::styled(val.to_string(), Style::default().fg(Color::Green).bold()),
                None => match self.calculate_live_preview() {
                    Some(val) => Span::styled(format!("Preview: {val}"), Style::default().fg(Color::DarkGray)),
                    None => Span::styled("Waiting...", Style::default().fg(Color::DarkGray)),
                },
            },
        ]));

        lines.push(Line::from("")); // spacing

        // Binary input boxes
        let mut binary_line = Line::default();
        for &bit in &self.binary_digits {
            binary_line.spans.push(Span::raw("["));
            binary_line.spans.push(Span::styled(format!("{bit}"), Style::default().bold()));
            binary_line.spans.push(Span::raw("] "));
        }
        for _ in self.binary_digits.len()..10 {
            binary_line.spans.push("[ ] ".into());
        }
        lines.push(binary_line);

        // Cursor pointer
        let mut cursor_line = Line::default();
        for i in 0..10 {
            if i == self.binary_digits.len() {
                cursor_line.spans.push(" ^  ".magenta());
            } else {
                cursor_line.spans.push("    ".into());
            }
        }
        lines.push(cursor_line);

        let text = Text::from(lines);
        Paragraph::new(text).centered().block(block).render(area, buf);
    }
}
