use super::{Trst, TrstMessage};
use iced::{Application, Renderer, Sandbox};
use trst_types::Config;

pub struct TrstMainMenu {}

impl Default for TrstMainMenu {
    fn default() -> Self {
        Self {}
    }
}

type Column<'a> = iced::widget::Column<
    'a,
    <Trst as Application>::Message,
    iced::Renderer<<Trst as Application>::Theme>,
>;

impl TrstMainMenu {
    pub(super) fn view(&self) -> Column {
        let button_generator = |label: &str, theme: iced::theme::Button, message: TrstMessage| {
            let text = iced::widget::text(label)
                .vertical_alignment(iced::alignment::Vertical::Center)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .height(iced::Length::Fill);

            iced::widget::button(text)
                .on_press(message)
                .height(iced::Length::FillPortion(5))
                .style(theme)
                .padding(10)
        };

        // Viewing Main-Menu

        let start_button = button_generator(
            "Start",
            iced::theme::Button::Positive,
            TrstMessage::StartTesting,
        );

        let preferences_button = button_generator(
            "Preferences",
            iced::theme::Button::Secondary,
            TrstMessage::SwitchToPreferences,
        );

        let tests_button = button_generator(
            "Tests",
            iced::theme::Button::Secondary,
            TrstMessage::SwitchToTests,
        );

        let program_button = button_generator(
            "Program",
            iced::theme::Button::Secondary,
            TrstMessage::SwitchToProgram,
        );

        let last_runs_button = button_generator(
            "Last runs",
            iced::theme::Button::Secondary,
            TrstMessage::ShowLastRuns,
        );

        let quit_button = button_generator(
            "Quit",
            iced::theme::Button::Destructive,
            TrstMessage::QuitApplication,
        );

        iced::widget::column!(
            start_button,
            preferences_button,
            tests_button,
            program_button,
            last_runs_button,
            quit_button
        )
        .spacing(15)
    }
}
