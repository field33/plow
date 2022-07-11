use crate::config_view::Config;
use crate::style::{COLOR_GREY, COLOR_RED, TEXT_SIZE_H1};
use crate::{HomepageViewMessage, Multitool, MultitoolMessage, SwitchViewMessage, ViewMessage};
use harriet::{ParseError, TurtleDocument};
use iced::{
    button, futures, pick_list, scrollable, Align, Button, Clipboard, Color, Column, Command,
    Container, Element, HorizontalAlignment, Length, PickList, Row, Scrollable, Text,
};
use nom::error::VerboseError;
use plow_package_management::edit::{AddDependency, EditOperation, RemoveDependency};
use plow_package_management::metadata::OntologyMetadata;
use plow_package_management::package::PackageVersionWithRegistryMetadata;
use plow_package_management::registry::on_disk::OnDiskRegistry;
use plow_package_management::registry::Registry;
use plow_package_management::resolve::Dependency;
use plow_package_management::version::SemanticVersion;
use plow_package_management::workspace::{
    OntologyWorkspace, OntologyWorkspaceLocked, OntologyWorkspaceWithCatalog,
    OntologyWorkspaceWithRetrievedDeps,
};
use std::path::{Path, PathBuf};
use plow_linter::lint::{FixSuggestion, Fixes, Lint, LintResult};
use plow_linter::lints::{required_package_management_lints, required_reference_registry_lints};

// TODO: move to lints?
pub struct CheckedLint {
    lint: Box<dyn Lint>,
    result: LintResult,
    suggested_fixes: Vec<Fixes>,
    suggested_fixes_button_states: Vec<button::State>,
}

#[allow(unused)]
pub struct LintSingleFileView {
    pub selected_file: PathBuf,
    is_fully_parsed: bool,
    current_file_contents: String,
    workspace: OntologyWorkspace,
    workspace_locked: Option<OntologyWorkspaceLocked>,
    workspace_locked_error: Option<anyhow::Error>,
    workspace_retrieved: Option<OntologyWorkspaceWithRetrievedDeps>,
    workspace_retrieved_error: Option<anyhow::Error>,
    workspace_with_catalog: Option<OntologyWorkspaceWithCatalog>,
    workspace_with_catalog_error: Option<anyhow::Error>,
    metadata: Option<OntologyMetadata>,

    scroll: scrollable::State,
    scroll_file_contents: scrollable::State,
    home_button: button::State,
    lint_different_file_button: button::State,
    checked_lints: Vec<CheckedLint>,

    new_dependency_pick_list: pick_list::State<PackageVersionWithRegistryMetadata>,
    new_dependency_options: Vec<PackageVersionWithRegistryMetadata>,
    new_dependency_selected: Option<PackageVersionWithRegistryMetadata>,
    new_dependency_add_button_state: button::State,

    open_in_protege_button: button::State,
    submit_to_registry_button: button::State,
    delete_dependency_button_states: Vec<button::State>,
}

#[derive(Debug, Clone)]
pub enum LintSingleFileViewMessage {
    ClickHome,
    ClickOpenInProtege {
        symlinked_path: PathBuf,
    },
    ClickLintFile(Option<PathBuf>),
    ClickSubmitToRegistry,
    ApplyLintFixSuggestion(Fixes, PathBuf),
    AddDependencySelected(PackageVersionWithRegistryMetadata),
    AddDependency {
        new_dependency: PackageVersionWithRegistryMetadata,
    },
    DeleteDependency {
        file_path: PathBuf,
        dependency_name: String,
    },
}

impl From<LintSingleFileViewMessage> for ViewMessage {
    fn from(original: LintSingleFileViewMessage) -> Self {
        ViewMessage::LintSingleFileView(original)
    }
}

impl LintSingleFileView {
    pub fn new(selected_file: PathBuf) -> Self {
        let config = Config::get_or_default();
        let file_contents = std::fs::read_to_string(&selected_file).expect("Failed reading file");
        let document = TurtleDocument::parse::<VerboseError<&str>>(&file_contents)
            .unwrap()
            .1;
        let is_fully_parsed = !matches!(
            TurtleDocument::parse_full(&file_contents),
            Err(ParseError::NotFullyParsed(_))
        );

        let mut lints = required_package_management_lints();
        lints.extend(required_reference_registry_lints().into_iter());
        let mut checked_lints = Vec::new();
        for lint in lints {
            let suggested_fixes = lint.suggest_fix(&document).unwrap_or_default();
            let suggested_fixes_button_states =
                suggested_fixes.iter().map(|_| Default::default()).collect();

            checked_lints.push(CheckedLint {
                result: lint.lint(&document),
                suggested_fixes,
                suggested_fixes_button_states,
                lint,
            });
        }

        let package_management_lints_passed = required_reference_registry_lints()
            .into_iter()
            .all(|lint| lint.lint(&document).is_success());
        let mut metadata: Option<OntologyMetadata> = None;
        let mut delete_dependency_button_states = vec![];
        if package_management_lints_passed {
            metadata = OntologyMetadata::try_from(&document).ok();

            delete_dependency_button_states = metadata
                .as_ref()
                .unwrap()
                .dependencies
                .iter()
                .map(|_| Default::default())
                .collect()
        }

        let registry_opt = Self::registry_from_config(&config)
            .expect("Unable to construct registry from config.")
            .expect("Registry path config has to be set");

        let WorkspaceSetup {
            workspace,
            workspace_locked,
            workspace_locked_error,
            workspace_retrieved,
            workspace_retrieved_error,
            workspace_with_catalog,
            workspace_with_catalog_error,
        } = WorkspaceSetup::setup_workspace_from_file(&selected_file, Some(registry_opt.clone()));

        Self {
            scroll: Default::default(),
            scroll_file_contents: Default::default(),
            home_button: Default::default(),
            lint_different_file_button: Default::default(),
            is_fully_parsed,
            selected_file,
            current_file_contents: file_contents,
            checked_lints,

            submit_to_registry_button: Default::default(),
            open_in_protege_button: Default::default(),
            workspace,
            workspace_locked,
            workspace_locked_error,
            workspace_retrieved,
            workspace_retrieved_error,
            workspace_with_catalog,
            workspace_with_catalog_error,
            metadata,
            delete_dependency_button_states,

            new_dependency_pick_list: Default::default(),
            new_dependency_options: registry_opt.list_all_package_versions().unwrap(),
            new_dependency_selected: None,
            new_dependency_add_button_state: Default::default(),
        }
    }

    fn registry_from_config(config: &Config) -> Result<Option<OnDiskRegistry>, anyhow::Error> {
        config
            .registry_path
            .clone()
            .map(OnDiskRegistry::new)
            .transpose()
    }

    pub fn update(
        &mut self,
        message: LintSingleFileViewMessage,
        _clipboard: &mut Clipboard,
    ) -> Command<MultitoolMessage> {
        match message {
            LintSingleFileViewMessage::ClickHome => {
                Command::perform(futures::future::ready(()), |_| {
                    MultitoolMessage::SwitchView(SwitchViewMessage::HomepageView)
                })
            }
            LintSingleFileViewMessage::ClickOpenInProtege { symlinked_path } => {
                open::that(&symlinked_path).expect("Opening in protege failed");
                Command::none()
            }
            LintSingleFileViewMessage::ClickLintFile(directory) => {
                Command::perform(Multitool::select_lint_file(directory), |path| {
                    MultitoolMessage::ViewMessage(ViewMessage::HomepageView(
                        HomepageViewMessage::ClickLintSelected(path),
                    ))
                })
            }
            LintSingleFileViewMessage::ClickSubmitToRegistry => {
                let contents = self.current_file_contents.clone();

                let config = Config::get_or_default();
                let registry = Self::registry_from_config(&config)
                    .expect("Unable to construct registry from config.")
                    .expect("Registry path config has to be set");

                registry
                    .submit_package(&contents)
                    .expect("Unable to submit package to registry");

                Command::none()
            }
            LintSingleFileViewMessage::ApplyLintFixSuggestion(fix, file_path) => {
                let mut backup_path = file_path.clone();
                backup_path.set_extension(format!(
                    "{}.bkp",
                    file_path.extension().unwrap().to_string_lossy()
                ));
                std::fs::copy(&file_path, backup_path).expect("Unable to copy file for backup");
                let file_contents =
                    std::fs::read_to_string(&file_path).expect("Unable to read file");
                let mut document = TurtleDocument::parse::<VerboseError<&str>>(&file_contents)
                    .unwrap()
                    .1;

                fix.apply(&mut document);

                std::fs::write(file_path, document.to_string()).expect("Unable to write file");

                Command::perform(futures::future::ready(()), |_| {
                    MultitoolMessage::FileUpdated
                })
            }
            LintSingleFileViewMessage::AddDependencySelected(new_selection) => {
                self.new_dependency_selected = Some(new_selection);
                Command::none()
            }
            LintSingleFileViewMessage::AddDependency { new_dependency } => {
                let file_path = &self.selected_file;
                let file_contents =
                    std::fs::read_to_string(&file_path).expect("Unable to read file");
                let mut document = TurtleDocument::parse_full(&file_contents).unwrap();
                let metadata = OntologyMetadata::try_from(&document).unwrap();

                // We can only have one version of a dependency.
                // If they are the same name we remove the existing one before adding the new one.
                metadata.dependencies.iter().for_each(|dep| {
                    if dep.full_name == new_dependency.package_name {
                        let remove_operation = RemoveDependency {
                            ontology_iri: metadata.root_prefix.clone(),
                            dependency_name: dep.full_name.clone(),
                        };
                        remove_operation.apply(&mut document).unwrap();
                    }
                });

                let add_operation = AddDependency {
                    ontology_iri: metadata.root_prefix,
                    dependency: Dependency::<SemanticVersion>::try_new(
                        &new_dependency.package_name,
                        &format!("={version}", version = new_dependency.version),
                    )
                    .unwrap_or_else(|_| {
                        // TODO: Is panicing ok here?
                        panic!(
                            "Failed to add a new dependency: {} {}",
                            &new_dependency.package_name,
                            &format!("={version}", version = new_dependency.version)
                        )
                    }),
                    dependency_ontology_iri: new_dependency.ontology_iri.unwrap(),
                };

                add_operation.apply(&mut document).unwrap();

                std::fs::write(file_path, document.to_string()).expect("Unable to write file");

                Command::perform(futures::future::ready(()), |_| {
                    MultitoolMessage::FileUpdated
                })
            }
            LintSingleFileViewMessage::DeleteDependency {
                file_path,
                dependency_name,
            } => {
                let file_contents =
                    std::fs::read_to_string(&file_path).expect("Unable to read file");
                let mut document = TurtleDocument::parse_full(&file_contents).unwrap();
                let metadata = OntologyMetadata::try_from(&document).unwrap();

                let delete_operation = RemoveDependency {
                    ontology_iri: metadata.root_prefix,
                    dependency_name,
                };

                delete_operation.apply(&mut document).unwrap();

                std::fs::write(file_path, document.to_string()).expect("Unable to write file");

                Command::perform(futures::future::ready(()), |_| {
                    MultitoolMessage::FileUpdated
                })
            }
        }
    }

    pub fn view(&mut self) -> Element<MultitoolMessage> {
        Element::from(self.view_inner())
            .map(|view_message| MultitoolMessage::from(ViewMessage::from(view_message)))
    }

    pub fn view_inner(&mut self) -> Scrollable<LintSingleFileViewMessage> {
        let LintSingleFileView {
            scroll,
            scroll_file_contents,
            home_button,
            lint_different_file_button,
            is_fully_parsed,
            selected_file,
            current_file_contents,
            checked_lints,
            open_in_protege_button,
            workspace,
            workspace_locked,
            workspace_locked_error,
            workspace_retrieved_error,
            workspace_with_catalog_error,
            metadata,
            delete_dependency_button_states,
            new_dependency_pick_list,
            new_dependency_options,
            new_dependency_selected,
            new_dependency_add_button_state,
            submit_to_registry_button,
            ..
        } = self;

        let home_button = Button::new(home_button, Text::new("Home"))
            .on_press(LintSingleFileViewMessage::ClickHome)
            .padding(10);
        let title = Text::new("Linter")
            .width(Length::Fill)
            .size(20)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(HorizontalAlignment::Center);

        let file_path = Text::new(format!("Linting file {}", selected_file.to_string_lossy()))
            .width(Length::Fill)
            .size(14);
        let lint_different_file_button = Button::new(
            lint_different_file_button,
            Text::new("Lint different ontology file"),
        )
        .on_press(LintSingleFileViewMessage::ClickLintFile(Some(
            selected_file.parent().unwrap().to_path_buf(),
        )))
        .padding(10);

        let open_in_protege_button =
            Button::new(open_in_protege_button, Text::new("Open in Protege"))
                .on_press(LintSingleFileViewMessage::ClickOpenInProtege {
                    symlinked_path: workspace.ontology_file().clone(),
                })
                .padding(10);

        let submit_to_registry_button = Button::new(
            submit_to_registry_button,
            Text::new("Submit package to registry"),
        )
        .on_press(LintSingleFileViewMessage::ClickSubmitToRegistry)
        .padding(10);

        let file_contents_header = Text::new("File contents")
            .width(Length::Fill)
            .size(16)
            .color([0.5, 0.5, 0.5]);
        let file_contents = Text::new(&*current_file_contents)
            .width(Length::Fill)
            .size(14);
        let scrollable_file_contents = Scrollable::new(scroll_file_contents)
            .padding(20)
            .max_height(100)
            .push(file_contents);

        let metadata_col = Self::dependency_section(
            selected_file.clone(),
            metadata.clone(),
            workspace_locked,
            workspace_locked_error,
            &self.workspace_retrieved,
            workspace_retrieved_error,
            workspace_with_catalog_error,
            new_dependency_pick_list,
            new_dependency_options,
            new_dependency_selected,
            new_dependency_add_button_state,
            delete_dependency_button_states,
        );

        let results_header = Text::new("Lint results")
            .width(Length::Fill)
            .size(16)
            .color([0.5, 0.5, 0.5]);
        let mut lints = Column::new().max_width(800).width(Length::Fill);

        if !*is_fully_parsed {
            lints = lints.push(
                Text::new("❌ Unable to fully parse the file. Since this is most likely due to missing parts of the parser, please tell the dev team (most likely Max) ❌️")
                    .color(Color::from_rgb(1f32, 0f32, 0f32))
                    .size(20),
            );
        }
        for checked_lint in checked_lints.iter_mut() {
            match &checked_lint.result {
                LintResult::Success(_) => {
                    lints = lints.push(
                        Text::new(format!("- {} ✅", checked_lint.lint.short_description()))
                            .color(Color::from_rgb(0f32, 1f32, 0f32))
                            .size(16),
                    );
                }
                LintResult::Warning(messages) => {
                    lints = lints.push(
                        Text::new(format!("- {} ⚠️", checked_lint.lint.short_description()))
                            .color(Color::from_rgb(1f32, 1f32, 0f32))
                            .size(16),
                    );
                    for message in messages {
                        lints = lints.push(
                            Text::new(format!("    - {}", message))
                                .color(Color::from_rgb(1f32, 1f32, 0f32))
                                .size(14),
                        );
                    }
                }
                LintResult::Failure(messages) => {
                    lints = lints.push(
                        Text::new(format!("- {} ❌️", checked_lint.lint.short_description()))
                            .color(COLOR_RED)
                            .size(16),
                    );
                    for message in messages {
                        lints = lints.push(
                            Text::new(format!("    - {}", message))
                                .color(COLOR_RED)
                                .size(14),
                        );
                    }

                    for (fix, button_state) in checked_lint
                        .suggested_fixes
                        .iter()
                        .zip(checked_lint.suggested_fixes_button_states.iter_mut())
                    {
                        // Applying fixes on an incompletely parsed file will corrupt them
                        if !*is_fully_parsed {
                            continue;
                        }
                        lints = lints.push(
                            Button::new(button_state, Text::new("Apply automatic fix"))
                                .on_press(LintSingleFileViewMessage::ApplyLintFixSuggestion(
                                    fix.clone(),
                                    selected_file.clone(),
                                ))
                                .padding(10),
                        );
                    }
                }
            }
        }

        let content = Column::new()
            .max_width(800)
            .spacing(20)
            .align_items(Align::Center)
            .push(home_button)
            .push(title)
            .push(file_path)
            .push(lint_different_file_button)
            .push(open_in_protege_button)
            .push(submit_to_registry_button)
            .push(file_contents_header)
            .push(scrollable_file_contents)
            .push(metadata_col)
            .push(results_header)
            .push(lints);

        Scrollable::new(scroll)
            .padding(40)
            .push(Container::new(content).width(Length::Fill).center_x())
    }

    #[allow(clippy::too_many_arguments)]
    fn dependency_section<'a>(
        file_path: PathBuf,
        metadata: Option<OntologyMetadata>,
        _workspace_locked: &'a Option<OntologyWorkspaceLocked>,
        workspace_locked_error: &'a Option<anyhow::Error>,
        workspace_retrieved: &'a Option<OntologyWorkspaceWithRetrievedDeps>,
        workspace_retrieved_error: &'a Option<anyhow::Error>,
        workspace_with_catalog_error: &'a Option<anyhow::Error>,
        new_dependency_pick_list: &'a mut pick_list::State<PackageVersionWithRegistryMetadata>,
        new_dependency_options: &'a [PackageVersionWithRegistryMetadata],
        new_dependency_selected: &'a Option<PackageVersionWithRegistryMetadata>,
        new_dependency_add_button_state: &'a mut button::State,
        delete_dependency_button_states: &'a mut [button::State],
    ) -> Column<'a, LintSingleFileViewMessage> {
        let mut metadata_col = Column::new().max_width(800).width(Length::Fill);
        let header = Text::new("Specified dependencies")
            .width(Length::Fill)
            .size(TEXT_SIZE_H1)
            .color(COLOR_GREY);
        metadata_col = metadata_col.push(header);
        if let Some(metadata) = metadata {
            for (dependency, state) in metadata
                .dependencies
                .iter()
                .zip(delete_dependency_button_states.iter_mut())
            {
                let mut dependency_row = Row::new().width(Length::Fill);
                dependency_row = dependency_row.push(Text::new(&dependency.full_name).size(16));
                dependency_row = dependency_row.push(Text::new(" : ").size(16).color(COLOR_GREY));
                dependency_row = dependency_row
                    .push(Text::new(dependency.version_requirement.to_string()).size(16));

                let delete_button = Button::new(state, Text::new("Remove dependency"))
                    .on_press(LintSingleFileViewMessage::DeleteDependency {
                        file_path: file_path.clone(),
                        dependency_name: dependency.full_name.clone(),
                    })
                    .padding(5);
                dependency_row = dependency_row.push(delete_button);

                metadata_col = metadata_col.push(dependency_row);
            }
        }
        let new_dependency_pick_list = PickList::new(
            new_dependency_pick_list,
            new_dependency_options,
            new_dependency_selected.to_owned(),
            LintSingleFileViewMessage::AddDependencySelected,
        );
        metadata_col = metadata_col.push(new_dependency_pick_list);
        let mut new_dependency_add_button =
            Button::new(new_dependency_add_button_state, Text::new("Add dependency")).padding(5);
        if let Some(new_dependency_selected) = new_dependency_selected {
            new_dependency_add_button =
                new_dependency_add_button.on_press(LintSingleFileViewMessage::AddDependency {
                    new_dependency: new_dependency_selected.to_owned(),
                });
        }

        metadata_col = metadata_col.push(new_dependency_add_button);

        let header_resolved = Text::new("Resolved dependencies:\n\n")
            .width(Length::Fill)
            .size(TEXT_SIZE_H1)
            .height(Length::Units(20))
            .color(COLOR_GREY);

        metadata_col = metadata_col.push(header_resolved);
        if let Some(workspace_retrieved) = workspace_retrieved {
            for dep in workspace_retrieved.retrieved_dependencies() {
                metadata_col = metadata_col.push(
                    Text::new(format!(
                        "{} {}",
                        dep.package.package_name, dep.package.version
                    ))
                    .width(Length::Fill)
                    .size(16)
                    .color(COLOR_GREY),
                );
            }
        }

        if let Some(err) = workspace_locked_error {
            metadata_col = metadata_col.push(
                Text::new(format!("Resolving/locking error: {}", err))
                    .width(Length::Fill)
                    .size(16)
                    .color(COLOR_RED),
            );
        }
        if let Some(err) = workspace_retrieved_error {
            metadata_col = metadata_col.push(
                Text::new(format!("Retrival error: {}", err))
                    .width(Length::Fill)
                    .size(16)
                    .color(COLOR_RED),
            );
        }
        if let Some(err) = workspace_with_catalog_error {
            metadata_col = metadata_col.push(
                Text::new(format!("Protege workspace creating error: {}", err))
                    .width(Length::Fill)
                    .size(16)
                    .color(COLOR_RED),
            );
        }

        metadata_col
    }
}

struct WorkspaceSetup {
    workspace: OntologyWorkspace,
    workspace_locked: Option<OntologyWorkspaceLocked>,
    workspace_locked_error: Option<anyhow::Error>,
    workspace_retrieved: Option<OntologyWorkspaceWithRetrievedDeps>,
    workspace_retrieved_error: Option<anyhow::Error>,
    workspace_with_catalog: Option<OntologyWorkspaceWithCatalog>,
    workspace_with_catalog_error: Option<anyhow::Error>,
}

impl WorkspaceSetup {
    fn setup_workspace_from_file(
        selected_file: &Path,
        registry_opt: Option<OnDiskRegistry>,
    ) -> Self {
        let workspace = OntologyWorkspace::mirror_file_to_workspace(selected_file).unwrap();

        let mut workspace_locked: Option<OntologyWorkspaceLocked> = None;
        let mut workspace_locked_error: Option<anyhow::Error> = None;
        let mut workspace_retrieved: Option<OntologyWorkspaceWithRetrievedDeps> = None;
        let mut workspace_retrieved_error: Option<anyhow::Error> = None;
        let mut workspace_with_catalog: Option<OntologyWorkspaceWithCatalog> = None;
        let mut workspace_with_catalog_error: Option<anyhow::Error> = None;

        if let Some(registry) = registry_opt {
            let registry = Box::new(registry);
            match workspace
                .clone()
                .lock(registry.as_ref(), Some(selected_file.to_path_buf()))
            {
                Ok(locked) => {
                    workspace_locked = Some(locked);
                }
                Err(err) => {
                    eprintln!("{:?}", err);
                    workspace_locked_error = Some(err)
                }
            }

            if let Some(workspace_locked) = &workspace_locked {
                match workspace_locked
                    .clone()
                    .retrieve_dependencies(registry.as_ref())
                {
                    Ok(retrieved) => {
                        workspace_retrieved = Some(retrieved);
                    }
                    Err(err) => {
                        eprintln!("{:?}", err);
                        workspace_retrieved_error = Some(err)
                    }
                }
            }

            if let Some(workspace_retrieved) = &workspace_retrieved {
                match workspace_retrieved.clone().generate_catalog_file() {
                    Ok(catalog) => {
                        workspace_with_catalog = Some(catalog);
                    }
                    Err(err) => {
                        eprintln!("{:?}", err);
                        workspace_with_catalog_error = Some(err)
                    }
                }
            }
        }

        Self {
            workspace,
            workspace_locked,
            workspace_locked_error,
            workspace_retrieved,
            workspace_retrieved_error,
            workspace_with_catalog,
            workspace_with_catalog_error,
        }
    }
}
