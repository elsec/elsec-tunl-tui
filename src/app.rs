use std::time::Instant;

use crate::wg::{self, Iface};

pub struct Tunnel {
    pub name: String,
    pub iface: Option<Iface>,
}

pub struct Message {
    pub text: String,
    pub is_error: bool,
}

pub struct App {
    pub tunnels: Vec<Tunnel>,
    pub selected: usize,
    pub message: Option<Message>,
    pub last_refresh: Instant,
}

impl App {
    pub fn new() -> Self {
        let mut app = App {
            tunnels: Vec::new(),
            selected: 0,
            message: None,
            last_refresh: Instant::now(),
        };
        app.refresh();
        app
    }

    pub fn refresh(&mut self) {
        self.last_refresh = Instant::now();
        let result = wg::list().and_then(|names| Ok((names, wg::status()?)));
        match result {
            Ok((names, mut status)) => {
                self.tunnels = names
                    .into_iter()
                    .map(|name| Tunnel {
                        iface: status.remove(&name),
                        name,
                    })
                    .collect();
                if self.selected >= self.tunnels.len() {
                    self.selected = self.tunnels.len().saturating_sub(1);
                }
            }
            Err(e) => self.error(e.to_string()),
        }
    }

    pub fn selected(&self) -> Option<&Tunnel> {
        self.tunnels.get(self.selected)
    }

    pub fn next(&mut self) {
        if self.selected + 1 < self.tunnels.len() {
            self.selected += 1;
        }
    }

    pub fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Brings the selected tunnel up or down. Blocks until wg-quick finishes.
    pub fn toggle(&mut self) {
        let Some(tunnel) = self.selected() else { return };
        let name = tunnel.name.clone();
        let result = if tunnel.iface.is_some() {
            wg::down(&name).map(|_| format!("{name} is down"))
        } else {
            wg::up(&name).map(|_| format!("{name} is up"))
        };
        match result {
            Ok(text) => self.info(text),
            Err(e) => self.error(format!("{name}: {e}")),
        }
        self.refresh();
    }

    pub fn info(&mut self, text: String) {
        self.message = Some(Message { text, is_error: false });
    }

    fn error(&mut self, text: String) {
        self.message = Some(Message { text, is_error: true });
    }
}
