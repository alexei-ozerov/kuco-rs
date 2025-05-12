use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, List, ListDirection, StatefulWidget},
};

use crate::kube_data::{NamespaceList, NamespaceListState};


impl StatefulWidget for NamespaceList {
    type State = NamespaceListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered()
            .title("kuco - kubernetes console")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        let namespaces = &self.namespaces.names;

        // TODO: Find ways to reduce the clone ...
        let list = List::new(namespaces.clone())
            .block(block)
            .style(Style::new().blue())
            .highlight_style(Style::new().italic())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::BottomToTop);

        list.render(area, buf, &mut state.ns_list_state);
    }
}
