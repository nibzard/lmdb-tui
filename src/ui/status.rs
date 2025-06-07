use ratatui::{
    prelude::{Frame, Rect},
    widgets::Paragraph,
};

use crate::config::Config;
use crate::util::key_label;

pub fn render(f: &mut Frame, area: Rect, cfg: &Config) {
    let text = format!(
        "{}: quit | {}: help | {}: query | {}: up | {}: down",
        key_label(&cfg.keybindings.quit),
        key_label(&cfg.keybindings.help),
        key_label(&cfg.keybindings.query),
        key_label(&cfg.keybindings.up),
        key_label(&cfg.keybindings.down),
    );
    let p = Paragraph::new(text);
    f.render_widget(p, area);
}
