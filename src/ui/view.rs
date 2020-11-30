use crate::CrossTerminal;

use super::{LogLevel, Model};
use tui::{Frame, Terminal, backend::{Backend, CrosstermBackend}, layout::{Constraint, Layout}, layout::Direction, layout::{Corner, Rect}, style::Modifier, style::Style, text::Span, text::Spans, widgets::Block, widgets::List, widgets::ListItem, widgets::{Borders, Paragraph, Wrap}};

pub fn render<B: Backend>(f: &mut Frame<B>, model: &Model) {

    // Split terminal into two vertical blocks
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.size());

    let upper_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), 
            Constraint::Percentage(50)].as_ref())
        .split(main_chunks[0]);

    draw_media(f, upper_chunks[0], model);
    draw_log(f, upper_chunks[1], model);
    draw_search_bar(f, main_chunks[1], model);
}

fn draw_media<B: Backend>(f: &mut Frame<B>, area: Rect, _model: &Model){
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Media:");
    let _chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Min(0),
            Constraint::Length(4)].as_ref())
        .split(block.inner(area));
    f.render_widget(block, area);
        
        
}

fn draw_log<B: Backend>(f: &mut Frame<B>, area: Rect, model: &Model){
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Log:");

    
    let items = model.get_log_items().iter()
        .take(64)                               // Take first 64 elements
        .filter(|x| (x.0 == LogLevel::General)) // Filter by LogLevel 
        .map(|x| ListItem::new(&(x.1)[..]))     // Convert to ListItems
        .collect::<Vec<ListItem>>();            // iter to Vec<ListItem>
    
    let log = List::new(items)
        .block(block)
        .start_corner(Corner::BottomLeft);
    f.render_widget(log, area);
}

fn draw_search_bar<B: Backend>(f: &mut Frame<B>, area: Rect, model: &Model) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Video File:");

    let textbox = Paragraph::new(model.get_input_span())
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(textbox, area)
}