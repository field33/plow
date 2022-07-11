use crate::{MultitoolMessage, SwitchViewMessage, ViewMessage};
use anyhow::anyhow;
use iced::{
    button, futures, scrollable, Align, Button, Clipboard, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Scrollable, Text,
};
use rfd::AsyncFileDialog;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::PathBuf;

pub struct ConfigView {
    config: Config,

    scroll: scrollable::State,
    home_button: button::State,
    pick_registry_path_button: button::State,
    save_button: button::State,
}

#[derive(Debug, Clone)]
pub enum ConfigViewMessage {
    ClickHome,
    ClickPickRegistryPath,
    ClickSave,
    RegistryPathSelected(Option<PathBuf>),
}

impl From<ConfigViewMessage> for ViewMessage {
    fn from(original: ConfigViewMessage) -> Self {
        ViewMessage::ConfigView(original)
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Config {
    pub registry_path: Option<PathBuf>,
}

impl Config {
    const MULTITOOL_CONFIG_DIR: &'static str = "ontology_tools";
    const MULTITOOL_CONFIG_FILENAME: &'static str = "config.json";

    fn config_file_path() -> Result<PathBuf, anyhow::Error> {
        Ok(dirs::config_dir()
            .ok_or_else(|| anyhow!("Configuration directory not detectable for platform"))?
            .join(Self::MULTITOOL_CONFIG_DIR)
            .join(Self::MULTITOOL_CONFIG_FILENAME))
    }

    /// Tries to read from disk if it exists, otherwise fallback to default
    pub fn get_or_default() -> Self {
        let path =
            Self::config_file_path().expect("Unable to construct path for configuration file");
        if !path.exists() {
            return Default::default();
        }

        let file_contents = read_to_string(path);
        if file_contents.is_err() {
            return Default::default();
        }
        let config = serde_json::from_str(&file_contents.unwrap()).ok();

        match config {
            Some(config) => config,
            None => Default::default(),
        }
    }

    /// Saves config to disk.
    pub fn save(&self) -> Result<(), anyhow::Error> {
        fs::create_dir_all(Self::config_file_path()?.parent().unwrap())?;
        let mut file = File::create(Self::config_file_path()?)?;
        file.write_all(serde_json::to_string_pretty(self)?.as_bytes())?;

        Ok(())
    }
}

impl ConfigView {
    pub fn new() -> Self {
        let existing_config = Config::get_or_default();

        Self {
            config: existing_config,
            scroll: Default::default(),
            home_button: Default::default(),
            pick_registry_path_button: Default::default(),
            save_button: Default::default(),
        }
    }

    pub fn update(
        &mut self,
        message: ConfigViewMessage,
        _clipboard: &mut Clipboard,
    ) -> Command<MultitoolMessage> {
        match message {
            ConfigViewMessage::ClickHome => Command::perform(futures::future::ready(()), |_| {
                MultitoolMessage::SwitchView(SwitchViewMessage::HomepageView)
            }),
            ConfigViewMessage::ClickPickRegistryPath => Command::perform(
                Self::pick_registry_path(self.config.registry_path.clone()),
                |path| {
                    MultitoolMessage::ViewMessage(ViewMessage::ConfigView(
                        ConfigViewMessage::RegistryPathSelected(path),
                    ))
                },
            ),
            ConfigViewMessage::ClickSave => {
                self.config.save().expect("Failed saving configuration");
                Command::none()
            }
            ConfigViewMessage::RegistryPathSelected(maybe_selected_dir) => {
                if let Some(selected_dir) = maybe_selected_dir {
                    self.config.registry_path = Some(selected_dir);
                }
                Command::none()
            }
        }
    }

    pub fn view(&mut self) -> Element<MultitoolMessage> {
        Element::from(self.view_inner())
            .map(|view_message| MultitoolMessage::from(ViewMessage::from(view_message)))
    }

    fn view_inner(&mut self) -> Scrollable<ConfigViewMessage> {
        let ConfigView {
            config,
            scroll,
            home_button,
            pick_registry_path_button,
            save_button,
        } = self;

        let home_button = Button::new(home_button, Text::new("Home"))
            .on_press(ConfigViewMessage::ClickHome)
            .padding(10);
        let title = Text::new("Change configuration")
            .width(Length::Fill)
            .size(20)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Center);

        let mut registry_path_row = Row::new().width(Length::Fill);
        registry_path_row = registry_path_row.push(Text::new("Registry path:"));
        registry_path_row = registry_path_row.push(Text::new(
            config
                .registry_path
                .clone()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
        ));
        registry_path_row = registry_path_row.push(
            Button::new(pick_registry_path_button, Text::new("Change directory"))
                .on_press(ConfigViewMessage::ClickPickRegistryPath)
                .padding(10),
        );

        let save_button = Button::new(save_button, Text::new("Save configuration"))
            .on_press(ConfigViewMessage::ClickSave)
            .padding(10);
        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .align_items(Align::Center)
            .push(home_button)
            .push(title)
            .push(registry_path_row)
            .push(save_button);

        Scrollable::new(scroll)
            .padding(40)
            .push(Container::new(content).width(Length::Fill).center_x())
    }

    async fn pick_registry_path(start_path: Option<PathBuf>) -> Option<PathBuf> {
        let mut dialog = AsyncFileDialog::new();

        if let Some(path) = start_path {
            dialog = dialog.set_directory(path);
        }

        dialog.pick_folder().await.map(|n| n.path().to_path_buf())
    }
}
