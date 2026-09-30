use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Instant;

use anyhow::Result;

use crate::wg::{self, Iface};

pub struct Tunnel {
    pub name: String,
    pub iface: Option<Iface>,
}

pub struct Message {
    pub text: String,
    pub is_error: bool,
}

/// An up/down running on a background thread.
struct Pending {
    name: String,
    going_up: bool,
    rx: Receiver<Result<()>>,
}

pub struct App {
    pub tunnels: Vec<Tunnel>,
    pub selected: usize,
    pub message: Option<Message>,
    /// Full error output from a failed up/down, shown in a popup until dismissed.
    pub popup: Option<String>,
    pub last_refresh: Instant,
    pending: Option<Pending>,
}

impl App {
    pub fn new() -> Self {
        let mut app = App {
            tunnels: Vec::new(),
            selected: 0,
            message: None,
            popup: None,
            last_refresh: Instant::now(),
            pending: None,
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
            Err(e) => self.error(last_line(&e.to_string())),
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

    pub fn is_busy(&self) -> bool {
        self.pending.is_some()
    }

    /// Starts bringing the selected tunnel up or down on a background thread.
    pub fn toggle(&mut self) {
        if self.is_busy() {
            return;
        }
        let Some(tunnel) = self.selected() else { return };
        let name = tunnel.name.clone();
        let going_up = tunnel.iface.is_none();

        let (tx, rx) = mpsc::channel();
        let thread_name = name.clone();
        thread::spawn(move || {
            let result = if going_up { wg::up(&thread_name) } else { wg::down(&thread_name) };
            let _ = tx.send(result);
        });

        let verb = if going_up { "bringing up" } else { "bringing down" };
        self.info(format!("{verb} {name}…"));
        self.pending = Some(Pending { name, going_up, rx });
    }

    /// Checks whether a background up/down has finished and reports the result.
    pub fn poll_pending(&mut self) {
        let Some(pending) = &self.pending else { return };
        let result = match pending.rx.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err(anyhow::anyhow!("helper thread exited unexpectedly")),
        };
        let Pending { name, going_up, .. } = self.pending.take().unwrap();
        match result {
            Ok(()) => self.info(format!("{name} is {}", if going_up { "up" } else { "down" })),
            Err(e) => {
                let text = e.to_string();
                self.error(format!("{name}: {} (see details)", last_line(&text)));
                self.popup = Some(text);
            }
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

fn last_line(text: &str) -> String {
    text.trim().lines().last().unwrap_or("unknown error").to_owned()
}
