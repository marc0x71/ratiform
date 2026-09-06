use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, StatefulWidget, Widget},
};

#[derive(Debug, Default, Clone)]
pub struct HorizontalListState {
    selected: Option<usize>,
}

impl HorizontalListState {
    pub fn with_selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub(crate) fn select_previous(&mut self) {
        let previous = self.selected.map_or(usize::MAX, |i| i.saturating_sub(1));
        self.select(Some(previous));
    }

    pub(crate) fn select_next(&mut self) {
        let next = self.selected.map_or(0, |i| i.saturating_add(1));
        self.select(Some(next));
    }

    pub(crate) fn select_first(&mut self) {
        self.select(Some(0));
    }

    pub(crate) fn select_last(&mut self) {
        self.select(Some(usize::MAX));
    }

    pub(crate) fn scroll_up_by(&mut self, amount: u16) {
        let selected = self.selected.unwrap_or_default();
        self.select(Some(selected.saturating_sub(amount as usize)));
    }

    pub(crate) fn scroll_down_by(&mut self, amount: u16) {
        let selected = self.selected.unwrap_or_default();
        self.select(Some(selected.saturating_add(amount as usize)));
    }

    fn clamp(&mut self, min: usize, max: usize) {
        let selected = self.selected.unwrap_or_default().clamp(min, max);
        self.select(Some(selected));
    }
}

pub struct HorizontalList<'a> {
    items: Vec<Line<'a>>,
    style: Style,
    highlight_style: Style,
    spacing: usize,
    preview: usize,
}

impl<'a> HorizontalList<'a> {
    pub fn new<I, S>(items: I, spacing: usize, preview: usize) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Line<'a>>,
    {
        Self {
            items: items.into_iter().map(|s| s.into()).collect(),
            style: Style::default(),
            highlight_style: Style::default().reversed(),
            spacing,
            preview,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    #[allow(dead_code)]
    pub fn spacing(mut self, spacing: usize) -> Self {
        self.spacing = spacing;
        self
    }

    #[allow(dead_code)]
    pub fn preview(mut self, preview: usize) -> Self {
        self.preview = preview;
        self
    }
}

impl StatefulWidget for HorizontalList<'_> {
    type State = HorizontalListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.clamp(0, self.items.len().saturating_sub(1));

        let required: usize = self
            .items
            .iter()
            .take(state.selected.unwrap_or_default() + self.preview + 1)
            .map(|i| i.width() + self.spacing)
            .sum();

        let scroll_x = required.saturating_sub(area.width as usize) as u16;

        let spaces = " ".repeat(self.spacing);
        let mut items = Vec::new();
        for (i, line) in self.items.into_iter().enumerate() {
            if Some(i) == state.selected {
                for span in line.spans {
                    items.push(span.patch_style(self.highlight_style));
                }
            } else {
                items.extend(line.spans);
            };
            items.push(Span::raw(spaces.as_str()));
        }

        // buf.set_string(
        //     1,
        //     10,
        //     format!(
        //         "style={:?} highlight={:?} len={} sel={:?}",
        //         self.style,
        //         self.highlight_style,
        //         items.len(),
        //         state.selected
        //     ),
        //     Style::default(),
        // );

        Paragraph::new(Line::from(items))
            .scroll((0, scroll_x))
            .render(area, buf);

        // buf.set_string(
        //     1,
        //     1,
        //     format!(
        //         "req: {required} scroll:{scroll_x} width={} sel={:?}",
        //         area.width,
        //         state.selected()
        //     ),
        //     Style::default(),
        // );
    }
}
