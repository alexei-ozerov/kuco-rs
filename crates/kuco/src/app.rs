use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Paragraph},
};

use crate::event::{AppEvent, Event, EventHandler};
use crate::kube_data::{KubeData, KubeState};

/// Application.
pub struct KucoInterface {
    /// Is the application running?
    pub running: bool,
    /// Counter.
    pub counter: u8,
    /// Event handler.
    pub events: EventHandler,
    /// Kubernetes Data,
    pub kube_data: KubeData,
    /// App Mode
    pub mode: KucoMode,
    /// Is Searching?
    pub searching: bool,
}

// TODO: Find a better place for this.
// TODO: Add sub-modes for VIM, etc.
pub enum KucoMode {
    NS,
    PODS,
    CONT,
    LOGS,
}

impl KucoInterface {
    /// Constructs a new instance of [`App`].
    pub async fn new() -> Self {
        Self {
            mode: KucoMode::NS,
            running: true,
            counter: 0,
            events: EventHandler::new(),
            kube_data: KubeData::new().await,
            searching: false,
        }
    }

    pub fn draw_namespace_view(&mut self, f: &mut Frame<'_>, kube_state: &mut KubeState) {
        // Set Chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .horizontal_margin(1)
            .constraints::<&[Constraint]>(
                [
                    Constraint::Length(3), // header
                    Constraint::Min(1),    // results list
                    Constraint::Length(3), // input
                ]
                .as_ref(),
            )
            .split(f.area());

        let title_chunk = chunks[0];
        let results_aggregate_chunk = chunks[1];
        let input_chunk = chunks[2];

        let results_inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(50), Constraint::Percentage(25)])
            .split(results_aggregate_chunk);

        let results_inner_left = results_inner_chunks[0];
        let results_center = results_inner_chunks[1];
        let _results_inner_right = results_inner_chunks[2];

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

        // TODO: Input should name itself after cluster context or something?
        //       There is a chance cluster context name would be too long.
        let input = format!(
            "[ SEARCH ] {}",
            kube_state.namespace_state.search.input.as_str(),
        );
        let input = Paragraph::new(input).style(Style::default());

        let input_block =
            input.block(Block::default().title(format!("{:─>width$}", "", width = 12)));

        // Render Title
        f.render_widget(&title, title_chunk);

        // Render Random Stuff
        // TODO: Make this meaningful later ...
        let mode: &str;
        if self.searching {
            mode = "[ MODE ] Searching...";
        } else {
            mode = "[ MODE ] Waiting...";
        }

        let mode_paragraph = Paragraph::new(mode).style(Style::default());
        let mode_block = mode_paragraph.block(Block::default());

        f.render_widget(mode_block, _results_inner_right);

        // Render List
        f.render_stateful_widget(
            self.kube_data.namespaces.clone(), // TODO: ugh, get rid of this clone later
            results_inner_left,
            &mut kube_state.namespace_state,
        );

        // Render Input Block
        f.render_widget(input_block, input_chunk);
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let mut kube_state = KubeState::new();
        self.kube_data.update_all().await;

        while self.running {
            // Deactivate search mode when buffer is empty
            if kube_state.namespace_state.search.input.len() == 0 {
                self.searching = false;
            } else {
                self.searching = true;
                self.search(&mut kube_state);
            }

            match self.mode {
                KucoMode::NS => {
                    terminal.draw(|frame| {
                        self.draw_namespace_view(frame, &mut kube_state);
                    })?;
                }
                KucoMode::PODS => todo!(),
                KucoMode::CONT => todo!(),
                KucoMode::LOGS => todo!(),
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
                    AppEvent::Refresh => self.kube_data.update_all().await,
                    AppEvent::Quit => self.quit(),
                },
            }
        }

        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(
        &mut self,
        key_event: KeyEvent,
        state: &mut KubeState,
    ) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }

            // Navigation
            KeyCode::Right => self.events.send(AppEvent::Increment),
            KeyCode::Left => self.events.send(AppEvent::Decrement),
            KeyCode::Up => state.namespace_state.list_state.select_next(),
            KeyCode::Down => state.namespace_state.list_state.select_previous(),

            // TODO: Add modes for insert, etc., so that `q` doesn't end the program.
            KeyCode::Char(to_insert) => {
                // Check if search buffer is clear or not, and swap search state if it is.
                if state.namespace_state.search.input.len() > 0 {
                    self.searching = true;
                }

                state.namespace_state.search.input += &to_insert.to_string();
            }
            KeyCode::Backspace => {
                let s = &mut state.namespace_state.search.input;
                if s.len() > 0 {
                    s.truncate(s.len() - 1);
                    state.namespace_state.search.input = s.to_string();
                }

                if self.searching {
                    self.events.send(AppEvent::Refresh); // TODO: Replace this with less time consuming approach
                }
            }
            _ => {}
        }
        Ok(())
    }

    // TODO: build a better implementation of this ...
    fn search(&mut self, state: &mut KubeState) {
        let mut parsed_namespaces = Vec::new();
        let ns_ref = self.kube_data.namespaces.get_namespaces_vector();
        ns_ref.iter().for_each(|ns| {
            if ns.contains(&state.namespace_state.search.input) {
                parsed_namespaces.push(ns.clone());
            }
        });

        self.kube_data.namespaces.namespaces.names = parsed_namespaces;
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {
        // Refresh data every tick
        // TODO: Find a better solution to prevent overwriting a current search.
        //       Perhaps set a mode / app state?

        if self.searching {
            // Do not send the refresh event ...
        } else {
            self.events.send(AppEvent::Refresh);
        }
    }

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
