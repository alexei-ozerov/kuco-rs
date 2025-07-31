use std::sync::Arc;

use kuco_sqlite_backend::{SqliteCache, SqliteDb};
use nucleo_matcher::{
    Config, Matcher,
    pattern::{CaseMatching, Normalization, Pattern},
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

use crate::data::{KubeComponentState, KubeWidgetState};
use crate::event::{AppEvent, Event, EventHandler};
use crate::view::KubeWidget;

#[derive(Debug)]
pub struct SqlitePoolCtx {
    pub cache: Arc<SqliteCache>, // KubeData in-memory cache.
    pub db: Arc<SqliteDb>, // TODO: Implement the persistence mechanisms at a later date.
}

impl SqlitePoolCtx {
   fn new(sqlite_cache: Arc<SqliteCache>, sqlite_db: Arc<SqliteDb>) -> Self {
        Self {
            cache: sqlite_cache,
            db: sqlite_db,
        }
    }
}

/// Application.
pub struct Kuco {
    pub arc_ctx: SqlitePoolCtx,
    pub running: bool,
    pub events: EventHandler,
    pub view: KubeWidget,
    pub cache: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub display: Vec<String>,
}

// TODO: Find a better place for this.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    NS,
    PODS,
    CONT,
    LOGS,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionMode {
    NORMAL,
    SEARCH,
}

impl Kuco {
    pub async fn new(sqlite_cache: Arc<SqliteCache>, sqlite_db: Arc<SqliteDb>) -> Self {
        Self {
            arc_ctx: SqlitePoolCtx::new(sqlite_cache.clone(), sqlite_db.clone()).into(),
            running: true,
            events: EventHandler::new(),
            view: KubeWidget::new(sqlite_cache.clone()).await,
            cache: None,
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        let mut kube_state = KubeWidgetState::new();

        while self.running {
            // Update widgets once the application is running
            self.view.update_widget_kube_data().await;

            // Set Mode-Specific Data
            // Using a reference here so that I don't need to copy state over and over ...
            let mode_state: &mut KubeComponentState;
            match self.view.view_mode {
                ViewMode::NS => {
                    if kube_state.namespace_state.list_state.selected().is_none() {
                        kube_state.namespace_state.list_state.select_first();
                    }
                    mode_state = &mut kube_state.namespace_state;
                    self.refresh_namespace_selection(mode_state);
                }
                ViewMode::PODS => {
                    if kube_state.pods_state.list_state.selected().is_none() {
                        kube_state.pods_state.list_state.select_first();
                    }
                    mode_state = &mut kube_state.pods_state;
                    self.refresh_pods_selection(mode_state);
                }
                ViewMode::CONT => {
                    if kube_state.containers_state.list_state.selected().is_none() {
                        kube_state.containers_state.list_state.select_first();
                    }
                    mode_state = &mut kube_state.containers_state;
                    self.refresh_containers_selection(mode_state);
                }
                ViewMode::LOGS => {
                    if kube_state.containers_state.list_state.selected().is_none() {
                        kube_state.containers_state.list_state.select_first();
                    }
                    mode_state = &mut kube_state.logs_state;
                }
            }

            // Reset search buffer
            match self.view.interact_mode {
                InteractionMode::NORMAL => {
                    mode_state.search.input = "".to_owned();
                }
                InteractionMode::SEARCH => {}
            }

            terminal.draw(|frame| {
                self.draw_view(frame, mode_state);
            })?;

            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => {
                    if let crossterm::event::Event::Key(key_event) = event {
                        self.handle_key_events(key_event, mode_state)?
                    }
                }
                Event::App(app_event) => match app_event {
                    // TODO: Implement a process that runs on another thread in a non-blocking
                    // fashion and continually updates the sqlite database with cluster
                    // information, and retool this event to pull data from the database ...
                    AppEvent::Refresh => self.view.update_widget_kube_data().await,
                    AppEvent::Quit => self.quit(),
                    AppEvent::NavRight => match self.view.view_mode {
                        ViewMode::NS => {
                            self.transition_ns_to_pod_view(mode_state).await;
                        }
                        ViewMode::PODS => {
                            self.transition_pod_to_cont_view(mode_state).await;
                        }
                        ViewMode::CONT => {
                            self.transition_cont_to_log_view(mode_state).await;
                        }
                        ViewMode::LOGS => {}
                    },
                    AppEvent::NavLeft => match self.view.view_mode {
                        ViewMode::NS => {}
                        ViewMode::PODS => {
                            self.view.view_mode = ViewMode::NS;
                            self.view.update_widget_kube_data().await;

                            // Reset pod list selection & current pod name
                            // TODO: Find a cleaner way to do this
                            self.view.data.current_pod_name = None;
                            mode_state.list_state.select(Some(0));
                        }
                        ViewMode::CONT => {
                            self.view.view_mode = ViewMode::PODS;
                            self.view.update_widget_kube_data().await;

                            self.view.data.current_container_name = None;
                            mode_state.list_state.select(Some(0));
                        }
                        ViewMode::LOGS => {
                            self.view.view_mode = ViewMode::CONT;

                            self.view.update_widget_kube_data().await;

                            self.view.data.current_log_line = None;
                            mode_state.list_state.select(Some(0));
                        }
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
                // Handle key events
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
                        if !self.view.display.clone().unwrap().is_empty()
                            && mode_state.list_state.selected().unwrap()
                                <= self.view.display.clone().unwrap().len() - 2_usize
                        {
                            mode_state.list_state.select_next()
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => mode_state.list_state.select_previous(),

                    _ => {}
                }
            }
            InteractionMode::SEARCH => {
                let matcher = Matcher::new(Config::DEFAULT.match_paths());

                // Init cache when search mode is turned on
                match self.cache {
                    Some(_) => {}
                    None => {
                        self.cache = self.view.display.clone();
                    }
                }

                match key_event.code {
                    KeyCode::Esc => {
                        self.view.interact_mode = InteractionMode::NORMAL;
                        self.cache = None; // Delete cached display list
                        self.events.send(AppEvent::Refresh);
                    }

                    KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                        self.events.send(AppEvent::Quit)
                    }

                    // Navigation
                    KeyCode::Right | KeyCode::Enter => {
                        self.events.send(AppEvent::NavRight);
                        self.view.interact_mode = InteractionMode::NORMAL;
                    }
                    KeyCode::Left => self.events.send(AppEvent::NavLeft),
                    KeyCode::Up => {
                        // Check for list length (since display and list_state.selected are set on
                        // initialization, I'm using unwrap() for now ... TODO: replace late with
                        // something less sketchy.)
                        if !self.view.display.clone().unwrap().is_empty()
                            && mode_state.list_state.selected().unwrap()
                                <= self.view.display.clone().unwrap().len() - 2_usize
                        {
                            mode_state.list_state.select_next()
                        }
                    }
                    KeyCode::Down => mode_state.list_state.select_previous(),

                    // Search Entry
                    KeyCode::Char(to_insert) => {
                        // Check if search buffer is clear or not, and swap search state if it is.
                        if !mode_state.search.input.is_empty() {
                            self.search(mode_state, matcher, self.view.display.clone());
                        }

                        mode_state.search.input += &to_insert.to_string();
                    }
                    KeyCode::Backspace => {
                        let s = &mut mode_state.search.input;
                        if !s.is_empty() {
                            s.truncate(s.len() - 1);
                            mode_state.search.input = s.to_string();
                        }
                        self.search(mode_state, matcher, self.cache.clone());
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    // TODO: build a better implementation of this ...
    fn search(
        &mut self,
        state: &mut KubeComponentState,
        mut matcher: Matcher,
        current_list: Option<Vec<String>>,
    ) {
        let pattern = &state.search.input;
        if let Some(display) = current_list {
            let matches = Pattern::parse(pattern, CaseMatching::Ignore, Normalization::Smart)
                .match_list(display, &mut matcher);

            state.list_state.select(Some(0));
            self.view.display = Some(
                matches
                    .iter()
                    .map(|matched_item| matched_item.0.to_owned())
                    .collect(),
            );
        }
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

    pub fn refresh_logs_selection(&mut self, component_state: &KubeComponentState) {
        let lo_index = component_state.list_state.selected();
        let lo_list = &self.view.display.as_ref().unwrap();

        let mut lo: &String = &"none".to_string();
        if !lo_list.is_empty() {
            lo = &lo_list[lo_index.unwrap()];
        }

        // Select Log Line
        self.view.data.current_log_line = Some(lo.clone());
    }

    pub fn refresh_containers_selection(&mut self, component_state: &KubeComponentState) {
        let co_index = component_state.list_state.selected();
        let co_list = &self.view.display.as_ref().unwrap();

        let mut co: &String = &"none".to_string();
        if !co_list.is_empty() {
            co = &co_list[co_index.unwrap()];
        }

        self.view.data.current_container_name = Some(co.clone());
    }

    pub fn refresh_pods_selection(&mut self, component_state: &KubeComponentState) {
        let po_index = component_state.list_state.selected();
        let po_list = &self.view.display.as_ref().unwrap();

        let mut po: &String = &"-".to_string();
        if !po_list.is_empty() {
            po = &po_list[po_index.unwrap()];
        }

        self.view.data.current_pod_name = Some(po.clone());
    }

    pub fn refresh_namespace_selection(&mut self, component_state: &KubeComponentState) {
        let ns_index = component_state.list_state.selected();
        let ns_list = &self.view.display.as_ref().unwrap();

        let mut ns: &String = &"default".to_string();
        if !ns_list.is_empty() {
            ns = &ns_list[ns_index.unwrap()];
        }

        // Select Namespace
        self.view.data.current_namespace_name = Some(ns.clone());
    }

    pub async fn transition_ns_to_pod_view(&mut self, component_state: &KubeComponentState) {
        tracing::debug!("VIEW: {:?}", self.view.display.clone());
        tracing::debug!("STATE: {:?}", component_state.list_state);
        self.refresh_namespace_selection(component_state); // Update Current Namespace
        self.view.update_widget_kube_data().await; // Update View
        self.view.view_mode = ViewMode::PODS;
        self.view.update_widget_kube_data().await; // Update View

        tracing::debug!("--- Refresh Event Start ---");
        tracing::debug!("Pods List: {:#?}", self.view.data.current_namespace_name);
        tracing::debug!("Pods List: {:#?}", self.view.data.pods.names);
        tracing::debug!("Pods Data: {:#?}", self.view.data.pods);
        tracing::debug!("--- Refresh Event End ---");
    }

    pub async fn transition_pod_to_cont_view(&mut self, component_state: &KubeComponentState) {
        self.refresh_pods_selection(component_state); // Update Current Pod Name
        self.view.view_mode = ViewMode::CONT;
        self.view.update_widget_kube_data().await; // Update View
    }

    pub async fn transition_cont_to_log_view(&mut self, component_state: &KubeComponentState) {
        self.refresh_containers_selection(component_state); // Update Current Container Name
        self.view.view_mode = ViewMode::LOGS;
        self.view.update_widget_kube_data().await; // Update View
    }
}
