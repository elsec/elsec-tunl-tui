use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let [main, footer] = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());
    let [left, right] = Layout::horizontal([Constraint::Length(24), Constraint::Min(1)]).areas(main);

    let items: Vec<ListItem> = app
        .tunnels
        .iter()
        .map(|t| {
            let (dot, color) = if t.iface.is_some() {
                ("●", Color::Green)
            } else {
                ("○", Color::DarkGray)
            };
            ListItem::new(Line::from(vec![Span::styled(dot, color), Span::raw(" "), Span::raw(&t.name)]))
        })
        .collect();
    let list = List::new(items)
        .block(Block::bordered().title(" Tunnels "))
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default().with_selected((!app.tunnels.is_empty()).then_some(app.selected));
    frame.render_stateful_widget(list, left, &mut state);

    let details = Paragraph::new(detail_lines(app))
        .block(Block::bordered().title(" Status "))
        .wrap(Wrap { trim: false });
    frame.render_widget(details, right);

    let footer_line = match &app.message {
        Some(m) if m.is_error => Line::from(m.text.as_str().red()),
        Some(m) => Line::from(m.text.as_str()),
        None => Line::from("j/k move  enter toggle  r refresh  q quit".dark_gray()),
    };
    frame.render_widget(Paragraph::new(footer_line), footer);

    if let Some(text) = &app.popup {
        let area = centered(frame.area(), 80, 60);
        let popup = Paragraph::new(text.as_str())
            .block(Block::bordered().title(" Error ").title_bottom(" any key to close ").red())
            .wrap(Wrap { trim: false });
        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
    }
}

fn centered(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center).areas(area);
    let [area] = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center).areas(area);
    area
}

fn detail_lines(app: &App) -> Vec<Line<'_>> {
    let Some(tunnel) = app.selected() else {
        let hint = match &app.message {
            Some(m) if m.is_error => "Could not load tunnels. Is elsec-tunl-priv installed? (sudo ./install.sh)",
            _ => "No tunnels found in /etc/wireguard",
        };
        return vec![Line::from(hint)];
    };
    let Some(iface) = &tunnel.iface else {
        return vec![
            Line::from(tunnel.name.as_str().bold()),
            Line::from("down".dark_gray()),
        ];
    };

    let mut lines = vec![
        Line::from(vec![tunnel.name.as_str().bold(), "  up".green()]),
        field("public key", iface.public_key.clone()),
        field("listen port", iface.listen_port.clone()),
    ];
    for peer in &iface.peers {
        lines.push(Line::default());
        lines.push(field("peer", peer.public_key.clone()));
        lines.push(field("endpoint", peer.endpoint.clone()));
        lines.push(field("allowed ips", peer.allowed_ips.clone()));
        lines.push(Line::from(vec![
            Span::styled(format!("{:>12}  ", "handshake"), Color::DarkGray),
            Span::styled(ago(peer.latest_handshake), handshake_color(peer.latest_handshake)),
        ]));
        lines.push(field("transfer", format!("{} received, {} sent", bytes(peer.rx), bytes(peer.tx))));
    }
    lines
}

fn field(label: &str, value: String) -> Line<'static> {
    Line::from(vec![Span::styled(format!("{label:>12}  "), Color::DarkGray), Span::raw(value)])
}

/// WireGuard re-handshakes about every 2 minutes on an active tunnel, so an older
/// handshake means the peer is idle or unreachable.
fn handshake_color(timestamp: u64) -> Color {
    match timestamp {
        0 => Color::Red,
        _ => match now().saturating_sub(timestamp) {
            0..=180 => Color::Green,
            181..=600 => Color::Yellow,
            _ => Color::Red,
        },
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn ago(timestamp: u64) -> String {
    if timestamp == 0 {
        return "never".into();
    }
    let secs = now().saturating_sub(timestamp);
    match secs {
        0..60 => format!("{secs}s ago"),
        60..3600 => format!("{}m ago", secs / 60),
        _ => format!("{}h ago", secs / 3600),
    }
}

fn bytes(n: u64) -> String {
    const UNITS: [&str; 4] = ["KiB", "MiB", "GiB", "TiB"];
    if n < 1024 {
        return format!("{n} B");
    }
    let mut value = n as f64;
    let mut unit = "B";
    for u in UNITS {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = u;
    }
    format!("{value:.1} {unit}")
}
