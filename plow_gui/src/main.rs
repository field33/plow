// We're currently relaxed for the gui crate.
#![warn(clippy::all)]
#![allow(clippy::negative_feature_names)]

mod config_view;
mod file_watcher;
mod homepage_view;
mod initialize_ontology_view;
mod lint_single_file_view;
mod style;

use crate::file_watcher::FileWatcher;
use crate::homepage_view::{HomepageView, HomepageViewMessage};
use crate::lint_single_file_view::{LintSingleFileView, LintSingleFileViewMessage};

use crate::config_view::{ConfigView, ConfigViewMessage};
use crate::initialize_ontology_view::{InitializeOntologyView, InitializeOntologyViewMessage};
use iced::{futures, Application, Clipboard, Command, Element, Settings, Subscription};
use rfd::AsyncFileDialog;
use std::path::PathBuf;

pub fn main() -> iced::Result {
    Multitool::run(Settings::default())
}

#[derive(Default)]
struct Multitool {
    view: View,
}

enum View {
    Homepage(HomepageView),
    InitializeOntology(InitializeOntologyView),
    LintSingleFile(Box<LintSingleFileView>),
    Config(ConfigView),
}

impl Default for View {
    fn default() -> Self {
        Self::Homepage(HomepageView::default())
    }
}

#[derive(Debug, Clone)]
pub enum MultitoolMessage {
    Noop,
    SwitchView(SwitchViewMessage),
    ViewMessage(ViewMessage),
    // triggered by file watcher
    FileUpdated,
}

#[derive(Debug, Clone)]
pub enum SwitchViewMessage {
    HomepageView,
    InitializeOntologyView,
    LintSingleFileView { ontology_file_path: PathBuf },
    Config,
}

#[derive(Debug, Clone)]
pub enum ViewMessage {
    HomepageView(HomepageViewMessage),
    InitializeOntologyView(InitializeOntologyViewMessage),
    LintSingleFileView(LintSingleFileViewMessage),
    ConfigView(ConfigViewMessage),
}

impl From<ViewMessage> for MultitoolMessage {
    fn from(original: ViewMessage) -> Self {
        Self::ViewMessage(original)
    }
}

impl Application for Multitool {
    type Executor = iced::executor::Default;
    type Message = MultitoolMessage;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        "Field 33 Tools".to_owned()
    }

    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            MultitoolMessage::Noop => Command::none(),
            MultitoolMessage::SwitchView(message) => {
                match message {
                    SwitchViewMessage::HomepageView => {
                        self.view = View::Homepage(HomepageView::default());
                    }
                    SwitchViewMessage::InitializeOntologyView => {
                        self.view = View::InitializeOntology(InitializeOntologyView::default());
                    }
                    SwitchViewMessage::LintSingleFileView { ontology_file_path } => {
                        self.view = View::LintSingleFile(
                            LintSingleFileView::new(ontology_file_path).into(),
                        );
                    }
                    SwitchViewMessage::Config => {
                        self.view = View::Config(ConfigView::new());
                    }
                }
                Command::none()
            }

            MultitoolMessage::ViewMessage(view_message) => match view_message {
                ViewMessage::HomepageView(view_message) => {
                    if let View::Homepage(ref mut view) = self.view {
                        view.update(view_message, clipboard)
                    } else {
                        Command::none()
                    }
                }
                ViewMessage::InitializeOntologyView(view_message) => {
                    if let View::InitializeOntology(ref mut view) = self.view {
                        view.update(view_message, clipboard)
                    } else {
                        Command::none()
                    }
                }
                ViewMessage::LintSingleFileView(view_message) => {
                    if let View::LintSingleFile(ref mut view) = self.view {
                        view.update(view_message, clipboard)
                    } else {
                        Command::none()
                    }
                }
                ViewMessage::ConfigView(view_message) => {
                    if let View::Config(ref mut view) = self.view {
                        view.update(view_message, clipboard)
                    } else {
                        Command::none()
                    }
                }
            },
            MultitoolMessage::FileUpdated => {
                if let View::LintSingleFile(ref view) = &self.view {
                    return Command::perform(
                        futures::future::ready(view.selected_file.clone()),
                        move |selected_file| {
                            MultitoolMessage::SwitchView(SwitchViewMessage::LintSingleFileView {
                                ontology_file_path: selected_file,
                            })
                        },
                    );
                }
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if let View::LintSingleFile(view) = &self.view {
            let watcher = FileWatcher::new(view.selected_file.clone());
            Subscription::from_recipe(watcher).map(|n| match n {
                Ok(_) => MultitoolMessage::FileUpdated,
                Err(_) => MultitoolMessage::Noop,
            })
        } else {
            Subscription::none()
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let Self { view } = self;
        match view {
            View::Homepage(view) => view.view(),
            View::InitializeOntology(view) => view.view(),
            View::LintSingleFile(view) => view.view(),
            View::Config(view) => view.view(),
        }
    }
}

impl Multitool {
    async fn select_lint_file(path: Option<PathBuf>) -> Option<PathBuf> {
        let mut dialog = AsyncFileDialog::new().add_filter("Turtle ontology", &["ttl", "owl"]);

        if let Some(path) = path {
            dialog = dialog.set_directory(path);
        }

        dialog.pick_file().await.map(|n| n.path().to_path_buf())
    }

    async fn save_file_path() -> Option<PathBuf> {
        AsyncFileDialog::new()
            .add_filter("Turtle ontology", &["ttl"])
            .save_file()
            .await
            .map(|n| n.path().to_path_buf())
    }
}
