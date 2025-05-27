use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

use crate::{app::*, constants::KUCO_VERSION, data::KubeComponentState};

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
                    Constraint::Length(0), // navigation TODO: remove this
                    Constraint::Min(1),    // results list
                    Constraint::Length(3), // input
                ]
                .as_ref(),
            )
            .split(f.area());

        // Results (Middle) Layout
        let mid_chunk = chunks[2];
        let results_inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100)])
            .split(mid_chunk);
        let mid_inner_list = results_inner_chunks[0];

        // Input (Bottom) Layout
        let bot_chunk = chunks[3];

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
        let mut search_input_string = mode_state.search.input.as_str();

        // TODO: Make this more elegant later ...
        let mut navigation = "".to_owned();
        if self.view.interact_mode == InteractionMode::NORMAL {
            navigation = match self.view.view_mode {
                ViewMode::NS => self
                    .view
                    .data
                    .current_namespace_name
                    .clone()
                    .unwrap_or("".to_owned()),
                ViewMode::PODS => {
                    let ns = self
                        .view
                        .data
                        .current_namespace_name
                        .clone()
                        .unwrap_or("".to_owned());
                    let po = self
                        .view
                        .data
                        .current_pod_name
                        .clone()
                        .unwrap_or("".to_owned());
                    format!("{} > {}", ns, po)
                }
                ViewMode::CONT => {
                    let ns = self
                        .view
                        .data
                        .current_namespace_name
                        .clone()
                        .unwrap_or("".to_owned());
                    let po = self
                        .view
                        .data
                        .current_pod_name
                        .clone()
                        .unwrap_or("".to_owned());
                    let co = self
                        .view
                        .data
                        .current_container_name
                        .clone()
                        .unwrap_or("".to_owned());
                    format!("{} > {} > {}", ns, po, co)
                }
                ViewMode::LOGS => {
                    let ns = self
                        .view
                        .data
                        .current_namespace_name
                        .clone()
                        .unwrap_or("".to_owned());
                    let po = self
                        .view
                        .data
                        .current_pod_name
                        .clone()
                        .unwrap_or("".to_owned());
                    let co = self
                        .view
                        .data
                        .current_container_name
                        .clone()
                        .unwrap_or("".to_owned());
                    format!("{} > {} > {}", ns, po, co)
                }
            };
            search_input_string = &navigation;
        };

        let input = format!("[ {} ] {}", mode, search_input_string,);
        let input = Paragraph::new(input).style(Style::default().fg(col));
        let input_block =
            input.block(Block::default().title(format!("{:─>width$}", "", width = 12)));

        // Render Title
        let title_chunk = chunks[0];
        let title_inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(title_chunk);
        let title_block = title_inner_chunks[0];
        let refresh_block = title_inner_chunks[1];

        // Define Header / Title
        let heading_style = Style::new()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::ITALIC | Modifier::BOLD);
        let title = Paragraph::new(Text::from(Span::styled(
            format!("KuCo v{}", KUCO_VERSION),
            heading_style,
        )))
        .alignment(Alignment::Left);
        f.render_widget(&title, title_block);

        // Define Refresh Header
        let refresh_style = Style::new().fg(Color::Gray).add_modifier(Modifier::ITALIC);
        let help_style = Style::new()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::ITALIC);

        let refresh_content = if self.view.data.last_refreshed_at == *"19:00:00" {
            Paragraph::new(Text::from(Span::styled(
                "loading cache ...".to_string(),
                refresh_style,
            )))
            .alignment(Alignment::Right)
        } else {
            let text = vec![
                Line::styled(
                    format!("last refreshed at {:#}", self.view.data.last_refreshed_at),
                    refresh_style,
                ),
                Line::styled("press 'r' to refresh".to_string(), help_style),
            ];

            Paragraph::new(Text::from(text)).alignment(Alignment::Right)
        };

        f.render_widget(&refresh_content, refresh_block);

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
