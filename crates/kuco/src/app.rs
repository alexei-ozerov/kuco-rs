use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Paragraph},
};

use crate::data::KubeWidgetState;
use crate::event::{AppEvent, Event, EventHandler};
use crate::view::KubeWidget;

/// Application.
pub struct Kuco {
    /// Is the application running?
    pub running: bool,
    /// Counter.
    pub counter: u8,
    /// Event handler.
    pub events: EventHandler,
    /// Kube Widget Data
    pub view: KubeWidget,
}

// TODO: Find a better place for this.
#[derive(Clone)]
pub enum ViewMode {
    NS,
    PODS,
    CONT,
    LOGS,
}

#[derive(Clone)]
pub enum InteractionMode {
    NORMAL,
    SEARCH,
}

impl Kuco {
    pub async fn new() -> Self {
        Self {
            running: true,
            counter: 0,
            events: EventHandler::new(),
            view: KubeWidget::new().await,
        }
    }

    pub fn draw_view(&mut self, f: &mut Frame<'_>, kube_state: &mut KubeWidgetState) {
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

        let input = format!(
            "[ {} ] {}",
            mode,
            kube_state.namespace_state.search.input.as_str(),
        );
        let input = Paragraph::new(input).style(Style::default());
        let input_block =
            input.block(Block::default().title(format!("{:─>width$}", "", width = 12)));

        // Render Title
        f.render_widget(&title, top_chunk);

        // Render List
        f.render_stateful_widget(
            self.view.clone(), // TODO: ugh, get rid of this clone later
            mid_inner_list,
            &mut kube_state.namespace_state,
        );

        // Render Input Block
        f.render_widget(input_block, bot_chunk);
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let mut kube_state = KubeWidgetState::new();
        self.view.update().await;

        while self.running {
            // Deactivate search mode when buffer is empty
            if kube_state.namespace_state.search.input.len() == 0 {
                self.view.search_mode = false;
            } else {
                // self.searching = true;
                // self.search(&mut kube_state);
            }

            match self.view.view_mode {
                ViewMode::NS => {
                    terminal.draw(|frame| {
                        self.draw_view(frame, &mut kube_state);
                    })?;
                }
                ViewMode::PODS => {
                    terminal.draw(|frame| {
                        self.draw_view(frame, &mut kube_state);
                    })?;
                }
                ViewMode::CONT => {
                    terminal.draw(|frame| {
                        self.draw_view(frame, &mut kube_state);
                    })?;
                }
                ViewMode::LOGS => {
                    terminal.draw(|frame| {
                        self.draw_view(frame, &mut kube_state);
                    })?;
                }
            }

            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => {
                        self.handle_key_events(key_event, &mut kube_state)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Increment => self.increment_counter(),
                    AppEvent::Decrement => self.decrement_counter(),
                    AppEvent::Refresh => self.view.update().await,
                    AppEvent::Quit => self.quit(),
                    AppEvent::NavRight => match self.view.view_mode {
                        ViewMode::NS => self.view.view_mode = ViewMode::PODS,
                        ViewMode::PODS => self.view.view_mode = ViewMode::CONT,
                        ViewMode::CONT => self.view.view_mode = ViewMode::LOGS,
                        ViewMode::LOGS => {}
                    },
                    AppEvent::NavLeft => match self.view.view_mode {
                        ViewMode::NS => {}
                        ViewMode::PODS => self.view.view_mode = ViewMode::NS,
                        ViewMode::CONT => self.view.view_mode = ViewMode::PODS,
                        ViewMode::LOGS => self.view.view_mode = ViewMode::CONT,
                    },
                },
            }
        }

        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(
        &mut self,
        key_event: KeyEvent,
        state: &mut KubeWidgetState,
    ) -> color_eyre::Result<()> {
        match self.view.interact_mode {
            InteractionMode::NORMAL => {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                    KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                        self.events.send(AppEvent::Quit)
                    }

                    // Refresh
                    KeyCode::Char('r') => self.events.send(AppEvent::Refresh),

                    // Modes
                    KeyCode::Char('/') => self.view.interact_mode = InteractionMode::SEARCH,

                    // Navigation
                    KeyCode::Right | KeyCode::Char('l') => self.events.send(AppEvent::NavRight),
                    KeyCode::Left | KeyCode::Char('h') => self.events.send(AppEvent::NavLeft),
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.namespace_state.list_state.select_next()
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.namespace_state.list_state.select_previous()
                    }

                    _ => {}
                }
            }
            InteractionMode::SEARCH => {
                match key_event.code {
                    KeyCode::Esc => self.view.interact_mode = InteractionMode::NORMAL,

                    KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                        self.events.send(AppEvent::Quit)
                    }

                    // Navigation
                    KeyCode::Right => self.events.send(AppEvent::NavRight),
                    KeyCode::Left => self.events.send(AppEvent::NavLeft),
                    KeyCode::Up => state.namespace_state.list_state.select_next(),
                    KeyCode::Down => state.namespace_state.list_state.select_previous(),

                    // Search Entry
                    KeyCode::Char(to_insert) => {
                        // Check if search buffer is clear or not, and swap search state if it is.
                        if state.namespace_state.search.input.len() > 0 {
                            // self.searching = true;
                        }

                        state.namespace_state.search.input += &to_insert.to_string();
                    }
                    KeyCode::Backspace => {
                        let s = &mut state.namespace_state.search.input;
                        if s.len() > 0 {
                            s.truncate(s.len() - 1);
                            state.namespace_state.search.input = s.to_string();
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    // TODO: build a better implementation of this ...
    fn search(&mut self, state: &mut KubeWidgetState) {
        // let ns_ref: &mut Vec<String> = &mut self.kube_widget.data.namespaces.names;
        // let ns_new_arc_ref = Arc::new(Mutex::new(Vec::<String>::new()));
        //
        // ns_ref.par_iter_mut().for_each(|ns| {
        //     if ns.contains(&state.namespace_state.search.input) {
        //         ns_new_arc_ref.lock().expect("[ERROR] Some multithreading stuff crashed.").push(ns.to_string());
        //     }
        // });
        //
        // let inner: Vec<_> = Arc::try_unwrap(ns_new_arc_ref).unwrap().into_inner().unwrap();
        // self.kube_data.namespaces.names = inner;
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    pub fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }
}
