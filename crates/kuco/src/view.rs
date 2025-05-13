use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Style, Stylize},
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
    pub search_mode: bool,
}

impl KubeWidget {
    pub async fn new() -> Self {
        KubeWidget {
            display: None,
            view_mode: ViewMode::NS,
            interact_mode: InteractionMode::NORMAL,
            data: KubeData::new().await,
            search_mode: false,
        }
    }

    pub async fn update(&mut self) {
        self.data.update_all().await;

        match self.view_mode {
            ViewMode::NS => self.display = Some(self.data.get_namespaces()),
            ViewMode::PODS => todo!(),
            ViewMode::CONT => todo!(),
            ViewMode::LOGS => todo!(),
        }
    }
}

impl StatefulWidget for KubeWidget {
    type State = KubeComponentState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default().title_alignment(Alignment::Left);

        let mut display_list = Vec::<String>::new();
        match self.view_mode {
            ViewMode::NS => display_list = self.data.namespaces.names,
            ViewMode::PODS => todo!(),
            ViewMode::CONT => todo!(),
            ViewMode::LOGS => todo!(),
        }

        let list = List::new(display_list)
            .block(block)
            .style(Style::new().blue())
            .highlight_style(Style::default().bold().white().on_black())
            // .highlight_style(Style::new().white())
            .highlight_spacing(HighlightSpacing::Always)
            .repeat_highlight_symbol(true)
            .direction(ListDirection::BottomToTop);

        // Select first item in index automatically
        // TODO: Make this select the most used namespace?
        //       Or should that re-order the vector?
        //       Should I be using a list or a table for this from Ratatui?
        if state.list_state.selected() == None {
            state.list_state.select(Some(0));
        }

        list.render(area, buf, &mut state.list_state);
    }
}
