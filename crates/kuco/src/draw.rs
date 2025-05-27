use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{app::*, data::KubeComponentState};

impl Kuco {
    pub fn draw_view(&mut self, f: &mut Frame<'_>, mode_state: &mut KubeComponentState) {
        // Setup Screen Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .horizontal_margin(1)
            .constraints::<&[Constraint]>(
                [
                    Constraint::Length(2), // header
                    Constraint::Length(0), // navigation
                    Constraint::Min(1),    // results list
                    Constraint::Length(3), // input
                ]
                .as_ref(),
            )
            .split(f.area());

        let top_chunk = chunks[0];
        let top_inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(top_chunk);
        let top_inner_title = top_inner_chunks[0];
        let top_inner_nav = top_inner_chunks[1];

        // Results (Middle) Layout
        let mid_chunk = chunks[2];
        let results_inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100)])
            .split(mid_chunk);
        let mid_inner_list = results_inner_chunks[0];

        // Input (Bottom) Layout
        let bot_chunk = chunks[3];

        // Navigation
        let _nav_row_1 = [" ", "S", "A", " "];
        let _nav_row_2 = ["N", "P", "C", "L"];
        let _nav_row_3 = [" ", "D", "D", " "];

        let nav_line: Line = match self.view.view_mode {
            ViewMode::NS => Line::from(vec![
                Span::styled("N", Style::default().fg(Color::Black).bg(Color::White)),
                Span::from(" P C L"),
            ]),
            ViewMode::PODS => Line::from(vec![
                Span::from("N "),
                Span::styled("P", Style::default().fg(Color::Black).bg(Color::White)),
                Span::from(" C L"),
            ]),
            ViewMode::CONT => Line::from(vec![
                Span::from("N P "),
                Span::styled("C", Style::default().fg(Color::Black).bg(Color::White)),
                Span::from(" L"),
            ]),
            ViewMode::LOGS => Line::from(vec![
                Span::from("N P C "),
                Span::styled("L", Style::default().fg(Color::Black).bg(Color::White)),
            ]),
        };

        // TODO: So much wrong here ... this is just a mock-up.
        let top_nav_line = Line::from(Span::from("  S A  "));
        let bot_nav_line = Line::from(Span::from("  D D  "));
        let nav_text: Vec<Line<'_>> = vec![top_nav_line, nav_line, bot_nav_line];
        let nav = Paragraph::new(nav_text).alignment(Alignment::Center).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Magenta)),
        );
        // f.render_widget(nav, top_inner_nav);

        // Mock Up Inner Results Data Pane
        // let data_block = Block::bordered().border_type(BorderType::Rounded);
        // f.render_widget(data_block, results_inner_data);
        let data_view_content = format!(
            "[ NAMESPACE ] {}
[ POD ]       {}
[ CONTAINER ] {}",
            self.view.data.current_namespace_name.clone().unwrap(),
            self.view
                .data
                .current_pod_name
                .clone()
                .unwrap_or("-".to_string()),
            self.view
                .data
                .current_container_name
                .clone()
                .unwrap_or("-".to_string()),
        );
        let data_view = Paragraph::new(data_view_content)
            .style(Style::default())
            .alignment(Alignment::Left);
        let data_view_block = data_view.block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Magenta)),
        );
        // f.render_widget(data_view_block, top_inner_title);

        // Define Header / Title
        let heading_style = Style::new()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::ITALIC | Modifier::BOLD);
        let title = Paragraph::new(Text::from(Span::styled(
            "KuCo v0.1.0".to_string(),
            heading_style,
        )))
        .alignment(Alignment::Left);

        // Interaction Mode Display
        let mode: &str;
        let col: Color;
        match self.view.interact_mode {
            InteractionMode::NORMAL => {
                mode = "NORMAL";
                col = Color::White;
            }
            InteractionMode::SEARCH => {
                mode = "SEARCH";
                col = Color::Cyan;
            }
        }

        // Input Display Configuration
        let search_input_string = mode_state.search.input.as_str();

        let input = format!("[ {} ] {}", mode, search_input_string,);
        let input = Paragraph::new(input).style(Style::default().fg(col));
        let input_block =
            input.block(Block::default().title(format!("{:─>width$}", "", width = 12)));

        // Render Title
        f.render_widget(&title, chunks[0]);

        // Render List
        f.render_stateful_widget(
            self.view.clone(), // TODO: ugh, get rid of this clone later
            mid_inner_list,
            mode_state,
        );

        // Render Input Block
        f.render_widget(input_block, bot_chunk);
    }
}
