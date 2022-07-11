use crate::{Multitool, MultitoolMessage, SwitchViewMessage, ViewMessage};
use std::path::PathBuf;

use iced::{
    button, futures, scrollable, Align, Button, Clipboard, Column, Command, Container, Element,
    HorizontalAlignment, Length, Scrollable, Text,
};

// We allow this because we explicitly want to underline that these cases belong to user interaction.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum HomepageViewMessage {
    ClickInitializeOntology,
    ClickLintFile(Option<PathBuf>),
    ClickLintSelected(Option<PathBuf>),
    ClickConfig,
}

impl From<HomepageViewMessage> for ViewMessage {
    fn from(original: HomepageViewMessage) -> Self {
        ViewMessage::HomepageView(original)
    }
}

#[derive(Default)]
pub struct HomepageView {
    scroll: scrollable::State,
    initialize_button: button::State,
    lint_button: button::State,
    config_button: button::State,
}

impl HomepageView {
    pub fn view(&mut self) -> Element<MultitoolMessage> {
        Element::from(self.view_inner())
            .map(|view_message| MultitoolMessage::from(ViewMessage::from(view_message)))
    }

    pub fn update(
        &mut self,
        message: HomepageViewMessage,
        _clipboard: &mut Clipboard,
    ) -> Command<MultitoolMessage> {
        match message {
            HomepageViewMessage::ClickInitializeOntology => {
                Command::perform(futures::future::ready(()), |_| {
                    MultitoolMessage::SwitchView(SwitchViewMessage::InitializeOntologyView)
                })
            }
            HomepageViewMessage::ClickLintFile(directory) => {
                Command::perform(Multitool::select_lint_file(directory), |path| {
                    MultitoolMessage::ViewMessage(ViewMessage::HomepageView(
                        HomepageViewMessage::ClickLintSelected(path),
                    ))
                })
            }
            HomepageViewMessage::ClickLintSelected(maybe_selected_file) => {
                if let Some(selected_file) = maybe_selected_file {
                    return Command::perform(futures::future::ready(()), move |_| {
                        MultitoolMessage::SwitchView(SwitchViewMessage::LintSingleFileView {
                            ontology_file_path: selected_file.clone(),
                        })
                    });
                }
                Command::none()
            }
            HomepageViewMessage::ClickConfig => {
                Command::perform(futures::future::ready(()), |_| {
                    MultitoolMessage::SwitchView(SwitchViewMessage::Config)
                })
            }
        }
    }

    fn view_inner(&mut self) -> Scrollable<HomepageViewMessage> {
        let HomepageView {
            scroll,
            initialize_button,
            lint_button,
            config_button,
        } = self;

        let title = Text::new("Ontology Tools")
            .width(Length::Fill)
            .size(20)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Center);

        let initialize_button =
            Button::new(initialize_button, Text::new("Initialize new ontology file"))
                .on_press(HomepageViewMessage::ClickInitializeOntology)
                .padding(10);
        let lint_button = Button::new(lint_button, Text::new("Lint ontology file"))
            .on_press(HomepageViewMessage::ClickLintFile(None))
            .padding(10);
        let config_button = Button::new(config_button, Text::new("Change configuration"))
            .on_press(HomepageViewMessage::ClickConfig)
            .padding(10);
        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .align_items(Align::Center)
            .push(title)
            .push(initialize_button)
            .push(lint_button)
            .push(config_button);

        Scrollable::new(scroll)
            .padding(40)
            .push(Container::new(content).width(Length::Fill).center_x())
    }
}
