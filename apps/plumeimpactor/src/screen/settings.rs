use std::collections::HashMap;

use iced::widget::{
    button, canvas, checkbox, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Color, Element, Fill, Length, Point, Rectangle, Size, Task, mouse, touch};
use plume_store::AccountStore;

use crate::appearance;

const ACCOUNT_ACTION_BUTTON_WIDTH: f32 = 165.0;
const ACCOUNT_LIST_HEIGHT: f32 = 210.0;
const ACCENT_CARD_HORIZONTAL_PADDING: f32 = 6.0;
const ACCENT_PICKER_HEIGHT: f32 = 212.0;
const ACCENT_HUE_STRIP_WIDTH: f32 = 28.0;
const ACCENT_SIDE_PANEL_WIDTH: f32 = 116.0;
const ACCENT_SWATCH_HEIGHT: f32 = 68.0;
const ACCENT_PRESET_COLUMNS: usize = 4;
const ACCENT_PRESET_SIZE: f32 = 22.0;
const ACCENT_PICKER_RADIUS: f32 = appearance::THEME_CORNER_RADIUS + 2.0;
const ACCENT_PICKER_HANDLE_RADIUS: f32 = 8.0;

const ACCENT_PRESETS: [appearance::AccentColor; 8] = [
    appearance::AccentColor::new(0xE8, 0x8A, 0xAB),
    appearance::AccentColor::new(0xF0, 0x92, 0x6A),
    appearance::AccentColor::new(0xE1, 0xB2, 0x4E),
    appearance::AccentColor::new(0x8D, 0xC6, 0x5A),
    appearance::AccentColor::new(0x4C, 0xC3, 0x9B),
    appearance::AccentColor::new(0x52, 0xB8, 0xD6),
    appearance::AccentColor::new(0x6F, 0x95, 0xF5),
    appearance::AccentColor::new(0xC0, 0x8A, 0xD8),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Team {
    pub name: String,
    pub id: String,
}

impl std::fmt::Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ShowLogin,
    SelectAccount(usize),
    RemoveAccount(usize),
    ExportP12,
    SelectTeam(String, String),
    FetchTeams(String),
    TeamsLoaded(String, Vec<Team>),
    ToggleAutoStart(bool),
    OpenAccentEditor(appearance::AccentColor),
    CloseAccentEditor,
    AccentHexChanged(String),
    AccentSpectrumChanged { saturation: f32, value: f32 },
    AccentHueChanged(f32),
    AccentPresetSelected(appearance::AccentColor),
    SubmitAccentEditor,
    ResetAccentEditor,
    AccentColorPicked(appearance::AccentColor),
    ResetAccentColor,
}

#[derive(Debug)]
pub struct SettingsScreen {
    teams: HashMap<String, Vec<Team>>,
    loading_teams: Option<String>,
    accent_editor_open: bool,
    accent_draft: appearance::AccentColor,
    accent_hsv: AccentHsv,
    accent_hex_input: String,
    accent_hex_valid: bool,
}

impl SettingsScreen {
    pub fn new() -> Self {
        let accent_draft = appearance::AccentColor::default();

        Self {
            teams: HashMap::new(),
            loading_teams: None,
            accent_editor_open: false,
            accent_draft,
            accent_hsv: AccentHsv::from_accent(accent_draft),
            accent_hex_input: accent_draft.to_hex(),
            accent_hex_valid: true,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchTeams(ref email) => {
                self.loading_teams = Some(email.clone());
                Task::none()
            }
            Message::TeamsLoaded(email, teams) => {
                self.teams.insert(email, teams);
                self.loading_teams = None;
                Task::none()
            }
            Message::OpenAccentEditor(accent) => {
                self.sync_accent_draft(accent);
                self.accent_editor_open = true;
                Task::none()
            }
            Message::CloseAccentEditor => {
                self.accent_editor_open = false;
                Task::none()
            }
            Message::AccentHexChanged(value) => {
                self.accent_hex_input = normalize_hex_input(value);

                if let Ok(accent) = self.accent_hex_input.parse() {
                    self.sync_accent_draft(accent);
                } else {
                    self.accent_hex_valid = false;
                }

                Task::none()
            }
            Message::AccentSpectrumChanged { saturation, value } => {
                self.sync_accent_hsv(AccentHsv {
                    saturation,
                    value,
                    ..self.accent_hsv
                });
                Task::none()
            }
            Message::AccentHueChanged(hue) => {
                self.sync_accent_hsv(AccentHsv {
                    hue,
                    ..self.accent_hsv
                });
                Task::none()
            }
            Message::AccentPresetSelected(accent) => {
                self.sync_accent_draft(accent);
                Task::none()
            }
            Message::SubmitAccentEditor => {
                if !self.accent_hex_valid {
                    return Task::none();
                }

                self.accent_editor_open = false;
                Task::done(Message::AccentColorPicked(self.accent_draft))
            }
            Message::ResetAccentEditor => {
                self.sync_accent_draft(appearance::AccentColor::default());
                Task::done(Message::ResetAccentColor)
            }
            Message::ToggleAutoStart(_) => Task::none(),
            Message::SelectTeam(_, _) => Task::none(),
            _ => Task::none(),
        }
    }

    pub fn view<'a>(
        &'a self,
        account_store: &'a Option<AccountStore>,
        accent_color: appearance::AccentColor,
    ) -> Element<'a, Message> {
        let Some(store) = account_store else {
            return column![text("Loading accounts...")]
                .spacing(appearance::THEME_PADDING)
                .padding(appearance::THEME_PADDING)
                .into();
        };

        let mut accounts: Vec<_> = store.accounts().iter().collect();
        accounts.sort_by_key(|(email, _)| *email);

        let selected_index = store
            .selected_account()
            .and_then(|acc| accounts.iter().position(|(email, _)| *email == acc.email()));

        if self.accent_editor_open {
            return container(
                column![self.view_accent_picker(accent_color)].spacing(appearance::THEME_PADDING),
            )
            .height(Fill)
            .into();
        }

        let mut content = column![].spacing(appearance::THEME_PADDING);

        if !accounts.is_empty() {
            let account_list = accounts.iter().enumerate().fold(
                column![].spacing(appearance::THEME_PADDING),
                |content, (index, (email, account))| {
                    let marker = if Some(index) == selected_index {
                        "[✓] "
                    } else {
                        "[ ] "
                    };
                    let style = if Some(index) == selected_index {
                        appearance::p_button
                    } else {
                        appearance::s_button
                    };

                    let account_button = button(
                        text(format!("{}{}", marker, account.email()))
                            .size(appearance::THEME_FONT_SIZE)
                            .align_x(Alignment::Start),
                    )
                    .on_press(Message::SelectAccount(index))
                    .style(style)
                    .width(Fill);

                    let mut account_row = row![account_button].spacing(appearance::THEME_PADDING);

                    if Some(index) == selected_index {
                        let team_id = account.team_id();
                        let is_loading = self.loading_teams.as_ref() == Some(email);
                        let teams = self.teams.get(*email).cloned().unwrap_or_default();

                        let current_team = if !team_id.is_empty() {
                            teams.iter().find(|team| team.id == *team_id).cloned()
                        } else {
                            None
                        };

                        let placeholder = if is_loading {
                            "Loading teams...".to_string()
                        } else if !team_id.is_empty() {
                            team_id.to_string()
                        } else {
                            "Select team...".to_string()
                        };

                        let email_owned = email.to_string();
                        let team_pick = pick_list(teams, current_team, move |selected: Team| {
                            Message::SelectTeam(email_owned.clone(), selected.id)
                        })
                        .placeholder(placeholder)
                        .on_open(Message::FetchTeams(email.to_string()))
                        .style(appearance::s_pick_list);

                        account_row = account_row.push(team_pick);
                    }

                    content.push(account_row)
                },
            );

            content = content.push(
                container(scrollable(account_list))
                    .height(Length::Fixed(ACCOUNT_LIST_HEIGHT))
                    .style(|theme: &iced::Theme| container::Style {
                        border: iced::Border {
                            width: 1.0,
                            color: theme.palette().background.scale_alpha(0.5),
                            radius: appearance::THEME_CORNER_RADIUS.into(),
                        },
                        ..Default::default()
                    }),
            );
        } else {
            content = content.push(text("No accounts added yet"));
        }

        content = content.push(container(text("")).height(Fill));
        content = content.push(self.view_accent_picker(accent_color));
        content = content.push(self.view_auto_start_toggle(crate::startup::auto_start_enabled()));
        content = content.push(self.view_account_buttons(selected_index));

        container(content).height(Fill).into()
    }

    fn sync_accent_draft(&mut self, accent: appearance::AccentColor) {
        self.accent_draft = accent;
        self.accent_hsv = AccentHsv::from_accent(accent);
        self.accent_hex_input = accent.to_hex();
        self.accent_hex_valid = true;
    }

    fn sync_accent_hsv(&mut self, hsv: AccentHsv) {
        self.accent_hsv = hsv.clamped();
        self.accent_draft = self.accent_hsv.to_accent();
        self.accent_hex_input = self.accent_draft.to_hex();
        self.accent_hex_valid = true;
    }

    fn view_accent_picker(&self, accent_color: appearance::AccentColor) -> Element<'_, Message> {
        let preview = if self.accent_editor_open {
            self.accent_draft
        } else {
            accent_color
        };
        let preview_color = preview.primary();

        let mut summary = row![
            container(text(""))
                .width(52)
                .height(32)
                .style(move |_theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(preview_color)),
                    border: iced::Border {
                        width: 1.0,
                        color: Color::WHITE.scale_alpha(0.16),
                        radius: appearance::THEME_CORNER_RADIUS.into(),
                    },
                    ..Default::default()
                }),
            column![
                text("Accent").size(appearance::THEME_FONT_SIZE + 1.0),
                text(preview.to_hex())
                    .size(appearance::THEME_FONT_SIZE - 1.0)
                    .style(|theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(theme.palette().text.scale_alpha(0.72)),
                    })
            ]
            .spacing(2)
            .width(Fill),
        ]
        .spacing(appearance::THEME_PADDING)
        .align_y(Alignment::Center);

        if self.accent_editor_open {
            summary = summary
                .push(
                    button(text("Reset").align_x(Alignment::Center))
                        .on_press(Message::ResetAccentEditor)
                        .padding([6.0, 10.0])
                        .style(appearance::s_button),
                )
                .push(
                    button(text("Cancel").align_x(Alignment::Center))
                        .on_press(Message::CloseAccentEditor)
                        .padding([6.0, 10.0])
                        .style(appearance::s_button),
                )
                .push(
                    button(text("Apply").align_x(Alignment::Center))
                        .on_press_maybe(
                            self.accent_hex_valid.then_some(Message::SubmitAccentEditor),
                        )
                        .padding([6.0, 10.0])
                        .style(move |theme, status| {
                            draft_accent_button_style(theme, status, self.accent_draft.primary())
                        }),
                );
        } else {
            summary = summary
                .push(
                    button(text("Pick").align_x(Alignment::Center))
                        .on_press(Message::OpenAccentEditor(accent_color))
                        .padding([6.0, 10.0])
                        .style(appearance::s_button),
                )
                .push(
                    button(text("Reset").align_x(Alignment::Center))
                        .on_press(Message::ResetAccentEditor)
                        .padding([6.0, 10.0])
                        .style(appearance::s_button),
                );
        }

        let mut card = column![summary].spacing(appearance::THEME_PADDING);

        if self.accent_editor_open {
            card = card.push(
                container(
                    row![
                        container(self.view_visual_accent_picker()).width(Fill),
                        self.view_accent_side_panel(),
                    ]
                    .spacing(appearance::THEME_PADDING)
                    .align_y(Alignment::Start),
                )
                .padding([appearance::THEME_PADDING, ACCENT_CARD_HORIZONTAL_PADDING])
                .style(|theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(
                        theme.palette().background.scale_alpha(0.55),
                    )),
                    ..Default::default()
                }),
            );
        }

        container(card)
            .padding([appearance::THEME_PADDING, ACCENT_CARD_HORIZONTAL_PADDING])
            .style(|theme: &iced::Theme| container::Style {
                border: iced::Border {
                    width: 1.0,
                    color: theme.palette().background.scale_alpha(0.5),
                    radius: appearance::THEME_CORNER_RADIUS.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn view_visual_accent_picker(&self) -> Element<'_, Message> {
        row![
            canvas(AccentSpectrumCanvas {
                hsv: self.accent_hsv,
            })
            .width(Fill)
            .height(Length::Fixed(ACCENT_PICKER_HEIGHT)),
            canvas(AccentHueCanvas {
                hue: self.accent_hsv.hue,
            })
            .width(Length::Fixed(ACCENT_HUE_STRIP_WIDTH))
            .height(Length::Fixed(ACCENT_PICKER_HEIGHT)),
        ]
        .spacing(appearance::THEME_PADDING)
        .align_y(Alignment::Center)
        .width(Fill)
        .into()
    }

    fn view_accent_side_panel(&self) -> Element<'_, Message> {
        let mut content = column![
            self.view_accent_swatch(),
            text_input("#E88AAB", &self.accent_hex_input)
                .on_input(Message::AccentHexChanged)
                .on_submit(Message::SubmitAccentEditor)
                .padding(8),
            self.view_accent_presets(),
        ]
        .spacing(8);

        if !self.accent_hex_valid {
            content = content.push(
                text("#RRGGBB")
                    .size(appearance::THEME_FONT_SIZE - 2.0)
                    .style(|_theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(Color::from_rgb(0.9, 0.4, 0.45)),
                    }),
            );
        }

        container(content)
            .width(Length::Fixed(ACCENT_SIDE_PANEL_WIDTH))
            .into()
    }

    fn view_accent_swatch(&self) -> Element<'_, Message> {
        let swatch_color = self.accent_draft.primary();

        container(
            column![
                container(text(""))
                    .width(Fill)
                    .height(Length::Fixed(ACCENT_SWATCH_HEIGHT))
                    .style(move |_theme: &iced::Theme| container::Style {
                        background: Some(iced::Background::Color(swatch_color)),
                        border: iced::Border {
                            width: 1.0,
                            color: Color::WHITE.scale_alpha(0.14),
                            radius: appearance::THEME_CORNER_RADIUS.into(),
                        },
                        ..Default::default()
                    }),
                text(self.accent_draft.to_hex()).size(appearance::THEME_FONT_SIZE - 1.0),
            ]
            .spacing(6),
        )
        .padding(8)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.palette().background.scale_alpha(0.45),
            )),
            border: iced::Border {
                width: 1.0,
                color: theme.palette().background.scale_alpha(0.35),
                radius: appearance::THEME_CORNER_RADIUS.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn view_accent_presets(&self) -> Element<'_, Message> {
        ACCENT_PRESETS
            .chunks(ACCENT_PRESET_COLUMNS)
            .fold(column![].spacing(8), |rows, chunk| {
                rows.push(
                    chunk
                        .iter()
                        .copied()
                        .fold(row![].spacing(8), |swatches, preset| {
                            let swatch_color = preset.primary();
                            let selected = preset == self.accent_draft;

                            swatches.push(
                                button(text(""))
                                    .on_press(Message::AccentPresetSelected(preset))
                                    .width(ACCENT_PRESET_SIZE)
                                    .height(ACCENT_PRESET_SIZE)
                                    .style(move |_theme: &iced::Theme, status| {
                                        let border_color = if selected {
                                            Color::WHITE.scale_alpha(0.95)
                                        } else {
                                            match status {
                                                iced::widget::button::Status::Hovered => {
                                                    Color::WHITE.scale_alpha(0.55)
                                                }
                                                iced::widget::button::Status::Pressed => {
                                                    Color::WHITE.scale_alpha(0.4)
                                                }
                                                iced::widget::button::Status::Disabled => {
                                                    Color::WHITE.scale_alpha(0.2)
                                                }
                                                iced::widget::button::Status::Active => {
                                                    Color::WHITE.scale_alpha(0.28)
                                                }
                                            }
                                        };

                                        iced::widget::button::Style {
                                            background: Some(iced::Background::Color(swatch_color)),
                                            text_color: iced::Color::TRANSPARENT,
                                            border: iced::Border {
                                                width: if selected { 2.0 } else { 1.0 },
                                                color: border_color,
                                                radius: appearance::THEME_CORNER_RADIUS.into(),
                                            },
                                            shadow: iced::Shadow::default(),
                                            snap: false,
                                        }
                                    }),
                            )
                        }),
                )
            })
            .into()
    }

    fn view_auto_start_toggle(&self, auto_start_enabled: bool) -> Element<'_, Message> {
        checkbox(auto_start_enabled)
            .label("Launch on Startup")
            .on_toggle(Message::ToggleAutoStart)
            .into()
    }

    fn view_account_buttons(&self, selected_index: Option<usize>) -> Element<'_, Message> {
        let mut buttons = row![
            button(appearance::icon_text(appearance::PLUS, "Add Account", None))
                .on_press(Message::ShowLogin)
                .style(appearance::s_button)
                .width(ACCOUNT_ACTION_BUTTON_WIDTH)
        ]
        .spacing(appearance::THEME_PADDING);

        if let Some(index) = selected_index {
            buttons = buttons
                .push(
                    button(appearance::icon_text(
                        appearance::MINUS,
                        "Remove Account",
                        None,
                    ))
                    .on_press(Message::RemoveAccount(index))
                    .style(appearance::s_button)
                    .width(ACCOUNT_ACTION_BUTTON_WIDTH),
                )
                .push(
                    button(appearance::icon_text(appearance::SHARE, "Export P12", None))
                        .on_press(Message::ExportP12)
                        .style(appearance::s_button)
                        .width(ACCOUNT_ACTION_BUTTON_WIDTH),
                );
        }

        buttons.align_y(Alignment::Center).into()
    }
}

#[derive(Debug, Clone, Copy)]
struct AccentHsv {
    hue: f32,
    saturation: f32,
    value: f32,
}

impl AccentHsv {
    fn from_accent(accent: appearance::AccentColor) -> Self {
        let red = accent.red() as f32 / 255.0;
        let green = accent.green() as f32 / 255.0;
        let blue = accent.blue() as f32 / 255.0;

        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let chroma = max - min;

        let hue = if chroma <= f32::EPSILON {
            0.0
        } else if (max - red).abs() <= f32::EPSILON {
            60.0 * ((green - blue) / chroma).rem_euclid(6.0)
        } else if (max - green).abs() <= f32::EPSILON {
            60.0 * (((blue - red) / chroma) + 2.0)
        } else {
            60.0 * (((red - green) / chroma) + 4.0)
        };

        let saturation = if max <= f32::EPSILON {
            0.0
        } else {
            chroma / max
        };

        Self {
            hue,
            saturation,
            value: max,
        }
        .clamped()
    }

    fn to_accent(self) -> appearance::AccentColor {
        let hsv = self.clamped();
        let chroma = hsv.value * hsv.saturation;
        let hue = hsv.hue.rem_euclid(360.0) / 60.0;
        let secondary = chroma * (1.0 - ((hue.rem_euclid(2.0)) - 1.0).abs());
        let match_hue = if (0.0..1.0).contains(&hue) {
            (chroma, secondary, 0.0)
        } else if (1.0..2.0).contains(&hue) {
            (secondary, chroma, 0.0)
        } else if (2.0..3.0).contains(&hue) {
            (0.0, chroma, secondary)
        } else if (3.0..4.0).contains(&hue) {
            (0.0, secondary, chroma)
        } else if (4.0..5.0).contains(&hue) {
            (secondary, 0.0, chroma)
        } else {
            (chroma, 0.0, secondary)
        };

        let match_offset = hsv.value - chroma;
        appearance::AccentColor::new(
            to_channel(match_hue.0 + match_offset),
            to_channel(match_hue.1 + match_offset),
            to_channel(match_hue.2 + match_offset),
        )
    }

    fn hue_color(self) -> Color {
        Self {
            saturation: 1.0,
            value: 1.0,
            ..self
        }
        .to_accent()
        .primary()
    }

    fn clamped(self) -> Self {
        Self {
            hue: self.hue.clamp(0.0, 360.0),
            saturation: self.saturation.clamp(0.0, 1.0),
            value: self.value.clamp(0.0, 1.0),
        }
    }
}

#[derive(Debug, Default)]
struct AccentPickerDragState {
    dragging: bool,
}

#[derive(Debug, Clone, Copy)]
struct AccentSpectrumCanvas {
    hsv: AccentHsv,
}

impl canvas::Program<Message> for AccentSpectrumCanvas {
    type State = AccentPickerDragState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let publish = |position: Point| {
            let (saturation, value) = spectrum_from_position(bounds, position);

            Message::AccentSpectrumChanged { saturation, value }
        };

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | canvas::Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(position) = cursor.position_over(bounds) {
                    state.dragging = true;

                    return Some(canvas::Action::publish(publish(position)).and_capture());
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. })
            | canvas::Event::Touch(touch::Event::FingerMoved { .. }) => {
                if state.dragging {
                    if let Some(position) = cursor.land().position() {
                        return Some(canvas::Action::publish(publish(position)).and_capture());
                    }
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | canvas::Event::Touch(touch::Event::FingerLifted { .. })
            | canvas::Event::Touch(touch::Event::FingerLost { .. }) => {
                if state.dragging {
                    state.dragging = false;
                    return Some(canvas::Action::capture());
                }
            }
            _ => {}
        }

        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let size = Size::new(bounds.width, bounds.height);
        let mut frame = canvas::Frame::new(renderer, size);
        let picker = canvas::Path::rounded_rectangle(
            Point::new(0.0, 0.0),
            size,
            ACCENT_PICKER_RADIUS.into(),
        );

        frame.fill(&picker, self.hsv.hue_color());
        frame.fill(
            &picker,
            canvas::gradient::Linear::new(Point::new(0.0, 0.0), Point::new(size.width, 0.0))
                .add_stop(0.0, Color::WHITE)
                .add_stop(
                    1.0,
                    Color {
                        a: 0.0,
                        ..Color::WHITE
                    },
                ),
        );
        frame.fill(
            &picker,
            canvas::gradient::Linear::new(Point::new(0.0, 0.0), Point::new(0.0, size.height))
                .add_stop(
                    0.0,
                    Color {
                        a: 0.0,
                        ..Color::BLACK
                    },
                )
                .add_stop(1.0, Color::BLACK),
        );
        frame.stroke(
            &picker,
            canvas::Stroke::default()
                .with_width(1.0)
                .with_color(Color::WHITE.scale_alpha(0.18)),
        );

        let handle_position = spectrum_handle_position(size, self.hsv);
        let shadow = canvas::Path::circle(handle_position, ACCENT_PICKER_HANDLE_RADIUS + 1.5);
        let ring = canvas::Path::circle(handle_position, ACCENT_PICKER_HANDLE_RADIUS);

        frame.stroke(
            &shadow,
            canvas::Stroke::default()
                .with_width(3.0)
                .with_color(Color::BLACK.scale_alpha(0.35)),
        );
        frame.stroke(
            &ring,
            canvas::Stroke::default()
                .with_width(2.0)
                .with_color(Color::WHITE.scale_alpha(0.95)),
        );

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::None
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AccentHueCanvas {
    hue: f32,
}

impl canvas::Program<Message> for AccentHueCanvas {
    type State = AccentPickerDragState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let publish =
            |position: Point| Message::AccentHueChanged(hue_from_position(bounds, position));

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | canvas::Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(position) = cursor.position_over(bounds) {
                    state.dragging = true;

                    return Some(canvas::Action::publish(publish(position)).and_capture());
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. })
            | canvas::Event::Touch(touch::Event::FingerMoved { .. }) => {
                if state.dragging {
                    if let Some(position) = cursor.land().position() {
                        return Some(canvas::Action::publish(publish(position)).and_capture());
                    }
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | canvas::Event::Touch(touch::Event::FingerLifted { .. })
            | canvas::Event::Touch(touch::Event::FingerLost { .. }) => {
                if state.dragging {
                    state.dragging = false;
                    return Some(canvas::Action::capture());
                }
            }
            _ => {}
        }

        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let size = Size::new(bounds.width, bounds.height);
        let mut frame = canvas::Frame::new(renderer, size);
        let picker = canvas::Path::rounded_rectangle(
            Point::new(0.0, 0.0),
            size,
            ACCENT_PICKER_RADIUS.into(),
        );

        frame.fill(
            &picker,
            canvas::gradient::Linear::new(Point::new(0.0, 0.0), Point::new(0.0, size.height))
                .add_stop(0.0, Color::from_rgb8(0xFF, 0x4D, 0x4D))
                .add_stop(1.0 / 6.0, Color::from_rgb8(0xFF, 0xD1, 0x4D))
                .add_stop(2.0 / 6.0, Color::from_rgb8(0x67, 0xE8, 0x66))
                .add_stop(3.0 / 6.0, Color::from_rgb8(0x45, 0xD9, 0xE8))
                .add_stop(4.0 / 6.0, Color::from_rgb8(0x5B, 0x7C, 0xFF))
                .add_stop(5.0 / 6.0, Color::from_rgb8(0xD2, 0x6B, 0xF5))
                .add_stop(1.0, Color::from_rgb8(0xFF, 0x4D, 0x4D)),
        );
        frame.stroke(
            &picker,
            canvas::Stroke::default()
                .with_width(1.0)
                .with_color(Color::WHITE.scale_alpha(0.18)),
        );

        let indicator_y = hue_handle_y(size.height, self.hue);
        let indicator = canvas::Path::line(
            Point::new(4.0, indicator_y),
            Point::new((size.width - 4.0).max(4.0), indicator_y),
        );

        frame.stroke(
            &indicator,
            canvas::Stroke::default()
                .with_width(6.0)
                .with_color(Color::BLACK.scale_alpha(0.32))
                .with_line_cap(canvas::LineCap::Round),
        );
        frame.stroke(
            &indicator,
            canvas::Stroke::default()
                .with_width(3.0)
                .with_color(Color::WHITE.scale_alpha(0.95))
                .with_line_cap(canvas::LineCap::Round),
        );

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::None
        }
    }
}

fn spectrum_from_position(bounds: Rectangle, position: Point) -> (f32, f32) {
    let relative = clamp_to_bounds(bounds, position);
    let width = bounds.width.max(1.0);
    let height = bounds.height.max(1.0);

    (
        (relative.x / width).clamp(0.0, 1.0),
        (1.0 - (relative.y / height)).clamp(0.0, 1.0),
    )
}

fn hue_from_position(bounds: Rectangle, position: Point) -> f32 {
    let relative = clamp_to_bounds(bounds, position);
    let height = bounds.height.max(1.0);

    ((relative.y / height).clamp(0.0, 1.0) * 360.0).clamp(0.0, 360.0)
}

fn spectrum_handle_position(size: Size, hsv: AccentHsv) -> Point {
    Point::new(
        hsv.saturation.clamp(0.0, 1.0) * size.width,
        (1.0 - hsv.value.clamp(0.0, 1.0)) * size.height,
    )
}

fn hue_handle_y(height: f32, hue: f32) -> f32 {
    (hue.clamp(0.0, 360.0) / 360.0) * height.max(1.0)
}

fn clamp_to_bounds(bounds: Rectangle, position: Point) -> Point {
    Point::new(
        (position.x - bounds.x).clamp(0.0, bounds.width.max(0.0)),
        (position.y - bounds.y).clamp(0.0, bounds.height.max(0.0)),
    )
}

fn to_channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn normalize_hex_input(input: String) -> String {
    let mut normalized = input.trim().to_uppercase();

    if !normalized.is_empty()
        && !normalized.starts_with('#')
        && normalized.chars().all(|ch| ch.is_ascii_hexdigit())
        && normalized.len() <= 6
    {
        normalized.insert(0, '#');
    }

    normalized
}

fn draft_accent_button_style(
    theme: &iced::Theme,
    status: iced::widget::button::Status,
    accent: Color,
) -> iced::widget::button::Style {
    let palette = theme.palette();

    match status {
        iced::widget::button::Status::Active => iced::widget::button::Style {
            background: Some(iced::Background::Color(accent)),
            text_color: palette.background,
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: appearance::THEME_CORNER_RADIUS.into(),
            },
            shadow: iced::Shadow::default(),
            snap: false,
        },
        iced::widget::button::Status::Hovered => iced::widget::button::Style {
            background: Some(iced::Background::Color(appearance::lighten(accent, 0.15))),
            text_color: appearance::lighten(palette.background, 0.1),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: appearance::THEME_CORNER_RADIUS.into(),
            },
            shadow: iced::Shadow::default(),
            snap: false,
        },
        iced::widget::button::Status::Pressed => iced::widget::button::Style {
            background: Some(iced::Background::Color(appearance::lighten(accent, 0.03))),
            text_color: appearance::darken(palette.background, 0.1),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: appearance::THEME_CORNER_RADIUS.into(),
            },
            shadow: iced::Shadow::default(),
            snap: false,
        },
        iced::widget::button::Status::Disabled => iced::widget::button::Style {
            background: Some(iced::Background::Color(
                appearance::lighten(accent, 0.05).scale_alpha(0.2),
            )),
            text_color: palette.background.scale_alpha(0.5),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: appearance::THEME_CORNER_RADIUS.into(),
            },
            shadow: iced::Shadow::default(),
            snap: false,
        },
    }
}
