use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

use crate::event::{AppEvent, Event, EventHandler};
use crate::kube_data::{KubeData, KubeState};

#[derive(Debug, Clone)]
pub struct Search {
    input: String,
}

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

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let mut kube_state = KubeState::new();
        self.kube_data.update_all().await;

        while self.running {
            match self.mode {
                KucoMode::NS => {
                    terminal
                        .draw(|frame| 
                            frame.render_stateful_widget(
                                self.kube_data.namespace_list.clone(), // TODO: ugh, get rid of
                                                                       // this later
                                frame.area(), 
                                &mut kube_state.namespace_state
                                )
                            )?;
                }
                KucoMode::PODS => todo!(),
                KucoMode::CONT => todo!(),
                KucoMode::LOGS => todo!(),
            }

            match self.events.next().await? {
                Event::Tick => self.tick().await,
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
    // TODO: I'm pretty sure this shouldn't be async and awaiting ........ something is very wrong.
    pub async fn tick(&mut self) {
        self.kube_data.update_all().await;
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

//     // TODO: Input should name itself after cluster context or something.
//     pub fn build_input(&self) -> Paragraph {
//         /// Max width of the UI box showing current mode
//         const MAX_WIDTH: usize = 14;
//         let (pref, mode) = (" ", "GLOBAL");
//         let mode_width = MAX_WIDTH - pref.len();
//         let input = format!("[{pref}{mode:^mode_width$}] {}", self.search.input.as_str(),);
//         let input = Paragraph::new(input);
//
//         input.block(
//             Block::default()
//                 .borders(Borders::LEFT | Borders::RIGHT)
//                 .border_type(BorderType::Rounded)
//                 .title(format!("{:─>width$}", "", width = 12)),
//         )
//     }
// }
