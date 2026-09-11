use ratatui::style::{Color, Style};
use std::cmp::Ordering;

use crate::field::FieldKind;

pub(crate) enum FieldState {
    Normal,
    Focused,
    Disabled,
    Readonly,
}

/// Which widget kind(s) a style rule applies to. Combine constants with `|`
/// to target more than one widget in a single rule, e.g.
/// `Widgets::SINGLE_LINE | Widgets::TEXT_AREA`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Widgets(u16);

impl Widgets {
    /// A `SingleLine` field.
    pub const SINGLE_LINE: Self = Self(1 << 0);
    /// A `TextArea` field.
    pub const TEXT_AREA: Self = Self(1 << 1);
    /// A `CheckBox` field.
    pub const CHECK_BOX: Self = Self(1 << 2);
    /// A `Select` field.
    pub const SELECT: Self = Self(1 << 3);
    /// A `MultiSelect` field.
    pub const MULTI_SELECT: Self = Self(1 << 4);

    /// Every widget kind — use this when a rule doesn't care which widget
    /// it applies to.
    pub const ANY: Self = Self(
        Self::SINGLE_LINE.0
            | Self::TEXT_AREA.0
            | Self::CHECK_BOX.0
            | Self::SELECT.0
            | Self::MULTI_SELECT.0,
    );

    const fn index(self) -> usize {
        self.0.trailing_zeros() as usize
    }
}

impl std::ops::BitOr for Widgets {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl From<&FieldKind> for Widgets {
    fn from(value: &FieldKind) -> Self {
        match value {
            FieldKind::SingleLine(_) => Widgets::SINGLE_LINE,
            FieldKind::CheckBox(_) => Widgets::CHECK_BOX,
            FieldKind::Select(_) => Widgets::SELECT,
            FieldKind::TextArea(_) => Widgets::TEXT_AREA,
            FieldKind::MultiSelect(_) => Widgets::MULTI_SELECT,
        }
    }
}

/// Which visual part of a widget a style rule applies to. Combine
/// constants with `|` to give more than one part the same style in a
/// single rule, e.g. `Parts::TEXT | Parts::ITEM | Parts::MARKER`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Parts(u16);

impl Parts {
    /// The field's caption.
    pub const LABEL: Self = Self(1 << 0);
    /// The validation error message shown under an invalid field.
    pub const ERROR: Self = Self(1 << 1);
    /// The surface occupied by the widget itself — e.g. the background
    /// box of a `SingleLine`/`TextArea`.
    pub const AREA: Self = Self(1 << 2);

    /// The main text content of a `SingleLine` or `TextArea`.
    pub const TEXT: Self = Self(1 << 3);
    /// Placeholder text shown while a `SingleLine`/`TextArea` is empty.
    pub const PLACEHOLDER: Self = Self(1 << 4);

    /// A symbolic marker, such as `CheckBox`'s `[x]`/`[ ]` glyph.
    pub const MARKER: Self = Self(1 << 5);

    /// One row/entry of a `Select` or `MultiSelect`.
    pub const ITEM: Self = Self(1 << 6);
    /// Whichever item currently has the cursor, regardless of selection.
    pub const ACTIVE: Self = Self(1 << 7);
    /// An item that is currently selected, regardless of cursor position.
    pub const SELECTED: Self = Self(1 << 8);

    /// Every part — use this when a rule applies no matter which part of
    /// the widget it's drawing.
    pub const ANY: Self = Self(
        Self::LABEL.0
            | Self::ERROR.0
            | Self::AREA.0
            | Self::TEXT.0
            | Self::PLACEHOLDER.0
            | Self::MARKER.0
            | Self::ITEM.0
            | Self::ACTIVE.0
            | Self::SELECTED.0,
    );

    const fn index(self) -> usize {
        self.0.trailing_zeros() as usize
    }
}

impl std::ops::BitOr for Parts {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Which field state(s) a style rule applies to, mirroring
/// `crate::style::FieldState`. Combine constants with `|` to cover more
/// than one state with the same style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct States(u8);

impl States {
    /// The field has no special state.
    pub const NORMAL: Self = Self(1 << 0);
    /// The field currently has input focus.
    pub const FOCUSED: Self = Self(1 << 1);
    /// The field is disabled.
    pub const DISABLED: Self = Self(1 << 2);
    /// The field is read-only.
    pub const READ_ONLY: Self = Self(1 << 3);

    /// Every state — use this when a rule doesn't vary by state.
    pub const ANY: Self =
        Self(Self::NORMAL.0 | Self::FOCUSED.0 | Self::DISABLED.0 | Self::READ_ONLY.0);

    const fn index(self) -> usize {
        self.0.trailing_zeros() as usize
    }
}
impl From<FieldState> for States {
    fn from(value: FieldState) -> Self {
        match value {
            FieldState::Normal => Self::NORMAL,
            FieldState::Focused => Self::FOCUSED,
            FieldState::Disabled => Self::DISABLED,
            FieldState::Readonly => Self::READ_ONLY,
        }
    }
}

const WIDGET_COUNT: usize = Widgets::ANY.0.count_ones() as usize; // 5
const PART_COUNT: usize = Parts::ANY.0.count_ones() as usize; // 9
const STATE_COUNT: usize = States::ANY.0.count_ones() as usize; // 4

impl std::ops::BitOr for States {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Builds a [`FormStyle`] one rule at a time. Start with
/// [`FormStyle::builder`].
///
/// Rules are declared in any order — [`FormStyleBuilder::build`] sorts them by
/// specificity before resolving conflicts, so you never need to think
/// about declaration order except as the tie-break between two equally
/// specific rules (the one declared last wins).
#[derive(Default)]
pub struct FormStyleBuilder {
    rules: Vec<Rule>,
}

impl FormStyleBuilder {
    /// Declares a style rule: whenever `FormStyle::get` is called with a
    /// widget, part and state that all intersect this rule's masks,
    /// `style` is a candidate for the result. Call this as many times as
    /// needed — later, more specific rules override earlier, more generic
    /// ones on the widget/part/state combinations they share.
    pub fn add(mut self, widgets: Widgets, parts: Parts, states: States, style: Style) -> Self {
        self.rules.push(Rule::new(widgets, parts, states, style));
        self
    }

    /// Consumes the builder and resolves every declared rule into a
    /// [`FormStyle`] ready to query with `StyleTree::get`.
    pub fn build(mut self) -> FormStyle {
        self.rules.sort();
        let mut product = FormStyle::new();
        for rule in self.rules.iter() {
            for widget in explode(rule.widgets.0) {
                for part in explode(rule.parts.0) {
                    for state in explode(rule.states.0 as u16) {
                        product.add(
                            Widgets(widget),
                            Parts(part),
                            States(state as u8),
                            rule.style,
                        );
                    }
                }
            }
        }
        product
    }
}

fn explode(value: u16) -> Vec<u16> {
    let mut result = Vec::with_capacity(value.count_ones() as usize);

    let mut bits = value;
    while bits != 0 {
        let bit = bits & bits.wrapping_neg();
        result.push(bit);
        bits &= bits - 1;
    }
    result
}

#[derive(Debug, Clone, Copy)]
struct Rule {
    widgets: Widgets,
    parts: Parts,
    states: States,
    style: Style,
}
impl Rule {
    fn new(widgets: Widgets, parts: Parts, states: States, style: Style) -> Self {
        Self {
            widgets,
            parts,
            states,
            style,
        }
    }
}
impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        (self.widgets, self.parts, self.states) == (other.widgets, other.parts, other.states)
    }
}
impl Eq for Rule {}
impl PartialOrd for Rule {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rule {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            std::cmp::Reverse(self.widgets.0.count_ones()),
            std::cmp::Reverse(self.parts.0.count_ones()),
            std::cmp::Reverse(self.states.0.count_ones()),
        )
            .cmp(&(
                std::cmp::Reverse(other.widgets.0.count_ones()),
                std::cmp::Reverse(other.parts.0.count_ones()),
                std::cmp::Reverse(other.states.0.count_ones()),
            ))
    }
}

/// A resolved set of style rules for a [`crate::Form`], queried with
/// `FormStyle::get`. Build one with [`FormStyle::builder`], or use
/// [`FormStyle::default`] for the built-in theme.
///
/// Internally, every rule declared through [`FormStyleBuilder`] is expanded
/// into a flat lookup table (one [`Style`] per concrete widget/part/state
/// triple), so `FormStyle::get` is a plain array index — no scanning or
/// allocation happens at resolve time.
#[derive(Debug, PartialEq, Eq)]
pub struct FormStyle {
    styles: Vec<Style>,
}

impl FormStyle {
    /// Starts building a `FormStyle` from scratch — every widget/part/state
    /// combination resolves to `Style::default()` until you add rules for
    /// it.
    pub fn builder() -> FormStyleBuilder {
        FormStyleBuilder::default()
    }

    fn new() -> Self {
        Self {
            styles: vec![Style::default(); WIDGET_COUNT * PART_COUNT * STATE_COUNT],
        }
    }

    fn add(&mut self, widget: Widgets, part: Parts, state: States, style: Style) {
        debug_assert_eq!(widget.0.count_ones(), 1);
        debug_assert_eq!(part.0.count_ones(), 1);
        debug_assert_eq!(state.0.count_ones(), 1);

        let index = (widget.index() * PART_COUNT * STATE_COUNT)
            + (part.index() * STATE_COUNT)
            + state.index();

        self.styles[index] = style;
    }

    pub(crate) fn get(&self, widget: Widgets, part: Parts, state: States) -> Style {
        debug_assert_eq!(widget.0.count_ones(), 1);
        debug_assert_eq!(part.0.count_ones(), 1);
        debug_assert_eq!(state.0.count_ones(), 1);

        let index = (widget.index() * PART_COUNT * STATE_COUNT)
            + (part.index() * STATE_COUNT)
            + state.index();

        self.styles[index]
    }
}

impl Default for FormStyle {
    /// The built-in theme: gray text everywhere, bold when a field has
    /// focus, reversed for the active row of a `Select`/`MultiSelect`,
    /// a dark background box for a focused `SingleLine`/`TextArea`, and
    /// a bold marker for `MultiSelect`. Disabled fields cross out
    /// uniformly across every part. Errors are red and bold; placeholders
    /// are gray and italic. Every widget/part/state combination has a
    /// value — build your own with [`FormStyle::builder`] for full
    /// control.
    fn default() -> Self {
        let normal = Style::default().fg(Color::Gray);
        FormStyle::builder()
            .add(
                Widgets::SINGLE_LINE | Widgets::TEXT_AREA,
                Parts::AREA,
                States::ANY,
                normal.bg(Color::from_u32(0x00303030)),
            )
            .add(Widgets::ANY, Parts::LABEL, States::FOCUSED, normal.bold())
            // VALUE
            .add(
                Widgets::ANY,
                Parts::TEXT | Parts::ITEM | Parts::MARKER,
                States::FOCUSED,
                normal.bold(),
            )
            // HIGHLIGHT
            .add(
                Widgets::ANY,
                Parts::AREA | Parts::ACTIVE,
                States::FOCUSED,
                normal.reversed(),
            )
            // ITEM decoupled for SELECT/MULTI_SELECT, so it can diverge from
            // the generic TEXT|ITEM|MARKER rule above without affecting it
            .add(
                Widgets::SELECT | Widgets::MULTI_SELECT,
                Parts::ITEM,
                States::NORMAL,
                normal,
            )
            .add(
                Widgets::SELECT | Widgets::MULTI_SELECT,
                Parts::ITEM,
                States::FOCUSED,
                normal.bold(),
            )
            // MARKER dedicated to MULTI_SELECT: bold in every non-disabled state
            .add(
                Widgets::MULTI_SELECT,
                Parts::MARKER,
                States::NORMAL | States::FOCUSED,
                normal.bold(),
            )
            // ERROR and PLACEHOLDER
            .add(
                Widgets::ANY,
                Parts::ERROR,
                States::ANY,
                Style::default().fg(Color::Red).bold(),
            )
            .add(
                Widgets::SINGLE_LINE | Widgets::TEXT_AREA,
                Parts::PLACEHOLDER,
                States::ANY,
                normal.italic(),
            )
            // Disabled fields get a uniform crossed-out look across every part
            .add(
                Widgets::ANY,
                Parts::ANY,
                States::DISABLED,
                normal.crossed_out(),
            )
            .add(Widgets::ANY, Parts::ANY, States::ANY, normal)
            .build()
    }
}

#[cfg(test)]
mod style_tree_tests {
    use super::*;

    #[test]
    fn every_widgets_constant_has_exactly_one_bit() {
        for w in [
            Widgets::SINGLE_LINE,
            Widgets::TEXT_AREA,
            Widgets::CHECK_BOX,
            Widgets::SELECT,
            Widgets::MULTI_SELECT,
        ] {
            assert_eq!(w.0.count_ones(), 1, "{w:?} it doesn't exactly have a bit");
        }
        assert_eq!(Widgets::ANY.0.count_ones() as usize, WIDGET_COUNT);
    }

    #[test]
    fn every_parts_constant_has_exactly_one_bit() {
        for p in [
            Parts::LABEL,
            Parts::ERROR,
            Parts::AREA,
            Parts::TEXT,
            Parts::PLACEHOLDER,
            Parts::MARKER,
            Parts::ITEM,
            Parts::ACTIVE,
            Parts::SELECTED,
        ] {
            assert_eq!(p.0.count_ones(), 1, "{p:?} it doesn't exactly have a bit");
        }
        assert_eq!(Parts::ANY.0.count_ones() as usize, PART_COUNT);
    }

    #[test]
    fn every_states_constant_has_exactly_one_bit() {
        for s in [
            States::NORMAL,
            States::FOCUSED,
            States::DISABLED,
            States::READ_ONLY,
        ] {
            assert_eq!(s.0.count_ones(), 1, "{s:?} it doesn't exactly have a bit");
        }
        assert_eq!(States::ANY.0.count_ones() as usize, STATE_COUNT);
    }

    #[test]
    fn explode_splits_a_combined_mask_into_single_bits() {
        assert_eq!(explode(0), Vec::<u16>::new());
        assert_eq!(explode(Widgets::SINGLE_LINE.0), vec![1]);
        assert_eq!(
            explode(Widgets::SINGLE_LINE.0 | Widgets::MULTI_SELECT.0),
            vec![1, 16]
        );
        assert_eq!(explode(Widgets::ANY.0).len(), WIDGET_COUNT);
    }

    #[test]
    fn widget_specificity_wins_even_against_a_larger_raw_bitmask() {
        let style = FormStyle::builder()
            .add(
                Widgets::SINGLE_LINE | Widgets::MULTI_SELECT,
                Parts::LABEL,
                States::NORMAL,
                Style::default().fg(Color::Blue),
            )
            .add(
                Widgets::SINGLE_LINE | Widgets::TEXT_AREA | Widgets::CHECK_BOX | Widgets::SELECT,
                Parts::LABEL,
                States::NORMAL,
                Style::default().fg(Color::Green),
            )
            .build();

        assert_eq!(
            style.get(Widgets::SINGLE_LINE, Parts::LABEL, States::NORMAL),
            Style::default().fg(Color::Blue)
        );
    }

    #[test]
    fn part_specificity_wins_over_state_specificity() {
        let style = FormStyle::builder()
            .add(
                Widgets::SINGLE_LINE,
                Parts::LABEL,
                States::ANY,
                Style::default().fg(Color::Blue),
            )
            .add(
                Widgets::SINGLE_LINE,
                Parts::ANY,
                States::FOCUSED,
                Style::default().fg(Color::Green),
            )
            .build();

        assert_eq!(
            style.get(Widgets::SINGLE_LINE, Parts::LABEL, States::FOCUSED),
            Style::default().fg(Color::Blue)
        );
    }

    #[test]
    fn equal_specificity_ties_are_won_by_the_last_declared_rule() {
        let style = FormStyle::builder()
            .add(
                Widgets::ANY,
                Parts::LABEL,
                States::FOCUSED,
                Style::default().fg(Color::Blue),
            )
            .add(
                Widgets::ANY,
                Parts::LABEL,
                States::FOCUSED,
                Style::default().fg(Color::Green),
            )
            .build();

        assert_eq!(
            style.get(Widgets::SINGLE_LINE, Parts::LABEL, States::FOCUSED),
            Style::default().fg(Color::Green)
        );
    }

    #[test]
    fn get_on_a_tree_with_no_rules_returns_default_style() {
        let style = FormStyle::builder().build();
        assert_eq!(
            style.get(Widgets::SINGLE_LINE, Parts::LABEL, States::NORMAL),
            Style::default()
        );
    }

    #[test]
    fn default_theme_overrides_area_background_for_text_widgets_only() {
        let style = FormStyle::default();
        assert_eq!(
            style.get(Widgets::SINGLE_LINE, Parts::AREA, States::NORMAL),
            Style::default()
                .fg(Color::Gray)
                .bg(Color::from_u32(0x00303030))
        );
        assert_eq!(
            style.get(Widgets::SELECT, Parts::ACTIVE, States::FOCUSED),
            Style::default().fg(Color::Gray).reversed()
        );
    }

    #[test]
    fn default_theme_applies_the_same_value_style_across_text_item_and_marker() {
        let style = FormStyle::default();
        let normal = Style::default().fg(Color::Gray);
        assert_eq!(
            style.get(Widgets::CHECK_BOX, Parts::MARKER, States::NORMAL),
            normal
        );
        assert_eq!(
            style.get(Widgets::SELECT, Parts::ITEM, States::FOCUSED),
            normal.bold()
        );
    }
}
