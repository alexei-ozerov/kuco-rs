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
        let results_chunk = chunks[1];
        let input_chunk = chunks[2];

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

        // Render List
        f.render_stateful_widget(
            self.kube_data.namespaces.clone(), // TODO: ugh, get rid of this clone later
            results_chunk,
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
                state.namespace_state.search.input += &to_insert.to_string()
            }
            KeyCode::Backspace => {
                let s = &mut state.namespace_state.search.input;
                if s.len() > 0 {
                    s.truncate(s.len() - 1);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {
        // Refresh data every tick
        self.events.send(AppEvent::Refresh);
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
