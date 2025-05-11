use crate::event::{AppEvent, Event, EventHandler};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    widgets::{Block, BorderType, Borders, ListState, Paragraph},
};

use kuco_k8s_backend::{PodInfo, get_pod_info};

#[derive(Debug, Clone)]
pub struct Search {
    input: String,
}

#[derive(Debug, Clone)]
pub struct KubeState {
    pub pod_info: Option<Vec<PodInfo>>,
    pub pod_list: ListState,
    pub search: Search,
}

impl KubeState {
    fn new() -> Self {
        KubeState {
            pod_info: None,
            pod_list: ListState::default(),
            search: Search {
                input: "".to_string(),
            },
        }
    }

    // TODO: Input should name itself after cluster context or something.
    pub fn build_input(&self) -> Paragraph {
        /// Max width of the UI box showing current mode
        const MAX_WIDTH: usize = 14;
        let (pref, mode) = (" ", "GLOBAL");
        let mode_width = MAX_WIDTH - pref.len();
        let input = format!("[{pref}{mode:^mode_width$}] {}", self.search.input.as_str(),);
        let input = Paragraph::new(input);

        input.block(
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .title(format!("{:─>width$}", "", width = 12)),
        )
    }
}

/// Application.
#[derive(Debug)]
pub struct KucoInterface {
    /// Is the application running?
    pub running: bool,
    /// Counter.
    pub counter: u8,
    /// Event handler.
    pub events: EventHandler,
    pub state: KubeState,
}

impl Default for KucoInterface {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            events: EventHandler::new(),
            state: KubeState::new(),
        }
    }
}

impl KucoInterface {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let pod_info_vec = get_pod_info().await.unwrap();
        self.state.pod_info = Some(pod_info_vec);

        while self.running {
            // TODO: Create update function in backend lib.
            self.state.pod_info = Some(get_pod_info().await.unwrap());
            let mut pod_data_state = self.state.clone();

            terminal.draw(|frame| {
                frame.render_stateful_widget(&self, frame.area(), &mut pod_data_state)
            })?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => self.handle_key_events(key_event)?,
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Increment => self.increment_counter(),
                    AppEvent::Decrement => self.decrement_counter(),
                    AppEvent::Quit => self.quit(),
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Right => self.events.send(AppEvent::Increment),
            KeyCode::Left => self.events.send(AppEvent::Decrement),
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

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
