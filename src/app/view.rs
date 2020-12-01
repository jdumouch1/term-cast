use super::Model;
use tui::{
    Frame, 
    backend::Backend, 
    layout::{Constraint, Layout, Direction, Corner, Rect}, 
    text::Spans, 
    widgets::{Block, List, ListItem, Borders, Paragraph, Wrap}
};


pub fn render<B: Backend>(f: &mut Frame<B>, model: &Model) {
    match &model.mode {
        super::Mode::Help => render_help(f, model),
        _ => render_control(f, model),
    }
}

fn render_help<B: Backend>(f: &mut Frame<B>, _model: &Model) {
    let chunk = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(1)
        .split(f.size());

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Help:");

    let text = Spans::default();
    let paragraph = Paragraph::new(text)
        .block(block);

    f.render_widget(paragraph, chunk[0])
}

fn render_control<B: Backend>(f: &mut Frame<B>, model: &Model) {
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

    
    let items = (&model.log).iter()
        .take(64)                               // Take first 64 elements
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