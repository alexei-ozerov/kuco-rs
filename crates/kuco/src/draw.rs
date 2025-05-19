use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

use crate::{app::*, data::KubeComponentState};

impl Kuco {
    pub fn draw_view(&mut self, f: &mut Frame<'_>, mut mode_state: &mut KubeComponentState) {
        // Setup Screen Layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .horizontal_margin(1)
            .constraints::<&[Constraint]>(
                [
                    Constraint::Length(1), // header
                    Constraint::Min(1),    // results list
                    Constraint::Length(3), // input
                ]
                .as_ref(),
            )
            .split(f.area());

        let top_chunk = chunks[0];

        // Results (Middle) Layout
        let mid_chunk = chunks[1];
        let results_inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(35),
                Constraint::Percentage(10),
                Constraint::Percentage(55),
            ])
            .split(mid_chunk);
        let mid_inner_list = results_inner_chunks[0];
        let mid_inner_data = results_inner_chunks[2];

        // Input (Bottom) Layout
        let bot_chunk = chunks[2];
        let input_inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(35),
                Constraint::Percentage(10),
                Constraint::Percentage(50),
                Constraint::Percentage(5),
            ])
            .split(bot_chunk);
        let input_inner_navigation = input_inner_chunks[2];

        // Navigation
        let _nav_row_1 = vec![" ", "S", "A", " "];
        let _nav_row_2 = vec!["N", "P", "C", "L"];
        let _nav_row_3 = vec![" ", "D", "D", " "];

        let nav_line: Line;
        match self.view.view_mode {
            ViewMode::NS => {
                nav_line = Line::from(vec![
                    Span::styled("N", Style::default().fg(Color::Black).bg(Color::White)),
                    Span::from(" P C L"),
                ]);
            }
            ViewMode::PODS => {
                nav_line = Line::from(vec![
                    Span::from("N "),
                    Span::styled("P", Style::default().fg(Color::Black).bg(Color::White)),
                    Span::from(" C L"),
                ]);
            }
            ViewMode::CONT => {
                nav_line = Line::from(vec![
                    Span::from("N P "),
                    Span::styled("C", Style::default().fg(Color::Black).bg(Color::White)),
                    Span::from(" L"),
                ]);
            }
            ViewMode::LOGS => {
                nav_line = Line::from(vec![
                    Span::from("N P C "),
                    Span::styled("L", Style::default().fg(Color::Black).bg(Color::White)),
                ]);
            }
        }

        // TODO: So much wrong here ... this is just a mock-up.
        let top_nav_line = Line::from(Span::from("  S A  "));
        let bot_nav_line = Line::from(Span::from("  D D  "));
        let nav_text: Vec<Line<'_>> = vec![top_nav_line.into(), nav_line, bot_nav_line.into()];
        let nav = Paragraph::new(nav_text).alignment(Alignment::Right);
        f.render_widget(nav, input_inner_navigation);

        // Mock Up Inner Results Data Pane
        // let data_block = Block::bordered().border_type(BorderType::Rounded);
        // f.render_widget(data_block, results_inner_data);
        let data_view_content = format!(
            "{} [ NAMESPACE ]
             {} [ POD ]      
             {} [ CONTAINER ]",
            self.view.data.current_namespace.clone().unwrap(),
            self.view.data.current_pod_name.clone().unwrap_or("none".to_string()),
            self.view.data.current_container.clone().unwrap_or("none".to_string()),
        );
        let data_view = Paragraph::new(data_view_content)
            .style(Style::default())
            .alignment(Alignment::Right);
        let data_view_block = data_view.block(Block::default());
        f.render_widget(data_view_block, mid_inner_data);

        // Define Header / Title
        let heading_style = Style::new()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::ITALIC | Modifier::BOLD);
        let title = Paragraph::new(Text::from(Span::styled(
            format!("KuCo v0.1.0"),
            heading_style,
        )))
        .alignment(Alignment::Left);

        // Interaction Mode Display
        let mode: &str;
        match self.view.interact_mode {
            InteractionMode::NORMAL => {
                mode = "NORMAL";
            }
            InteractionMode::SEARCH => {
                mode = "SEARCH";
            }
        }

        // Input Display Configuration
        let search_input_string = mode_state.search.input.as_str();

        let input = format!("[ {} ] {}", mode, search_input_string,);
        let input = Paragraph::new(input).style(Style::default());
        let input_block =
            input.block(Block::default().title(format!("{:─>width$}", "", width = 12)));

        // Render Title
        f.render_widget(&title, top_chunk);

        // Render List
        f.render_stateful_widget(
            self.view.clone(), // TODO: ugh, get rid of this clone later
            mid_inner_list,
            &mut mode_state,
        );

        // Render Input Block
        f.render_widget(input_block, bot_chunk);
    }
}
