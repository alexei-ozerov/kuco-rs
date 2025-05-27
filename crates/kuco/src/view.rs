use std::sync::Arc;

use kuco_sqlite_backend::SqliteCache;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, HighlightSpacing, List, ListDirection, StatefulWidget},
};

use crate::app::{InteractionMode, ViewMode};
use crate::data::{KubeComponentState, KubeData};

#[derive(Clone)]
pub struct KubeWidget {
    pub display: Option<Vec<String>>,
    pub view_mode: ViewMode,
    pub interact_mode: InteractionMode,
    pub data: KubeData,
}

impl KubeWidget {
    pub async fn new(arc_ctx: Arc<SqliteCache>) -> Self {
        KubeWidget {
            display: None,
            view_mode: ViewMode::NS,
            interact_mode: InteractionMode::NORMAL,
            data: KubeData::new(arc_ctx).await,
        }
    }

    pub async fn update_widget_kube_data(&mut self) {
        self.data.update_context().await;
        let _ = self.data.get_timestamp().await;
        match self.view_mode {
            ViewMode::NS => {
                let _ = self.data.update_namespaces_names_list().await;
                self.display = Some(self.data.get_namespaces())
            }
            ViewMode::PODS => {
                let _ = self.data.update_pods_names_list().await;
                self.display = Some(self.data.get_pods())
            }
            ViewMode::CONT => {
                self.data.update_containers_names_list().await;
                self.display = Some(self.data.get_containers());
            }
            ViewMode::LOGS => {
                self.data.update_logs_lines_list().await;
                self.display = Some(self.data.get_logs());
            }
        }
    }
}

impl StatefulWidget for KubeWidget {
    type State = KubeComponentState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let list;
        let mut reverse_list_flag = true;

        let block = Block::default().title_alignment(Alignment::Left);

        let mut display_list;
        if self.display.clone().unwrap().is_empty() {
            match self.view_mode {
                ViewMode::NS => display_list = self.data.namespace_names_list,
                ViewMode::PODS => display_list = self.data.pod_names_list,
                ViewMode::CONT => display_list = self.data.containers.names,
                ViewMode::LOGS => display_list = self.data.logs.lines,
            }
        } else {
            // TODO: Is there a way to not take a clone of self here? Cannot pass &mut self to
            // render() method
            display_list = self.display.clone().unwrap();
        }

        if self.view_mode == ViewMode::LOGS {
            reverse_list_flag = false;
        }

        if reverse_list_flag {
            list = List::new(display_list)
                .block(block)
                .style(Style::new().fg(Color::Magenta))
                .highlight_style(Style::default().bold().white().on_black())
                .highlight_spacing(HighlightSpacing::Always)
                .repeat_highlight_symbol(true)
                .direction(ListDirection::BottomToTop);

            // Select first item in index automatically
            // TODO: Make this select the most used namespace
            if state.list_state.selected().is_none() {
                state.list_state.select_first();
            }
        } else {
            // In the case of logs, the vector should be reversed to preserve the key movement.
            display_list.reverse();
            list = List::new(display_list.clone())
                .block(block)
                .style(Style::new().fg(Color::Magenta))
                .highlight_style(Style::default().bold().white().on_black())
                .highlight_spacing(HighlightSpacing::Always)
                .repeat_highlight_symbol(true)
                .direction(ListDirection::BottomToTop);

            if state.list_state.selected().is_none() {
                state.list_state.select_first();
            }
        }

        list.render(area, buf, &mut state.list_state);
    }
}
