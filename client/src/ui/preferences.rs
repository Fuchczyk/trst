use iced::{
    widget::{Container, Row},
    Element, Sandbox,
};
use iced_native::{Renderer, Widget};

use super::TrstMessage;
use trst_types::RunningMode;

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum TestPlace {
    Local,
    Ssh,
}

impl TestPlace {
    fn desc(&self) -> &'static str {
        match self {
            Self::Local => "Local instance",
            Self::Ssh => "Ssh instance",
        }
    }
}

pub(super) struct TrstPreferences {
    concurrency: bool,
    test_place: Option<TestPlace>,
    git_address: String
}

#[derive(Clone, Debug)]
pub enum TrstPreferencesMessage {
    ConcurrencySelected(bool),
    TestPlaceSelected(TestPlace),
    GitAddressChange(String)
}

impl Default for TrstPreferences {
    fn default() -> Self {
        Self {
            concurrency: false,
            test_place: None,
            git_address: String::new()
        }
    }
}

impl TrstPreferences {
    pub fn update(&mut self, msg: TrstPreferencesMessage) {
        match msg {
            TrstPreferencesMessage::ConcurrencySelected(val) => self.concurrency = val,
            TrstPreferencesMessage::TestPlaceSelected(place) => self.test_place = Some(place),
            TrstPreferencesMessage::GitAddressChange(address) => self.git_address = address,
        }
    }

    fn generate_concurrency_box(&self) -> Container<TrstMessage> {
        let content = iced::widget::column![
            iced::widget::text("Concurrent testing")
                .size(40)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .width(iced::Length::Fill),
            iced::widget::column(vec![
                iced::Element::from(iced::widget::radio(
                    "Enabled",
                    true,
                    Some(self.concurrency),
                    |val| TrstPreferencesMessage::ConcurrencySelected(val).into()
                )),
                iced::Element::from(iced::widget::radio(
                    "Disabled",
                    false,
                    Some(self.concurrency),
                    |val| TrstPreferencesMessage::ConcurrencySelected(val).into()
                ))
            ])
            .spacing(3)
        ]
        .spacing(15);

        let appearance = iced::theme::Container::Custom(|_| {
            let mut app = iced::widget::container::Appearance::default();
            app.border_color = iced::Color::BLACK;
            app.background = None;
            app.border_width = 3.0;
            app.border_radius = 2.0;

            app
        });

        iced::widget::container(content.padding(15))
            .style(appearance)
            .width(iced::Length::Fill)
    }

    fn generate_test_place_box(&self) -> Container<TrstMessage> {
        let content = iced::widget::column![
            iced::widget::text("Testing place")
                .size(40)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .width(iced::Length::Fill),
            iced::widget::column(vec![
                iced::Element::from(iced::widget::radio(
                    TestPlace::Local.desc(),
                    TestPlace::Local,
                    self.test_place,
                    |val| TrstPreferencesMessage::TestPlaceSelected(val).into()
                )),
                iced::Element::from(iced::widget::radio(
                    TestPlace::Ssh.desc(),
                    TestPlace::Ssh,
                    self.test_place,
                    |val| TrstPreferencesMessage::TestPlaceSelected(val).into()
                ))
            ])
            .spacing(3)
        ]
        .spacing(15);

        let appearance = iced::theme::Container::Custom(|_| {
            let mut app = iced::widget::container::Appearance::default();
            app.border_color = iced::Color::BLACK;
            app.background = None;
            app.border_width = 3.0;
            app.border_radius = 2.0;

            app
        });

        iced::widget::container(content.padding(15))
            .style(appearance)
            .width(iced::Length::Fill)
    }

    fn generate_git_settings_box(&self) -> Container<TrstMessage> {
        let content = iced::widget::column![
            iced::widget::text("Testing place")
                .size(40)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .width(iced::Length::Fill),
            iced::widget::text("This git repository should be only used by this program.")
                .size(15)
                .horizontal_alignment(iced::alignment::Horizontal::Left)
                .width(iced::Length::Fill),
            iced::widget::column(vec![
                iced::Element::from(iced::widget::text("Git address")
                .size(15)
                .horizontal_alignment(iced::alignment::Horizontal::Left)
                .width(iced::Length::Fill)),
                iced::Element::from(iced::widget::TextInput::new("Git repository address", &self.git_address, |val|
                    TrstPreferencesMessage::GitAddressChange(val).into()))
            ])
            .spacing(3)
        ]
        .spacing(15);

        let appearance = iced::theme::Container::Custom(|_| {
            let mut app = iced::widget::container::Appearance::default();
            app.border_color = iced::Color::BLACK;
            app.background = None;
            app.border_width = 3.0;
            app.border_radius = 2.0;

            app
        });

        iced::widget::container(content.padding(15))
            .style(appearance)
            .width(iced::Length::Fill)
    }

    pub fn view(&self) -> Element<TrstMessage> {
        let concurrency_box = self.generate_concurrency_box();
        let test_place_box = self.generate_test_place_box();

        let mut column2;
        if self.test_place == Some(TestPlace::Ssh) {
            let git_address_box = self.generate_git_settings_box();
            column2 = iced::widget::column!(concurrency_box, test_place_box, git_address_box)
            .width(iced::Length::FillPortion(6))
            .spacing(20);
        } else {
            column2 = iced::widget::column!(concurrency_box, test_place_box)
            .width(iced::Length::FillPortion(6))
            .spacing(20);
        }

        let column1 = iced::widget::column!().width(iced::Length::FillPortion(2));

        let column3 = iced::widget::column!().width(iced::Length::FillPortion(2));

        let row = iced::widget::row![column1, column2, column3].padding(50);

        iced::Element::from(row)
    }
}
