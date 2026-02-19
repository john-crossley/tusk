use std::io::{self, IsTerminal};

use colored::{ColoredString, Colorize};

use crate::models::item::ItemPriority;

pub struct Theme {
    pub color: bool,
}

impl Theme {
    pub fn new(color: bool) -> Self {
        let color = io::stdout().is_terminal() && color;
        Self { color }
    }

    pub fn title(&self, s: &str) -> ColoredString {
        if self.color { s.bold() } else { s.normal() }
    }

    pub fn dim(&self, s: &str) -> ColoredString {
        if self.color { s.dimmed() } else { s.normal() }
    }

    pub fn ok(&self, s: &str) -> ColoredString {
        if self.color {
            s.green().bold()
        } else {
            s.normal()
        }
    }

    pub fn warn(&self, s: &str) -> ColoredString {
        if self.color {
            s.yellow().bold()
        } else {
            s.normal()
        }
    }

    pub fn info(&self, s: &str) -> ColoredString {
        if self.color {
            s.blue().dimmed().bold()
        } else {
            s.normal()
        }
    }

    pub fn info_em(&self, s: &str) -> ColoredString {
        if !self.color {
            return s.normal();
        }

        s.blue().dimmed().bold().italic()
    }

    pub fn checkbox(&self, done: bool) -> &'static str {
        if self.color && io::stdout().is_terminal() {
            if done { "☑" } else { "☐" }
        } else {
            if done { "[x]" } else { "[ ]" }
        }
    }

    pub fn priority(&self, p: &ItemPriority) -> ColoredString {
        let g = match p {
            ItemPriority::High => "‼",
            ItemPriority::Medium => "▲",
            ItemPriority::Low => "▽",
        };

        if !self.color {
            return g.normal();
        }

        match p {
            ItemPriority::High => g.red().bold(),
            ItemPriority::Medium => g.yellow().bold(),
            ItemPriority::Low => g.dimmed(),
        }
    }
}
