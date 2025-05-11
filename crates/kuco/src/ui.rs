use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, List, ListDirection, StatefulWidget},
};

use crate::app::{KubeState, KucoInterface};

impl StatefulWidget for &KucoInterface {
    type State = KubeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered()
            .title("kuco - kubernetes console")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        // let pod_info = match &state.pod_info {
        //     Some(pod_info_data) => {
        //         let pod_names_vec: Vec<String> =
        //             pod_info_data.iter().map(|po| po.name.clone()).collect();
        //         pod_names_vec
        //     }
        //     None => {
        //         let error_string = "[ERROR] Figure out how to handle this later.".to_string();
        //         let mut pod_names_vec: Vec<String> = Vec::new();
        //         pod_names_vec.push(error_string);
        //         pod_names_vec
        //     }
        // };
        
        let namespaces = &state.data.namespaces.names;

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
