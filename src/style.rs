use ratatui::style::{Color, Modifier, Style};

pub(crate) enum FieldState {
    Normal,
    Focused,
    Disabled,
    Readonly,
}

#[derive(Debug, Clone, Copy)]
pub struct FormStyle {
    pub label: FieldStyle,
    pub value: FieldStyle,
    pub highlight: FieldStyle,
    pub placeholder: Style,
    pub error: Style,
}
impl FormStyle {
    pub fn builder() -> FormStyleBuilder {
        FormStyleBuilder::default()
    }
}
impl Default for FormStyle {
    fn default() -> Self {
        let normal = Style::default().fg(Color::Gray);
        FormStyle::builder()
            .label(
                FieldStyle::builder()
                    .normal(normal)
                    .focused(normal.bold())
                    .disabled(normal.crossed_out())
                    .build(),
            )
            .value(
                FieldStyle::builder()
                    .normal(normal)
                    .focused(normal.bold())
                    .build(),
            )
            .highlight(
                FieldStyle::builder()
                    .normal(normal)
                    .focused(normal.reversed())
                    .build(),
            )
            .error(Style::default().fg(Color::Red).bold())
            .placeholder(normal.italic())
            .build()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FieldStyle {
    pub normal: Style,
    pub focused: Style,
    pub disabled: Style,
    pub readonly: Style,
}
impl FieldStyle {
    pub fn builder() -> FieldStyleBuilder {
        FieldStyleBuilder::default()
    }
    pub(crate) fn style_for(&self, state: &FieldState) -> Style {
        match state {
            FieldState::Normal => self.normal,
            FieldState::Focused => self.focused,
            FieldState::Disabled => self.disabled,
            FieldState::Readonly => self.readonly,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FormStyleBuilder {
    label: FieldStyle,
    value: FieldStyle,
    highlight: FieldStyle,
    error: Style,
    placeholder: Style,
}

impl FormStyleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, style: FieldStyle) -> Self {
        self.label = style;
        self
    }

    pub fn value(mut self, style: FieldStyle) -> Self {
        self.value = style;
        self
    }

    pub fn highlight(mut self, style: FieldStyle) -> Self {
        self.highlight = style;
        self
    }

    pub fn error(mut self, style: Style) -> Self {
        self.error = style;
        self
    }

    pub fn placeholder(mut self, style: Style) -> Self {
        self.placeholder = style;
        self
    }

    pub fn build(self) -> FormStyle {
        FormStyle {
            label: self.label,
            value: self.value,
            highlight: self.highlight,
            error: self.error,
            placeholder: self.placeholder,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FieldStyleBuilder {
    normal: Style,
    focused: Style,
    disabled: Style,
    readonly: Style,
}

impl FieldStyleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn normal(mut self, style: Style) -> Self {
        self.normal = style;
        self
    }

    pub fn focused(mut self, style: Style) -> Self {
        self.focused = style;
        self
    }

    pub fn disabled(mut self, style: Style) -> Self {
        self.disabled = style;
        self
    }

    pub fn readonly(mut self, style: Style) -> Self {
        self.readonly = style;
        self
    }

    pub fn build(self) -> FieldStyle {
        FieldStyle {
            normal: self.normal,
            focused: self.focused,
            disabled: self.disabled,
            readonly: self.readonly,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn field_style_returns_style_for_state() {
        let normal = Style::default();
        let focused = Style::default().bold();
        let disabled = Style::default().dim();
        let readonly = Style::default().italic();

        let style = FieldStyle::builder()
            .normal(normal)
            .focused(focused)
            .disabled(disabled)
            .readonly(readonly)
            .build();

        assert_eq!(style.style_for(&FieldState::Normal), normal);
        assert_eq!(style.style_for(&FieldState::Focused), focused);
        assert_eq!(style.style_for(&FieldState::Disabled), disabled);
        assert_eq!(style.style_for(&FieldState::Readonly), readonly);
    }
}
