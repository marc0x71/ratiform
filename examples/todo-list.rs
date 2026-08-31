use std::str::FromStr;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};
use ratiform::{Form, FormLayout, FormResult, FormState, builder::FormBuilder};

const PRIORITIES: &[(&str, &str)] = &[("low", "Low"), ("medium", "Medium"), ("high", "High")];

#[derive(Debug, Clone)]
struct Todo {
    title: String,
    description: String,
    priority: Priority,
    done: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    fn as_value(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
        }
    }
}

impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(Self::Low),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            value => Err(format!("invalid priority: {value}")), // this should be impossible! :)
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
enum TaskField {
    Title,
    Description,
    Priority,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Mode {
    Browse,
    Add,
    Edit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}

struct App {
    form: FormState<TaskField>,
    todos: Vec<Todo>,
    selected: usize,
    mode: Mode,
}

impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let form = FormBuilder::new()
            .single_line(TaskField::Title, "Title")
            .placeholder("What needs to be done?")
            .required("Title cannot be empty".to_owned())
            .text_area(TaskField::Description, "Description")
            .placeholder("Optional details")
            .height(4)
            .optional()
            .select(TaskField::Priority, "Priority")
            .values_ref(PRIORITIES)
            .selected(1)
            .height(3)
            .build()?;

        let todos = vec![
            Todo {
                title: "Try ratiform".to_owned(),
                description: "Build a small example application".to_owned(),
                priority: Priority::High,
                done: true,
            },
            Todo {
                title: "Add a todo".to_owned(),
                description: "Use the form to create another task".to_owned(),
                priority: Priority::Medium,
                done: false,
            },
        ];

        Ok(Self {
            form,
            todos,
            selected: 0,
            mode: Mode::Browse,
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            if self.handle_events()? {
                break;
            }
        }
        Ok(())
    }

    fn save(&mut self) {
        let title = self.form.value(&TaskField::Title).unwrap_or_default();
        let description = self.form.value(&TaskField::Description).unwrap_or_default();
        let priority = match self.form.value_as(&TaskField::Priority) {
            Some(Ok(p)) => p,
            Some(Err(err)) => {
                // for `Select` this should be impossible! :)
                panic!("invalid priority value: {err}");
            }
            None => Priority::Medium,
        };

        // alternatively we can also use:
        //
        // let priority = self
        //     .form
        //     .value_as::<Priority>(&TaskField::Priority)
        //     .expect("priority field should have a value")
        //     .expect("priority value should be valid");

        match self.mode {
            Mode::Add => {
                self.todos.push(Todo {
                    title,
                    description,
                    priority,
                    done: false,
                });
                self.selected = self.todos.len() - 1;
            }

            Mode::Edit => {
                let todo = &mut self.todos[self.selected];
                todo.title = title;
                todo.description = description;
                todo.priority = priority;
            }

            Mode::Browse => {
                unreachable!()
            }
        }

        self.mode = Mode::Browse;
    }

    fn render(&mut self, frame: &mut Frame<'_>) {
        let [header, list_area, action_area, footer] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(match self.mode {
                    Mode::Browse => 5,
                    Mode::Add | Mode::Edit => 14,
                }),
                Constraint::Length(1),
            ])
            .areas(frame.area());

        frame.render_widget(
            Paragraph::new("A tiny in-memory todo app built with Ratatui + ratiform").block(
                Block::default()
                    .title(" ratiform todo ")
                    .borders(Borders::ALL),
            ),
            header,
        );

        let todo_lines = if self.todos.is_empty() {
            vec![Line::from(Span::styled(
                "No tasks yet — press a to add one.",
                Style::default().add_modifier(Modifier::DIM),
            ))]
        } else {
            self.todos
                .iter()
                .enumerate()
                .flat_map(|(index, todo)| {
                    let cursor = if index == self.selected { "› " } else { "  " };
                    let checkbox = if todo.done { "[✓]" } else { "[ ]" };

                    let title_style = if todo.done {
                        Style::default().add_modifier(Modifier::DIM | Modifier::CROSSED_OUT)
                    } else if index == self.selected {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let mut lines = vec![Line::from(vec![
                        Span::styled(cursor, title_style),
                        Span::styled(format!("{checkbox} "), title_style),
                        Span::raw(format!("[{}] ", todo.priority.label())),
                        Span::styled(todo.title.clone(), title_style),
                    ])];

                    if !todo.description.is_empty() {
                        lines.push(Line::from(vec![
                            Span::raw("      "),
                            Span::styled(
                                todo.description.clone(),
                                Style::default().add_modifier(Modifier::DIM),
                            ),
                        ]));
                    }

                    lines
                })
                .collect()
        };

        frame.render_widget(
            Paragraph::new(todo_lines).block(
                Block::default()
                    .title(format!(" Tasks ({}) ", self.todos.len()))
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1)),
            ),
            list_area,
        );

        match self.mode {
            Mode::Browse => {
                frame.render_widget(
                    Paragraph::new(vec![
                        Line::from("[a] add      ·   [e] edit"),
                        Line::from("[↑/↓] select ·   [Space] toggle"),
                        Line::from("[d] delete   ·   [q] quit"),
                    ])
                    .block(Block::default().title(" Actions ").borders(Borders::ALL)),
                    action_area,
                );
            }

            Mode::Add | Mode::Edit => {
                let title = match self.mode {
                    Mode::Add => " Add task ",
                    Mode::Edit => " Edit task ",
                    Mode::Browse => unreachable!(),
                };

                let block = Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .padding(Padding::uniform(1));

                let inner = block.inner(action_area);

                frame.render_widget(block, action_area);

                frame.render_stateful_widget(
                    Form::default().with_layout(FormLayout::Stacked),
                    inner,
                    &mut self.form,
                );

                if let Some(position) = self.form.cursor_position() {
                    frame.set_cursor_position(position);
                }
            }
        }

        let help = match self.mode {
            Mode::Browse => "q quit",
            Mode::Add => "Tab next field · Ctrl+Enter add · Esc cancel",
            Mode::Edit => "Tab next field · Ctrl+Enter save · Esc cancel",
        };

        frame.render_widget(Paragraph::new(help), footer);
    }

    fn handle_events(&mut self) -> std::io::Result<bool> {
        let Event::Key(key) = event::read()? else {
            return Ok(false);
        };

        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }

        match self.mode {
            Mode::Browse => match key.code {
                KeyCode::Char('q') => {
                    return Ok(true);
                }
                KeyCode::Char('a') => {
                    self.form.reset();
                    self.mode = Mode::Add;
                }
                KeyCode::Char('e') if !self.todos.is_empty() => {
                    self.form.reset();
                    self.form
                        .set_value(&TaskField::Title, &self.todos[self.selected].title);
                    self.form.set_value(
                        &TaskField::Description,
                        &self.todos[self.selected].description,
                    );
                    self.form.set_value(
                        &TaskField::Priority,
                        self.todos[self.selected].priority.as_value(),
                    );
                    self.mode = Mode::Edit;
                }
                KeyCode::Up if !self.todos.is_empty() => {
                    self.selected = self.selected.saturating_sub(1);
                }
                KeyCode::Down if !self.todos.is_empty() => {
                    self.selected = (self.selected + 1).min(self.todos.len() - 1);
                }
                KeyCode::Char(' ') if !self.todos.is_empty() => {
                    self.todos[self.selected].done = !self.todos[self.selected].done;
                }
                KeyCode::Char('d') if !self.todos.is_empty() => {
                    self.todos.remove(self.selected);
                    if self.todos.is_empty() {
                        self.selected = 0;
                    } else {
                        self.selected = self.selected.min(self.todos.len() - 1);
                    }
                }
                _ => {}
            },

            Mode::Add | Mode::Edit => {
                self.form.handle_input(key);
                match self.form.result() {
                    FormResult::Submitted => {
                        self.save();
                    }

                    FormResult::Cancelled => {
                        self.mode = Mode::Browse;
                    }

                    FormResult::Working => {}
                }
            }
        }
        Ok(false)
    }
}
