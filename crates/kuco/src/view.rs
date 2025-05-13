use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    widgets::{Block, List, ListDirection, StatefulWidget},
};

use crate::app::KucoMode;
use crate::data::{KubeComponentState, KubeData};

#[derive(Clone)]
pub struct KubeWidget {
    pub display: Option<Vec<String>>,
    pub mode: KucoMode,
    pub data: KubeData,
    pub search_mode: bool,
}

impl KubeWidget {
    pub async fn new() -> Self {
        KubeWidget {
            display: None,
            mode: KucoMode::NS,
            data: KubeData::new().await,
            search_mode: false,
        }
    }

    pub async fn update(&mut self) {
        self.data.update_all().await;

        match self.mode {
            KucoMode::NS => self.display = Some(self.data.get_namespaces()),
            KucoMode::PODS => todo!(),
            KucoMode::CONT => todo!(),
            KucoMode::LOGS => todo!(),
        }
    }
}

impl StatefulWidget for KubeWidget {
    type State = KubeComponentState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default().title_alignment(Alignment::Left);

        let mut display_list = Vec::<String>::new();
        match self.mode {
            KucoMode::NS => display_list = self.data.namespaces.names,
            KucoMode::PODS => todo!(),
            KucoMode::CONT => todo!(),
            KucoMode::LOGS => todo!(),
        }

        let list = List::new(display_list)
            .block(block)
            .style(Style::new().blue())
            .highlight_style(Style::new().italic())
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
