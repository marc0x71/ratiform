use ratatui::style::{Color, Style};

pub(crate) enum FieldState {
    Normal,
    Focused,
    Disabled,
    Readonly,
}

/// The color/style theme for a [`crate::Form`], applied via
/// [`crate::Form::with_style`]. Build one with [`FormStyle::builder`], or
/// use [`FormStyle::default`] for the built-in theme.
#[derive(Debug, Clone, Copy)]
pub struct FormStyle {
    /// The field's caption, on the left.
    pub label: FieldStyle,
    /// The field's own content: the text color of a `SingleLine` field, a
    /// `Select` field's list items, and a `Checkbox`'s `[✓]`/`[ ]` glyph.
    pub value: FieldStyle,
    /// Emphasis for whatever is "active right now": the background box
    /// behind a `SingleLine` field, and the currently selected row of a
    /// `Select` list.
    pub highlight: FieldStyle,
    /// Placeholder/hint text style, shown in place of `value`'s style
    /// while a field is empty. Used today by `SingleLine`'s placeholder
    /// text.
    pub placeholder: Style,
    /// The validation error message shown under an invalid field.
    pub error: Style,
}
impl FormStyle {
    /// Starts building a `FormStyle` from scratch — every area is
    /// `Style::default()`/`FieldStyle::default()` until you set it.
    pub fn builder() -> FormStyleBuilder {
        FormStyleBuilder::default()
    }
}
impl Default for FormStyle {
    /// The built-in theme: gray text everywhere, bold when a field has
    /// focus. `label` also crosses out on `disabled()` fields. Errors are
    /// red and bold; placeholders are gray and italic. Not every state is
    /// covered for every area — `readonly`, for instance, isn't styled
    /// differently from `normal` here — build your own with
    /// [`FormStyle::builder`] for full control.
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

/// One `Style` per field state, used for the `label`, `value` and
/// `highlight` areas of a [`FormStyle`]. You don't need to pick which one
/// applies yourself — it's resolved automatically, with `disabled` taking
/// priority over `readonly`, which takes priority over `focused`.
#[derive(Debug, Clone, Copy, Default)]
pub struct FieldStyle {
    pub normal: Style,
    pub focused: Style,
    pub disabled: Style,
    pub readonly: Style,
}

impl FieldStyle {
    /// Starts building a `FieldStyle` from scratch — every state is
    /// `Style::default()` until you set it.
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
    /// Equivalent to [`FormStyle::builder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `label` area — see [`FormStyle::label`].
    pub fn label(mut self, style: FieldStyle) -> Self {
        self.label = style;
        self
    }

    /// Sets the `value` area — see [`FormStyle::value`].
    pub fn value(mut self, style: FieldStyle) -> Self {
        self.value = style;
        self
    }

    /// Sets the `highlight` area — see [`FormStyle::highlight`].
    pub fn highlight(mut self, style: FieldStyle) -> Self {
        self.highlight = style;
        self
    }

    /// Sets the `error` area — see [`FormStyle::error`].
    pub fn error(mut self, style: Style) -> Self {
        self.error = style;
        self
    }

    /// Sets the `placeholder` area — see [`FormStyle::placeholder`].
    pub fn placeholder(mut self, style: Style) -> Self {
        self.placeholder = style;
        self
    }

    /// Finishes building the `FormStyle`.
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

/// Builder for a [`FieldStyle`]. Start with [`FieldStyle::builder`].
#[derive(Debug, Clone, Copy, Default)]
pub struct FieldStyleBuilder {
    normal: Style,
    focused: Style,
    disabled: Style,
    readonly: Style,
}

impl FieldStyleBuilder {
    /// Equivalent to [`FieldStyle::builder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the style for a field with no other state — see
    /// [`FieldStyle::normal`].
    pub fn normal(mut self, style: Style) -> Self {
        self.normal = style;
        self
    }

    /// Sets the style for the focused field — see [`FieldStyle::focused`].
    pub fn focused(mut self, style: Style) -> Self {
        self.focused = style;
        self
    }

    /// Sets the style for a disabled field — see [`FieldStyle::disabled`].
    pub fn disabled(mut self, style: Style) -> Self {
        self.disabled = style;
        self
    }

    /// Sets the style for a readonly field — see [`FieldStyle::readonly`].
    pub fn readonly(mut self, style: Style) -> Self {
        self.readonly = style;
        self
    }

    /// Finishes building the `FieldStyle`.
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
