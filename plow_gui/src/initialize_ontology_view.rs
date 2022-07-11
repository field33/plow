use crate::{Multitool, MultitoolMessage, SwitchViewMessage, ViewMessage};
use iced::{
    button, futures, scrollable, text_input, Align, Button, Clipboard, Column, Command, Container,
    Element, HorizontalAlignment, Length, Scrollable, Text, TextInput,
};
use plow_ontology::initialize_ontology;
use std::path::PathBuf;

#[derive(Default)]
pub struct InitializeOntologyView {
    scroll: scrollable::State,
    create_button: button::State,
    ontology_name_input: text_input::State,
    ontology_name_value: String,
}

#[derive(Debug, Clone)]
pub enum InitializeOntologyViewMessage {
    OntologyNameTextInputChanged(String),
    ClickCreateOntologyFile,
    CreateOntologyFileSelected(String, Option<PathBuf>),
}

impl From<InitializeOntologyViewMessage> for ViewMessage {
    fn from(original: InitializeOntologyViewMessage) -> Self {
        ViewMessage::InitializeOntologyView(original)
    }
}

impl InitializeOntologyView {
    pub fn update(
        &mut self,
        message: InitializeOntologyViewMessage,
        _clipboard: &mut Clipboard,
    ) -> Command<MultitoolMessage> {
        match message {
            InitializeOntologyViewMessage::OntologyNameTextInputChanged(new_value) => {
                self.ontology_name_value = new_value;
                Command::none()
            }
            InitializeOntologyViewMessage::ClickCreateOntologyFile => {
                let ontology_name = self.ontology_name_value.clone();
                Command::perform(Multitool::save_file_path(), move |path| {
                    MultitoolMessage::from(ViewMessage::from(
                        InitializeOntologyViewMessage::CreateOntologyFileSelected(
                            ontology_name.clone(),
                            path,
                        ),
                    ))
                })
            }
            InitializeOntologyViewMessage::CreateOntologyFileSelected(ontology_name, path) => {
                if let Some(save_path) = path {
                    let contents = initialize_ontology(&ontology_name).unwrap();
                    std::fs::write(save_path, contents).expect("Unable to write file");
                    Command::perform(futures::future::ready(()), |_| {
                        MultitoolMessage::SwitchView(SwitchViewMessage::HomepageView)
                    })
                } else {
                    Command::none()
                }
            }
        }
    }

    pub fn view(&mut self) -> Element<MultitoolMessage> {
        Element::from(self.view_inner())
            .map(|view_message| MultitoolMessage::from(ViewMessage::from(view_message)))
    }

    fn view_inner(&mut self) -> Scrollable<InitializeOntologyViewMessage> {
        let InitializeOntologyView {
            scroll,
            create_button,
            ontology_name_input,
            ontology_name_value,
        } = self;

        let title = Text::new("Initialize new ontology file")
            .width(Length::Fill)
            .size(20)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Center);

        let input = TextInput::new(
            ontology_name_input,
            "Name of the ontology...",
            ontology_name_value,
            InitializeOntologyViewMessage::OntologyNameTextInputChanged,
        );
        let create_button = Button::new(create_button, Text::new("Create ontology file"))
            .on_press(InitializeOntologyViewMessage::ClickCreateOntologyFile)
            .padding(10);
        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .align_items(Align::Center)
            .push(title)
            .push(input)
            .push(create_button);

        Scrollable::new(scroll)
            .padding(40)
            .push(Container::new(content).width(Length::Fill).center_x())
    }
}
