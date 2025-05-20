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
    pub async fn new() -> Self {
        KubeWidget {
            display: None,
            view_mode: ViewMode::NS,
            interact_mode: InteractionMode::NORMAL,
            data: KubeData::new().await,
        }
    }

    pub async fn update_widget_kube_data(&mut self) {
        self.data.update_context().await;
        match self.view_mode {
            ViewMode::NS => {
                self.data.update_namespaces().await;
                self.display = Some(self.data.get_namespaces())
            }
            ViewMode::PODS => {
                self.data.update_pods_names_list().await;
                self.display = Some(self.data.get_pods())
            }
            ViewMode::CONT => {
                self.data.update_containers_names_list().await;
                self.display = Some(self.data.get_containers());
            },
            ViewMode::LOGS => self.display = Some(self.data.get_pods()),
        }
    }
}

impl StatefulWidget for KubeWidget {
    type State = KubeComponentState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default().title_alignment(Alignment::Left);

        let display_list;
        if self.display.clone().unwrap().len() == 0 as usize {
            match self.view_mode {
                ViewMode::NS => display_list = self.data.namespaces.names,
                ViewMode::PODS => display_list = self.data.pods.names,
                ViewMode::CONT => display_list = self.data.container.names,
                ViewMode::LOGS => display_list = self.data.pods.names, // TODO: Update to LOGS
            }
        } else {
            display_list = self.display.clone().unwrap();
        }

        let list = List::new(display_list)
            .block(block)
            .style(Style::new().fg(Color::Magenta))
            .highlight_style(Style::default().bold().white().on_black())
            .highlight_spacing(HighlightSpacing::Always)
            .repeat_highlight_symbol(true)
            .direction(ListDirection::BottomToTop);

        // Select first item in index automatically
        // TODO: Make this select the most used namespace
        if state.list_state.selected() == None {
            state.list_state.select(Some(0));
        }

        list.render(area, buf, &mut state.list_state);
    }
}
