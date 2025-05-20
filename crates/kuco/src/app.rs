use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

use crate::data::{KubeComponentState, KubeWidgetState};
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
#[derive(Debug, Clone)]
pub enum ViewMode {
    NS,
    PODS,
    CONT,
    LOGS,
}

#[derive(Debug, Clone)]
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

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let mut kube_state = KubeWidgetState::new();

        self.view.update().await;

        while self.running {
            // Set Mode-Specific Data
            let mut mode_state: &mut KubeComponentState;
            match self.view.view_mode {
                ViewMode::NS => {
                    if kube_state.namespace_state.list_state.selected() == None {
                        kube_state.namespace_state.list_state.select_first();
                    }
                    mode_state = &mut kube_state.namespace_state;
                    self.refresh_namespace_selection(&mode_state);
                }
                ViewMode::PODS => {
                    if kube_state.pods_state.list_state.selected() == None {
                        kube_state.pods_state.list_state.select_first();
                    }
                    mode_state = &mut kube_state.pods_state;
                    self.refresh_pods_selection(&mode_state);
                }
                ViewMode::CONT => {
                    if kube_state.containers_state.list_state.selected() == None {
                        kube_state.containers_state.list_state.select_first();
                    }
                    mode_state = &mut kube_state.containers_state;
                }
                ViewMode::LOGS => {
                    if kube_state.logs_state.list_state.selected() == None {
                        kube_state.logs_state.list_state.select_first();
                    }
                    mode_state = &mut kube_state.logs_state;
                }
            }

            terminal.draw(|frame| {
                self.draw_view(frame, &mut mode_state);
            })?;

            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => {
                        self.handle_key_events(key_event, &mut mode_state)?
                    }
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Refresh => self.view.update().await,
                    AppEvent::Quit => self.quit(),
                    AppEvent::NavRight => match self.view.view_mode {
                        ViewMode::NS => {
                            self.view.update().await;
                            self.transition_ns_to_pod_view(&mut mode_state).await;
                        }
                        ViewMode::PODS => self.view.view_mode = ViewMode::CONT,
                        ViewMode::CONT => self.view.view_mode = ViewMode::LOGS,
                        ViewMode::LOGS => {}
                    },
                    AppEvent::NavLeft => match self.view.view_mode {
                        ViewMode::NS => {}
                        ViewMode::PODS => {
                            self.view.view_mode = ViewMode::NS;
                            self.view.update().await;

                            // Reset pod list selection & current pod name
                            // TODO: Find a cleaner way to do this
                            self.view.data.current_pod_name = None;
                            mode_state.list_state.select(Some(0));
                        },
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
        mode_state: &mut KubeComponentState,
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
                        // Check for list length (since display and list_state.selected are set on
                        // initialization, I'm using unwrap() for now ... TODO: replace late with
                        // something less sketchy.)
                        if self.view.display.clone().unwrap().len() > 0 {
                            if mode_state.list_state.selected().unwrap()
                                <= self.view.display.clone().unwrap().len() - 2 as usize
                            {
                                mode_state.list_state.select_next()
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => mode_state.list_state.select_previous(),

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
                    KeyCode::Up => {
                        // Check for list length (since display and list_state.selected are set on
                        // initialization, I'm using unwrap() for now ... TODO: replace late with
                        // something less sketchy.)
                        if self.view.display.clone().unwrap().len() > 0 {
                            if mode_state.list_state.selected().unwrap()
                                <= self.view.display.clone().unwrap().len() - 2 as usize
                            {
                                mode_state.list_state.select_next()
                            }
                        }
                    },
                    KeyCode::Down => mode_state.list_state.select_previous(),

                    // Search Entry
                    KeyCode::Char(to_insert) => {
                        // Check if search buffer is clear or not, and swap search state if it is.
                        if mode_state.search.input.len() > 0 {
                            // self.searching = true;
                        }

                        mode_state.search.input += &to_insert.to_string();
                    }
                    KeyCode::Backspace => {
                        let s = &mut mode_state.search.input;
                        if s.len() > 0 {
                            s.truncate(s.len() - 1);
                            mode_state.search.input = s.to_string();
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    // TODO: build a better implementation of this ...
    fn _search(&mut self, _state: &mut KubeWidgetState) {
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

    pub fn refresh_pods_selection(&mut self, component_state: &KubeComponentState) {
        let po_index = component_state.list_state.selected();
        let po_list = &self.view.display.as_ref().unwrap();

        let mut po: &String = &"none".to_string();
        if po_list.len() > 0 as usize {
            po = &po_list[po_index.unwrap()];
        }

        self.view.data.current_pod_name = Some(po.clone());
    }

    pub fn refresh_namespace_selection(&mut self, component_state: &KubeComponentState) {
        let ns_index = component_state.list_state.selected();
        let ns_list = &self.view.display.as_ref().unwrap();

        let mut ns: &String = &"default".to_string();
        if ns_list.len() > 0 {
            ns = &ns_list[ns_index.unwrap()];
        }

        // Select Namespace
        self.view.data.current_namespace = Some(ns.clone());
    }

    pub async fn transition_ns_to_pod_view(&mut self, component_state: &KubeComponentState) {
        self.refresh_namespace_selection(component_state); // Update Current Namespace
        self.view.update().await; // Update View
        self.view.view_mode = ViewMode::PODS;
        self.view.update().await; // Update View

        tracing::debug!("--- Refresh Event Start ---");
        tracing::debug!("Pods List: {:#?}", self.view.data.current_namespace);
        tracing::debug!("Pods List: {:#?}", self.view.data.pods.names);
        tracing::debug!("Pods Data: {:#?}", self.view.data.pods);
        tracing::debug!("--- Refresh Event End ---");
    }

    pub async fn transition_pod_to_cont_view(&mut self, component_state: &KubeComponentState) {
        self.refresh_pods_selection(component_state); // Update Current Namespace
        self.view.view_mode = ViewMode::CONT;
        self.view.update().await; // Update View
    }
}
