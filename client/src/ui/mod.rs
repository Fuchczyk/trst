use self::{
    main_menu::TrstMainMenu,
    preferences::{TrstPreferences, TrstPreferencesMessage},
};
use iced::Application;

mod main_menu;
mod preferences;

enum TrstFocus {
    MainMenu,
    Preferences,
    TestsSettings,
}

#[derive(Debug, Clone)]
enum TrstMessage {
    StartTesting,
    SwitchToPreferences,
    PreferencesMessage(TrstPreferencesMessage),
    SwitchToProgram,
    SwitchToTests,
    ShowLastRuns,
    QuitApplication,
}

impl From<TrstPreferencesMessage> for TrstMessage {
    fn from(preferences_message: TrstPreferencesMessage) -> Self {
        Self::PreferencesMessage(preferences_message)
    }
}

pub fn r() {
    let settings = iced::Settings::default();
    Trst::run(settings).unwrap();
}

struct Trst {
    state: TrstFocus,
    main_menu: TrstMainMenu,
    preferences: TrstPreferences,
}

impl Default for Trst {
    fn default() -> Self {
        Self {
            state: TrstFocus::MainMenu,
            main_menu: TrstMainMenu::default(),
            preferences: TrstPreferences::default(),
        }
    }
}

impl Application for Trst {
    type Executor = iced::executor::Default;

    type Message = TrstMessage;

    type Theme = iced::theme::Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self::default(), iced::Command::none())
    }

    fn title(&self) -> String {
        "Trst".into()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        println!("{:?}", message);

        match message {
            TrstMessage::PreferencesMessage(msg) => self.preferences.update(msg),
            _ => {}
        }

        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let preferences_col =
            iced::widget::column!(self.preferences.view()).width(iced::Length::FillPortion(9));

        let rows = iced::widget::row!(
            preferences_col,
            self.main_menu.view().width(iced::Length::FillPortion(2))
        )
        .padding(12);

        iced::Element::new(rows)
    }
}
