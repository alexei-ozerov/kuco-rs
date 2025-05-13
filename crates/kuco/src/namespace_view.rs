use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    widgets::{Block, List, ListDirection, StatefulWidget},
};

use crate::kube_data::{NamespaceList, KubeComponentState};

impl StatefulWidget for NamespaceList {
    type State = KubeComponentState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::default().title_alignment(Alignment::Left);

        let namespaces = &self.namespaces.names;

        // TODO: Find ways to reduce the clone ...
        let list = List::new(namespaces.clone())
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
