use std::{num::ParseIntError, time::Instant};
use dragking::{DragEvent, DropPosition};
use iced::{
    font,
    widget::{
        button, checkbox, column, container, horizontal_rule, horizontal_space, pick_list,
        progress_bar, row, scrollable, text, text_input, Column, Container, Row, Space,
    },
    Alignment, Color, Element, Font, Length, Padding, Task, Theme,
};
mod utils;
use utils::structs::LogType as LT;
use utils::{dobot::*, experiment::*, helpers::*, structs::*, styling::*};

fn main() -> iced::Result {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Off)
        .with_module_level("iced_dobot_controller", log::LevelFilter::Info)
        .with_colors(true)
        .with_timestamp_format(time::macros::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .init()
        .unwrap();

    log::info!("Running!");

    configure_startup();

    iced::application(
        IcedDobotController::title(),
        IcedDobotController::update,
        IcedDobotController::view,
    )
    .antialiasing(true)
    .theme(IcedDobotController::theme)
    .default_font(Font {
        family: font::Family::Name("PlemolJP"),
        ..Default::default()
    })
    .run_with(IcedDobotController::new)
}

#[derive(Clone)]
struct IcedDobotController {
    is_title_font_ok: bool,
    is_busy         : bool,

    // Tab Related Variable
    active_main_tab    : Tabpage,
    active_sequence_tab: SequencerTabpage,

    // Sequencer Tab Variable
    active_sequencer_sequence : Position,
    active_sequencer_sequences: Vec<Position>,
    sequences_paths           : Vec<String>,
    active_seq_path_input     : String,
    active_sequences          : Vec<Position>,

    // Experiment Tab Variable
    active_experiment_parameters   : Parameters,
    active_experiment_instance     : Option<ExperimentInstance>,
    active_experiment_state        : ExperimentStage,
    active_experiment_shapes_to_use: Vec<(Shapes, bool)>,
    n_v                            : f32,
    n_a                            : f32,
    error_info                     : Option<String>,
    time_main                      : Instant,
    is_time_counting               : bool,
    time_start                     : u128,
    time_end                       : u128,
    active_max_idx                 : u32,
    active_idx                     : u32,

    // Results Tab Variable
    results_paths     : Vec<String>,
    active_result_item: Option<ResultExports>,

    // Administration
    logs            : Vec<LogMessage>,
    active_config   : Config,
    is_debug_view   : bool,
    is_simulate_mode: bool,
    active_theme    : Theme,
}

#[derive(Clone, Debug)]
enum Message {
    TabSelected(Tabpage),
    SeqTabSelected(SequencerTabpage),

    // Sequencer Messages
    SequenceReorder(DragEvent),
    SequencerInputUpdated(Coordinate, String),
    SequencerMoveToPressed,
    SequencerAddToSequencesPressed,
    SequencerPerform,
    SequencerPerformResult(Option<String>),
    RemoveASequencePressed(usize),
    PasteSequencePressed(usize),
    ClearSequences,
    ClearSeqInput,
    SeqFilenameInputUpdated(String),
    SeqSaveFilePressed,
    SeqSaveFileResult(Option<String>),
    SeqLoadFilePressed(usize, bool),
    SeqLoadFileResult(Option<String>, Option<String>, bool),
    PerformSequenceResult(Option<String>),
    DobotGoHome,
    DobotTestConnection,
    DobotResult(Option<String>),

    // Experiment Messages
    ParameterInputChanged(ParameterType, String),
    SelectedShapesChanged(Shapes),
    BeginExperimentPressed,
    ShapeSelected(Shapes),
    RetryButtonPressed,
    ForceAbortPressed,
    GoToNextStage,
    DrawingResult(Option<String>),
    ResultsProcessed(Option<ResultExports>, Option<String>),

    // Results
    UpdateResultsList,
    ResultButtonPressed(String),
    ResultButtonResult(Option<String>, Option<String>),

    // Administration
    FontLoaded(Result<(), font::Error>),
    ConfigFileLoaded(Option<String>, Option<String>),
    SequencesListsLoaded(Option<Vec<String>>, Option<String>),
    ResultsListsUpdated(Option<Vec<String>>, Option<String>),
    DebugCheckboxPressed(bool),
    SimulateModeCheckboxPressed(bool),
    ThemeSelected(Theme),
}

impl IcedDobotController {
    fn title() -> &'static str {
        "Dobot Experiment App Rewrite"
    }

    fn new() -> (Self, Task<Message>) {
        (
            Self {
                is_title_font_ok   : false,
                is_busy            : false,
                active_main_tab    : Tabpage::Sequencer,
                active_sequence_tab: SequencerTabpage::Sequencer,

                active_sequencer_sequence : Position::default(),
                active_sequencer_sequences: Vec::new(),
                sequences_paths           : Vec::new(),
                active_seq_path_input     : "".to_string(),
                active_sequences          : Vec::new(),

                active_experiment_parameters   : Parameters::new(),
                active_experiment_instance     : None,
                active_experiment_state        : ExperimentStage::NotInExperiment,
                active_experiment_shapes_to_use: Shapes::create_vec_shape_bool(),
                n_v                            : 0.0,
                n_a                            : 0.0,
                error_info                     : None,
                time_main                      : Instant::now(),
                is_time_counting               : false,
                time_start                     : 0,
                time_end                       : 0,
                active_max_idx                 : 0,
                active_idx                     : 0,

                results_paths     : Vec::new(),
                active_result_item: None,

                logs            : Vec::new(),
                active_config   : Config::default(),
                is_debug_view   : false,
                is_simulate_mode: false,
                active_theme    : Theme::KanagawaDragon,
            },
            // Font is optional.
            Task::batch([font::load(
                include_bytes!("../fonts/porter-sans-inline-block.ttf").as_slice(),
            )
            .map(Message::FontLoaded)]),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        if let Message::FontLoaded(fontloaded_result) = message {
            if fontloaded_result.is_err() {
                log::error!("Failed to load font!");
            } else {
                self.is_title_font_ok = true;
            }
            return Task::perform(load_file_content("./.config.json".to_string()), |res| {
                Message::ConfigFileLoaded(res.0, res.1)
            });
        }

        if let Message::ConfigFileLoaded(config, errmsg) = message {
            if errmsg.is_some() {
                self.log(
                    LT::E,
                    format!(
                        "Config file failed to load. {}. Fallback to using default values...",
                        errmsg.unwrap()
                    ),
                );
                configure_startup();
            } else {
                match serde_json::from_str(config.unwrap().as_str()) {
                    Ok(config_json) => self.active_config = config_json,
                    Err(errmsg) => {
                        self.log(LT::E,
                            format!("Failed to parse .config.json file. Fallback to default values ...{:?}",
                            errmsg));
                        configure_startup();
                    }
                }
            }
            return Task::perform(
                update_dir_lists(self.active_config.sequences_path.clone()),
                |res| Message::SequencesListsLoaded(res.0, res.1),
            );
        }

        if let Message::SequencesListsLoaded(seqlistsopt, erropt) = message {
            if let Some(errmsg) = erropt {
                //handle error here vro
                self.log(LT::E, format!("Sequences failed to load! {}", errmsg));
            } else {
                self.sequences_paths = seqlistsopt.unwrap()
            }
            return Task::none();
        }

        if let Message::ResultsListsUpdated(reslistopt, erropt) = message {
            self.is_busy = false;
            if let Some(errmsg) = erropt {
                self.log(
                    LT::E,
                    format!("Result paths list failed to load! {}", errmsg),
                );
            } else {
                self.results_paths = reslistopt.unwrap()
            }
            return Task::none();
        }

        if let Message::TabSelected(selected_tab) = message {
            return handle_tabs(self, selected_tab);
        }

        if let Message::SeqTabSelected(selected_tab) = message {
            return handle_sequencer_tabs(self, selected_tab);
        }

        match self.active_main_tab {
            Tabpage::Sequencer => {
                if let Message::DobotGoHome = message {
                    self.is_busy = true;
                    return Task::perform(go_home(self.dobot()), Message::DobotResult);
                } else if let Message::DobotTestConnection = message {
                    self.is_busy = true;
                    return Task::perform(test_connection(self.dobot()), Message::DobotResult);
                } else if let Message::DobotResult(erropt) = message {
                    if let Some(errmsg) = erropt {
                        self.log(LT::E, format!("Dobot error: {}", errmsg));
                    } else {
                        self.log(LT::I, "Dobot ok");
                    }
                    self.is_busy = false;
                    return Task::none();
                } else {
                }
                match self.active_sequence_tab {
                    SequencerTabpage::Sequencer => {
                        match message {
                            Message::SequenceReorder(event) => {
                                match event {
                                    DragEvent::Dropped {
                                        index,
                                        target_index,
                                        drop_position,
                                    } => match drop_position {
                                        DropPosition::Swap => {
                                            if target_index != index {
                                                self.active_sequencer_sequences
                                                    .swap(index, target_index);
                                            }
                                        }
                                        DropPosition::Before | DropPosition::After => {
                                            if target_index != index && target_index != index + 1 {
                                                let item =
                                                    self.active_sequencer_sequences.remove(index);
                                                let insert_idx = if index < target_index {
                                                    target_index - 1
                                                } else {
                                                    target_index
                                                };
                                                self.active_sequencer_sequences
                                                    .insert(insert_idx, item);
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                                Task::none()
                            }
                            Message::SequencerMoveToPressed => {
                                log::info!("Moving to position...");

                                Task::perform(
                                    perform_moveto(
                                        self.dobot(),
                                        self.active_sequencer_sequence.clone(),
                                    ),
                                    Message::SequencerPerformResult,
                                )
                            }
                            Message::SequencerAddToSequencesPressed => {
                                match self
                                    .active_sequencer_sequence
                                    .clone()
                                    .into_iter()
                                    .any(|item: &PositionItem| item.in_string.is_empty())
                                {
                                    true => {
                                        self.log(
                                            LT::E,
                                            "Cannot add to queue, some input(s) are empty",
                                        );
                                    }
                                    false => {
                                        self.active_sequencer_sequences
                                            .push(self.active_sequencer_sequence.clone());
                                    }
                                }
                                Task::none()
                            }
                            Message::SequencerInputUpdated(coordtype, input_val) => {
                                if input_val.is_empty() {
                                    match coordtype {
                                        Coordinate::X => self
                                            .active_sequencer_sequence
                                            .x
                                            .update_s_f32(input_val, 0.0),
                                        Coordinate::Y => self
                                            .active_sequencer_sequence
                                            .y
                                            .update_s_f32(input_val, 0.0),
                                        Coordinate::Z => self
                                            .active_sequencer_sequence
                                            .z
                                            .update_s_f32(input_val, 0.0),
                                        Coordinate::R => self
                                            .active_sequencer_sequence
                                            .r
                                            .update_s_f32(input_val, 0.0),
                                        Coordinate::V => self
                                            .active_sequencer_sequence
                                            .v
                                            .update_s_f32(input_val, 0.0),
                                        Coordinate::A => self
                                            .active_sequencer_sequence
                                            .a
                                            .update_s_f32(input_val, 0.0),
                                    }
                                } else {
                                    match input_val.parse::<f32>() {
                                        Ok(value) => match coordtype {
                                            Coordinate::X => self
                                                .active_sequencer_sequence
                                                .x
                                                .update_s_f32(input_val, value),
                                            Coordinate::Y => self
                                                .active_sequencer_sequence
                                                .y
                                                .update_s_f32(input_val, value),
                                            Coordinate::Z => self
                                                .active_sequencer_sequence
                                                .z
                                                .update_s_f32(input_val, value),
                                            Coordinate::R => self
                                                .active_sequencer_sequence
                                                .r
                                                .update_s_f32(input_val, value),
                                            Coordinate::V => self
                                                .active_sequencer_sequence
                                                .v
                                                .update_s_f32(input_val, value),
                                            Coordinate::A => self
                                                .active_sequencer_sequence
                                                .a
                                                .update_s_f32(input_val, value),
                                        },
                                        Err(_) => {
                                            self.log(
                                                LT::W,
                                                format!(
                                                    "Input {} not parseable to f32",
                                                    coordtype.to_string()
                                                ),
                                            );
                                        }
                                    }
                                }
                                Task::none()
                            }
                            Message::RemoveASequencePressed(index) => {
                                let _ = self.active_sequencer_sequences.remove(index);
                                Task::none()
                            }
                            Message::PasteSequencePressed(index) => {
                                self.active_sequencer_sequence =
                                    self.active_sequencer_sequences[index].clone();
                                Task::none()
                            }
                            Message::ClearSequences => {
                                self.active_sequencer_sequences = Vec::new();
                                Task::none()
                            }
                            Message::ClearSeqInput => {
                                self.active_sequencer_sequence = Position::default();
                                Task::none()
                            }
                            Message::SeqFilenameInputUpdated(intext) => {
                                self.active_seq_path_input = intext;
                                Task::none()
                            }
                            Message::SeqSaveFilePressed => {
                                if self.active_sequencer_sequences.is_empty() {
                                    self.log(LT::W, "Sequences empty, not saving...");
                                    Task::none()
                                } else {
                                    self.is_busy = true;
                                    log::info!("Saving as {}", self.active_seq_path_input);
                                    match serde_json::to_string_pretty(&NamedSequence {
                                        name: self.active_seq_path_input.clone(),
                                        sequences: self.active_sequencer_sequences.clone(),
                                    }) {
                                        Ok(json_content) => Task::perform(
                                            save_str_to_json(
                                                self.active_config.sequences_path.clone(),
                                                json_content.clone(),
                                                self.active_seq_path_input.clone(),
                                            ),
                                            Message::SeqSaveFileResult,
                                        ),
                                        Err(errmsg) => {
                                            self.log(
                                                LT::E,
                                                format!("Error while saving: {}", errmsg),
                                            );
                                            Task::none()
                                        }
                                    }
                                }
                            }
                            Message::SeqSaveFileResult(erroption) => {
                                self.is_busy = false;
                                if let Some(errmsg) = erroption {
                                    self.log(
                                        LT::E,
                                        format!(
                                            "Failed to save {} to file. {}",
                                            self.active_seq_path_input, errmsg
                                        ),
                                    );
                                    Task::none()
                                } else {
                                    self.log(LT::I, "Save successful.");
                                    Task::perform(
                                        update_dir_lists(self.active_config.sequences_path.clone()),
                                        |(_ok, _err)| Message::SequencesListsLoaded(_ok, _err),
                                    )
                                }
                            }
                            Message::SequencesListsLoaded(ok, err) => {
                                if let Some(errmsg) = err {
                                    self.log(LT::E, errmsg);
                                } else {
                                    self.sequences_paths = ok.unwrap();
                                }
                                Task::none()
                            }
                            Message::SequencerPerform => {
                                self.is_busy = true;
                                Task::perform(
                                    perform_sequences(
                                        DobotPath {
                                            dobotpath: self.active_config.dobot_path.clone(),
                                        },
                                        self.active_sequencer_sequences.clone(),
                                    ),
                                    Message::SequencerPerformResult,
                                )
                            }
                            Message::SequencerPerformResult(erropt) => {
                                if let Some(errmsg) = erropt {
                                    self.log(LT::E, format!("Error running sequence: {}", errmsg));
                                }
                                self.is_busy = false;
                                Task::none()
                            }
                            _ => unimplemented!(),
                        }
                    }
                    SequencerTabpage::Sequences => match message {
                        Message::SeqLoadFilePressed(index, do_perform) => {
                            self.is_busy = true;
                            Task::perform(
                                load_file_content(self.sequences_paths[index].clone()),
                                move |(_ok, _err)| {
                                    Message::SeqLoadFileResult(_ok, _err, do_perform)
                                },
                            )
                        }
                        Message::SeqLoadFileResult(ok, err, do_perform) => {
                            if let Some(errmsg) = err {
                                self.is_busy = false;
                                self.log(LT::E, format!("Error loading sequence file: {}", errmsg));
                                return Task::none();
                            } else {
                                match serde_json::from_str::<NamedSequence>(ok.unwrap().as_str()) {
                                    Ok(some_json) => {
                                        if do_perform {
                                            self.active_sequences = some_json.sequences.clone();
                                            self.log(LT::I, format!("Running {}", some_json.name));
                                        } else {
                                            self.active_sequencer_sequences =
                                                some_json.sequences.clone();
                                            self.log(
                                                LT::I,
                                                format!("Loaded {} into sequencer", some_json.name),
                                            );
                                        }
                                    }
                                    Err(some_err) => {
                                        self.is_busy = false;
                                        self.log(
                                            LT::E,
                                            format!("Error parsing sequence data: {}", some_err),
                                        );
                                        return Task::none();
                                    }
                                }
                            }
                            if do_perform {
                                Task::perform(
                                    perform_sequences(
                                        DobotPath {
                                            dobotpath: self.active_config.dobot_path.clone(),
                                        },
                                        self.active_sequences.clone(),
                                    ),
                                    Message::PerformSequenceResult,
                                )
                            } else {
                                self.is_busy = false;
                                Task::none()
                            }
                        }
                        Message::PerformSequenceResult(erropt) => {
                            self.is_busy = false;
                            if let Some(errmsg) = erropt {
                                self.log(LT::E, format!("Performing sequences failed! {}", errmsg));
                            }
                            Task::none()
                        }
                        _ => unimplemented!(),
                    },
                }
            }
            Tabpage::Experiment => match self.active_experiment_state {
                ExperimentStage::NotInExperiment => match message {
                    Message::SelectedShapesChanged(selected_shape) => {
                        let _ = self
                            .active_experiment_shapes_to_use
                            .iter_mut()
                            .filter(|(shape, _)| selected_shape == *shape)
                            .map(|(_, is_use)| *is_use = !*is_use)
                            .collect::<Vec<_>>();
                        Task::none()
                    }
                    Message::ParameterInputChanged(param_type, invalue) => {
                        if invalue.is_empty() {
                            let _ = self
                                .active_experiment_parameters
                                .iter_mut()
                                .filter(|param| param.parameter_type == param_type)
                                .map(|item| {
                                    item.value = invalue.clone();
                                    ()
                                })
                                .collect::<Vec<()>>();
                        } else {
                            if let Some(_) = self
                                .active_experiment_parameters
                                .iter_mut()
                                .filter(|param| param.parameter_type == param_type)
                                .map(|item| {
                                    let mut ret_opt = None;
                                    match param_type {
                                        ParameterType::SubjectName
                                        | ParameterType::ModulationType
                                        | ParameterType::CarrierType => {
                                            item.value = invalue.clone();
                                        }
                                        _ => match invalue.parse::<u32>() {
                                            Ok(_) => item.value = invalue.clone(),
                                            Err(errmsg) => ret_opt = Some(errmsg),
                                        },
                                    }
                                    ret_opt
                                })
                                .collect::<Vec<Option<ParseIntError>>>()
                                .first()
                                .unwrap()
                            {
                                self.log(LT::W, format!("Not parseable as u32."));
                            };
                        }
                        Task::none()
                    }

                    Message::BeginExperimentPressed => {
                        let empty_check = self
                            .active_experiment_parameters
                            .into_iter()
                            .filter(|item| item.value.is_empty())
                            .map(|item| item.clone())
                            .collect::<Vec<_>>()
                            .clone();

                        if empty_check.len() != 0 {
                            for empty_params in empty_check {
                                self.log(
                                    LT::W,
                                    format!(
                                        "{} is empty",
                                        empty_params.parameter_type.show_title(false)
                                    ),
                                );
                                self.error_info = Some(format!(
                                    "{} is empty",
                                    empty_params.parameter_type.show_title(false)
                                ));
                            }
                        } else if self
                            .active_experiment_shapes_to_use
                            .iter()
                            .all(|(_, in_use)| *in_use == false)
                        {
                            self.log(LT::W, "No shape selection made!");
                            self.error_info = Some("Please select some shapes!".to_string());
                        } else {
                            self.log(LT::I, "Beginning experiment!");
                            self.error_info = Some("".to_string());
                            self.is_busy = true;
                            self.n_v = self
                                .active_experiment_parameters
                                .speed
                                .value
                                .parse::<f32>()
                                .unwrap();
                          
                            self.n_a = self
                                .active_experiment_parameters
                                .acceleration
                                .value
                                .parse::<f32>()
                                .unwrap();
                          
                            self.active_experiment_instance =
                                Some(create_experiment_instance(&self));

                            self.active_max_idx = self
                                .active_experiment_parameters
                                .test_count
                                .value
                                .parse::<u32>()
                                .unwrap() - 1;
                          
                            self.active_idx = 0;
                            self.active_experiment_state = ExperimentStage::BeginTiming;
                            return Task::perform(_wait_n_ms(50), |_| Message::GoToNextStage);
                        }
                        Task::none()
                    }
                    Message::ResultsProcessed(result_option, erroption) => {
                        if let Some(errmsg) = erroption {
                            self.log(LT::E, format!("Error while saving results! {}", errmsg));
                        } else {
                            self.active_result_item = result_option;
                            self.log(LT::I, "Saved successfully.");
                        }
                        Task::perform(
                            update_dir_lists(self.active_config.clone().results_path),
                            |(_ok, _err)| Message::ResultsListsUpdated(_ok, _err),
                        )
                    }
                    _ => unimplemented!(),
                },
                ExperimentStage::BeginTiming => match message {
                    Message::GoToNextStage => {
                        self.is_time_counting = false;
                        self.active_experiment_state = ExperimentStage::Preparation;
                        Task::perform(_wait_n_ms(50), |_| Message::GoToNextStage)
                    }
                    Message::ForceAbortPressed => {
                        self.log(LT::W, "Force abort button pressed.");
                        self.reset_experiment_variable();
                        Task::none()
                    }
                    _ => unimplemented!(),
                },
                ExperimentStage::Preparation => match message {
                    Message::GoToNextStage => {
                        let cur_exp_item = self.active_experiment_instance.clone().unwrap();

                        if self.is_simulate_mode {
                            Task::perform(simulate_drawing(), Message::DrawingResult)
                        } else {
                            Task::perform(
                                draw_shape(
                                    self.dobot(),
                                    self.relay(),
                                    cur_exp_item.list_of_shapes[cur_exp_item.experiment_index],
                                    self.n_v,
                                    self.n_a,
                                ),
                                Message::DrawingResult,
                            )
                        }
                    }
                    Message::DrawingResult(erropt) => {
                        if let Some(errmsg) = erropt {
                            self.log(
                                LT::E,
                                format!(
                                    "Error while in experiment, aborting experiment... {}",
                                    errmsg
                                ),
                            );
                            self.reset_experiment_variable();
                        } else {
                            if !self.is_time_counting {
                                self.time_start = self.time_main.elapsed().as_millis();
                                self.is_time_counting = true;
                            }
                            self.active_experiment_state = ExperimentStage::Answering;
                        }
                        Task::none()
                    }
                    Message::ForceAbortPressed => {
                        self.log(LT::W, "Force abort button pressed.");
                        self.reset_experiment_variable();
                        Task::none()
                    }
                    _ => unimplemented!(),
                },
                ExperimentStage::Answering => match message {
                    Message::ShapeSelected(shape) => {
                        self.time_end = self.time_main.elapsed().as_millis();
                        let mut cur_exp_item = self.active_experiment_instance.clone().unwrap();

                        cur_exp_item.list_of_guesses.push(shape);
                        cur_exp_item
                            .list_of_time
                            .push(self.time_end - self.time_start);

                        if cur_exp_item.experiment_index
                            == cur_exp_item.list_of_shapes.capacity() - 1
                        {
                            self.is_busy = false;
                            self.active_experiment_state = ExperimentStage::NotInExperiment;
                            self.active_experiment_instance = Some(cur_exp_item);
                            Task::perform(process_results(self.clone()), |(_ok, _err)| {
                                Message::ResultsProcessed(_ok, _err)
                            })
                        } else {
                            cur_exp_item.experiment_index += 1;
                            self.active_idx += 1;
                            self.active_experiment_instance = Some(cur_exp_item);
                            self.active_experiment_state = ExperimentStage::BeginTiming;
                            Task::perform(_wait_n_ms(50), |_| Message::GoToNextStage)
                        }
                    }
                    Message::ForceAbortPressed => {
                        self.log(LT::W, "Force abort button pressed.");
                        self.reset_experiment_variable();
                        Task::none()
                    }
                    Message::RetryButtonPressed => {
                        self.log(LT::I, "Retry button pressed.");
                        let mut cur_exp_item = self.active_experiment_instance.clone().unwrap();
                        cur_exp_item.list_of_retries[cur_exp_item.experiment_index] += 1;
                        self.active_experiment_instance = Some(cur_exp_item);
                        self.active_experiment_state = ExperimentStage::Preparation;
                        Task::perform(_wait_n_ms(50), |_| Message::GoToNextStage)
                    }
                    _ => unimplemented!(),
                },
            },
            Tabpage::Results => match message {
                Message::UpdateResultsList => {
                    self.is_busy = true;
                    Task::perform(
                        update_dir_lists(self.active_config.results_path.clone()),
                        |(_ok, _err)| Message::ResultsListsUpdated(_ok, _err),
                    )
                }
                Message::ResultButtonPressed(result_path) => {
                    self.is_busy = true;
                    Task::perform(load_file_content(result_path), |(_ok, _err)| {
                        Message::ResultButtonResult(_ok, _err)
                    })
                }
                Message::ResultButtonResult(content_opt, erropt) => {
                    self.is_busy = false;
                    if let Some(errmsg) = erropt {
                        self.log(LT::E, format!("Failed to load selected result! {}", errmsg));
                    } else {
                        match serde_json::from_str(content_opt.unwrap().as_str()) {
                            Ok(result_json) => self.active_result_item = Some(result_json),
                            Err(errmsg) => {
                                self.log(LT::E, format!("Could not parse result json! {}", errmsg))
                            }
                        }
                    }
                    Task::none()
                }
                _ => unimplemented!(),
            },
            Tabpage::Settings => match message {
                Message::DebugCheckboxPressed(somebool) => {
                    self.is_debug_view = somebool;
                    Task::none()
                }
                Message::SimulateModeCheckboxPressed(somebool) => {
                    self.is_simulate_mode = somebool;
                    Task::none()
                }
                Message::ThemeSelected(theme) => {
                    self.active_theme = theme;
                    Task::none()
                }
                _ => unimplemented!(),
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut final_column = Column::new().spacing(10).padding(5);
        final_column = final_column.push(generate_header(self.is_busy, self.is_title_font_ok));
        final_column = final_column.push(horizontal_rule(10));
        final_column = final_column.push(match self.active_main_tab {
            Tabpage::Sequencer => generate_sequencer_tab(self),
            Tabpage::Experiment => generate_experiment_tab(self),
            Tabpage::Results => generate_result_tab(self),
            Tabpage::Settings => generate_settings_tab(self),
        });

        if self.active_main_tab == Tabpage::Sequencer {
            final_column = final_column.push(
                container(generate_error_log(self))
                    .style(cont_w_2_10)
                    .padding(10)
                    .height(150)
                    .width(Length::Fill),
            );
        }

        let trans2elem: Element<'_, Message> = container(
            container(final_column)
                .padding(10)
                .height(Length::Fill)
                .width(Length::Fill)
                .style(cont_w_2_10),
        )
        .padding(5)
        .into();

        // Make beautiful debug view
        if self.is_debug_view {
            trans2elem.explain(iced::Color::from_rgb(200.0, 0.0, 0.0))
        } else {
            trans2elem
        }
        .into()
    }

    fn theme(&self) -> Theme {
        self.active_theme.clone()
    }

    fn reset_experiment_variable(&mut self) {
        self.active_experiment_state = ExperimentStage::NotInExperiment;
        self.active_experiment_instance = None;
        self.is_busy = false;
        self.n_v = 0.0;
        self.n_a = 0.0;
        self.time_start = 0;
        self.time_end = 0;
        self.is_time_counting = false;
    }

    fn dobot(&self) -> DobotPath {
        DobotPath {
            dobotpath: self.active_config.dobot_path.clone(),
        }
    }

    fn relay(&self) -> RelayPath {
        RelayPath {
            relaypath: self.active_config.relay_path.clone(),
        }
    }

    fn log<T: Into<String> + std::fmt::Display>(&mut self, logtype: LogType, logmsg: T) {
        match logtype {
            LogType::I => {
                log::info!("{}", logmsg);
                append_log(&mut self.logs, LogType::I, logmsg.into());
            }
            LogType::W => {
                log::warn!("{}", logmsg);
                append_log(&mut self.logs, LogType::W, logmsg.into());
            }
            LogType::E => {
                log::error!("{}", logmsg);
                append_log(&mut self.logs, LogType::E, logmsg.into());
            }
        }
    }
}

fn generate_header<'a>(is_busy: bool, is_font_ok: bool) -> Element<'a, Message> {
    let tab_button_row = Tabpage::into_iter()
        .map(|page| {
            button(text_ccff_container(page.to_string()))
                .width(92)
                .height(40)
                .on_press_maybe(if is_busy {
                    None
                } else {
                    Some(Message::TabSelected(page))
                })
        })
        .fold(Row::new(), |mut accu, pagetext| {
            accu = accu.push(pagetext);
            accu
        });

    row![
        text("DOBOT CONTROLLER 3001")
            .font(porter_sans_inline_font(is_font_ok))
            .size(26),
        horizontal_space(),
        tab_button_row.spacing(10)
    ]
    .align_y(Alignment::Center)
    .spacing(10)
    .height(40)
    .into()
}

fn generate_sequencer_tab<'a>(appv: &IcedDobotController) -> Element<'a, Message> {
    let seqtab_button_row = SequencerTabpage::into_iter()
        .map(|seqpage| {
            button(text_ccff_container(seqpage.to_string()))
                .width(92)
                .height(40)
                .on_press_maybe(if appv.is_busy {
                    None
                } else {
                    Some(Message::SeqTabSelected(seqpage))
                })
        })
        .fold(Row::new(), |mut accu, seqtabbutton| {
            accu = accu.push(seqtabbutton);
            accu
        });

    let seqtab_content = match appv.active_sequence_tab {
        SequencerTabpage::Sequencer => generate_sequencer_content(appv),
        SequencerTabpage::Sequences => generate_sequences_content(appv),
    };

    let dobot_buttonrow = row![
        button(text_ccff_container("Home"))
            .on_press_maybe(if appv.is_busy {
                None
            } else {
                Some(Message::DobotGoHome)
            })
            .width(92)
            .height(40),
        button(text_ccff_container("Test"))
            .on_press_maybe(if appv.is_busy {
                None
            } else {
                Some(Message::DobotTestConnection)
            })
            .width(92)
            .height(40)
    ]
    .spacing(10);

    column![
        row![
            dobot_buttonrow,
            horizontal_space(),
            seqtab_button_row.spacing(10)
        ],
        seqtab_content
    ]
    .spacing(10)
    .into()
}

fn generate_sequencer_content<'a>(appv: &IcedDobotController) -> Element<'a, Message> {
    let coords_inputs = Coordinate::into_iter()
        .zip(appv.active_sequencer_sequence.into_iter())
        .map(|(coord, active_coord)| {
            row![
                row![text(coord.to_string()).width(10), text(":").width(10)]
                    .align_y(Alignment::Center),
                text_input("", &active_coord.in_string)
                    .width(80)
                    .on_input_maybe(if appv.is_busy {
                        None
                    } else {
                        Some(move |input_text| Message::SequencerInputUpdated(coord, input_text))
                    })
            ]
            .align_y(Alignment::Center)
            .height(30)
        })
        .fold(Row::new().spacing(10), |mut accu, coorditem| {
            accu = accu.push(coorditem);
            accu
        });

    let is_any_inputs_empty = appv
        .active_sequencer_sequence
        .into_iter()
        .any(|item| item.in_string.is_empty());
    let input_section = column![
        text("Input coordinates, servo rotation, velocity, acceleration below."),
        row![
            coords_inputs,
            horizontal_space(),
            row![
                button("Move to Pos").on_press_maybe(if appv.is_busy || is_any_inputs_empty {
                    None
                } else {
                    Some(Message::SequencerMoveToPressed)
                }),
                button("Add to Seq").on_press_maybe(if appv.is_busy || is_any_inputs_empty {
                    None
                } else {
                    Some(Message::SequencerAddToSequencesPressed)
                }),
                button("Empty")
                    .on_press_maybe(if appv.is_busy {
                        None
                    } else {
                        Some(Message::ClearSeqInput)
                    })
                    .style(|_t, _s| button::secondary(_t, _s)),
            ]
            .spacing(10)
        ],
    ]
    .spacing(10);

    let queue_section = scrollable(
        dragking::column(
            appv.active_sequencer_sequences
                .iter()
                .enumerate()
                .map(|(idx, sequence)| {
                    (
                        idx,
                        sequence
                            .into_iter()
                            .zip(Coordinate::into_iter())
                            .enumerate()
                            .fold(Row::new(), |mut accu, (indx, (pos, posname))| {
                                accu = accu.push(text(format!("{}: ", posname)));
                                if indx == 5 {
                                    accu = accu.push(text(format!("{}", pos.in_string)));
                                } else {
                                    accu = accu.push(text(format!("{}, ", pos.in_string)));
                                }
                                accu
                            }),
                    )
                })
                .map(|(idx, itemrow)| {
                    row![
                        container(
                            row![
                                text(format!("{}:", idx)),
                                itemrow,
                                horizontal_space(),
                                button("Remove").on_press_maybe(if appv.is_busy {
                                    None
                                } else {
                                    Some(Message::RemoveASequencePressed(idx))
                                }),
                                button("Paste to input").on_press_maybe(if appv.is_busy {
                                    None
                                } else {
                                    Some(Message::PasteSequencePressed(idx))
                                }),
                            ]
                            .align_y(Alignment::Center)
                            .spacing(10)
                        )
                        .padding(Padding {
                            top: 5.0,
                            right: 10.0,
                            bottom: 5.0,
                            left: 10.0
                        })
                        .style(cont_gray_0_10),
                        horizontal_space().width(10)
                    ]
                    .into()
                })
                .collect::<Vec<Element<'_, Message>>>(),
        )
        .on_drag(Message::SequenceReorder)
        .spacing(5),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .anchor_bottom();

    let save_section = row![
        text("File name"),
        text(":"),
        text_input(
            "Enter save filename here",
            appv.active_seq_path_input.as_str()
        )
        .on_input_maybe(if appv.is_busy {
            None
        } else {
            Some(Message::SeqFilenameInputUpdated)
        }),
        button("Save").width(92).width(92).on_press_maybe(
            if appv.is_busy
                || appv.active_seq_path_input.is_empty()
                || appv.active_sequencer_sequences.is_empty()
            {
                None
            } else {
                Some(Message::SeqSaveFilePressed)
            }
        ),
        button("Perform").width(92).on_press_maybe(
            if appv.is_busy || appv.active_sequencer_sequences.is_empty() {
                None
            } else {
                Some(Message::SequencerPerform)
            }
        ),
        button("Clear")
            .width(92)
            .on_press_maybe(
                if appv.is_busy || appv.active_sequencer_sequences.is_empty() {
                    None
                } else {
                    Some(Message::ClearSequences)
                }
            )
            .style(|_t, _s| button::secondary(_t, _s)),
    ]
    .align_y(Alignment::Center)
    .height(30)
    .spacing(10);

    let final_view = column![
        container(input_section).style(cont_w_2_10).padding(10),
        container(queue_section).style(cont_w_2_10).padding(10),
        container(save_section).style(cont_w_2_10).padding(10),
    ]
    .spacing(10);
    final_view.into()
}

fn generate_sequences_content<'a>(appv: &IcedDobotController) -> Element<'a, Message> {
    if appv.sequences_paths.is_empty() {
        text(format!(
            "No sequences found in path \'{}\'",
            appv.active_config.sequences_path
        ))
        .into()
    } else {
        scrollable(
            container(
                appv.sequences_paths
                    .iter()
                    .enumerate()
                    .map(|(idx, paths)| {
                        row![
                            container(
                                row![
                                    text(format!("{}: ", idx)),
                                    text(format!("{} ", paths)),
                                    horizontal_space(),
                                    button("Load").on_press_maybe(if appv.is_busy {
                                        None
                                    } else {
                                        Some(Message::SeqLoadFilePressed(idx, false))
                                    }),
                                    button("Perform").on_press_maybe(if appv.is_busy {
                                        None
                                    } else {
                                        Some(Message::SeqLoadFilePressed(idx, true))
                                    }),
                                ]
                                .align_y(Alignment::Center)
                                .spacing(10)
                            )
                            .padding(5)
                            .style(cont_gray_0_10),
                            Space::with_width(Length::Fixed(15.0))
                        ]
                    })
                    .fold(
                        Column::new().spacing(10).width(Length::Fill),
                        |mut accu, rows| {
                            accu = accu.push(rows);
                            accu
                        },
                    ),
            )
            .style(cont_w_2_10)
            .padding(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn generate_error_log<'a>(appv: &IcedDobotController) -> Element<'a, Message> {
    let errorcols = appv
        .logs
        .iter()
        .map(|error_message| {
            container(row![
                text(format!("{} ", error_message.index)).width(if error_message.index < 100 {
                    20
                } else if error_message.index >= 100 && error_message.index < 1000 {
                    30
                } else {
                    40
                }),
                text(format!(
                    "{}",
                    match error_message.kind {
                        LT::I => "Info ",
                        LT::W => "Warn ",
                        LT::E => "Error ",
                    }
                ))
                .shaping(text::Shaping::Advanced),
                text(":").width(10),
                text(format!("{}", error_message.logmsg)).width(Length::Fill),
            ])
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Alignment::Start)
            .padding(Padding {
                top: 0.0,
                right: 20.0,
                bottom: 0.0,
                left: 10.0,
            })
            .style(match error_message.kind {
                LT::I => cont_25510f_0_10,
                LT::W => cont_532206_0_10,
                LT::E => cont_510515_0_10,
            })
        })
        .fold(Column::new().spacing(5), |mut accu, erritem| {
            accu = accu.push(erritem);
            accu
        });

    scrollable(errorcols).anchor_bottom().into()
}

fn generate_experiment_tab<'a>(appv: &'a IcedDobotController) -> Element<'a, Message> {
    if appv.active_experiment_state == ExperimentStage::NotInExperiment {
        ccff_container(container(
            column![
                appv.active_experiment_parameters
                    .into_iter()
                    .fold(Column::new(), |mut accu, parameter| {
                        {
                            accu = accu.push(make_exp_prep_input(
                                parameter.parameter_type.show_title(true),
                                &parameter.parameter_type.show_title_en(),
                                &parameter.value,
                                |input_val| {
                                    Message::ParameterInputChanged(
                                        parameter.parameter_type.clone(),
                                        input_val,
                                    )
                                },
                            ));
                            accu
                        }
                    })
                    .align_x(Alignment::Center)
                    .padding(10)
                    .spacing(10),
                button(text_ccff_container("Start"))
                    .on_press(Message::BeginExperimentPressed)
                    .width(350)
                    .height(30),
                text(if let Some(err_info) = &appv.error_info {
                    err_info.to_string()
                } else {
                    String::from("")
                }),
                appv.active_experiment_shapes_to_use.iter().fold(
                    Row::new().spacing(10),
                    |mut accu, (shape, is_in_use)| {
                        accu = accu.push(
                            checkbox(shape.show_name_en(), *is_in_use)
                                .on_toggle(|_| Message::SelectedShapesChanged(shape.clone())),
                        );
                        accu
                    }
                ),
            ]
            .align_x(Alignment::Center),
        ))
        .into()
    } else {
        let enable_buttons = appv.active_experiment_state == ExperimentStage::Answering;
        let shape_selection = appv
            .active_experiment_shapes_to_use
            .iter()
            .filter(|(_, do_use)| *do_use)
            .map(|(active_shape, _)| active_shape)
            .fold(Row::new().spacing(10), |mut accu, shape| {
                accu = accu.push(
                    button(text_size_ccff_container(shape.to_string(), 25))
                        .width(200)
                        .height(50)
                        .on_press_maybe(if !enable_buttons {
                            None
                        } else {
                            Some(Message::ShapeSelected(shape.clone()))
                        }),
                );
                accu
            });

        ccff_container(
            column![
                text(format!(
                    "実験 {} / {}",
                    appv.active_idx + 1, appv.active_max_idx + 1
                ))
                .size(20),
                progress_bar(0.0..=appv.active_max_idx as f32, appv.active_idx as f32).width(400),
                shape_selection.wrap(),
                button(text_size_ccff_container(" ⟲ リトライ ⟲ ", 20))
                    .width(200)
                    .height(50)
                    .on_press_maybe(if !enable_buttons {
                        None
                    } else {
                        Some(Message::RetryButtonPressed)
                    })
                    .style(button::secondary),
                button(text_size_ccff_container(" ⟁ 中断 ⟁ ", 20))
                    .width(200)
                    .height(50)
                    .on_press(Message::ForceAbortPressed)
                    .style(button::danger),
            ]
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .into()
    }
}

fn generate_result_tab<'a>(appv: &IcedDobotController) -> Element<'a, Message> {
    row![
        container(scrollable(
            column![
                button("Refresh")
                    .width(Length::Fill)
                    .on_press_maybe(if appv.is_busy {
                        None
                    } else {
                        Some(Message::UpdateResultsList)
                    }),
                appv.results_paths
                    .iter()
                    .map(|pathstring| {
                        button(container(text(pathstring.clone())))
                            .width(Length::Fill)
                            .on_press_maybe(if appv.is_busy {
                                None
                            } else {
                                Some(Message::ResultButtonPressed(pathstring.clone()))
                            })
                    })
                    .fold(Column::new().spacing(10), |mut accu, pathbutton| {
                        accu = accu.push(pathbutton);
                        accu
                    })
            ]
            .spacing(10)
        ))
        .height(Length::Fill)
        .width(Length::FillPortion(1))
        .padding(10)
        .style(cont_w_2_10),
        container(column![generate_results_summary(
            appv.active_result_item.clone()
        )])
        .height(Length::Fill)
        .width(Length::FillPortion(1))
        .padding(10)
        .style(cont_w_2_10),
    ]
    .spacing(10)
    .into()
}

fn generate_results_table<'a>(result_option: Option<ResultExports>) -> Element<'a, Message> {
    if let Some(result_content) = result_option {
        let mut table_column = Column::new();

        let headers = row![
            generate_table_container(text("Shape")),
            generate_table_container(text("Guess")),
            generate_table_container(text("Result")),
            generate_table_container(text("Time")),
            generate_table_container(text("Retries")),
        ];

        table_column = table_column.push(headers);

        for idx in 0..result_content
            .parameters
            .test_count
            .value
            .parse::<usize>()
            .unwrap()
        {
            table_column = table_column.push(row![
                //1 Shape
                generate_table_container(
                    text(result_content.results[idx].true_shape.show_name_symbol())
                        .shaping(text::Shaping::Advanced)
                ),
                //2 Guess
                generate_table_container(
                    text(result_content.results[idx].guess_shape.show_name_symbol())
                        .shaping(text::Shaping::Advanced)
                ),
                //3 Result
                generate_table_container(match result_content.results[idx].is_correct {
                    true => {
                        text("CORRECT").color(Color::from_rgb8(200, 200, 200))
                    }
                    false => {
                        text("WRONG")
                    }
                }),
                //4 Time
                generate_table_container(text(format!(
                    "{} [ms]",
                    result_content.results[idx].time
                ))),
                //5 Retries
                generate_table_container(text(format!("{}", result_content.results[idx].retries)))
            ])
        }
        table_column.into()
    } else {
        text("").into()
    }
}

fn generate_table_container<'a, T: Into<Element<'a, Message>>>(
    content: T,
) -> Container<'a, Message, Theme, iced::Renderer> {
    container(content)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
}

fn generate_results_summary<'a>(result_option: Option<ResultExports>) -> Element<'a, Message> {
    if let Some(result) = result_option.clone() {
        column![
            scrollable(make_analysis_view(result)).height(Length::FillPortion(1)),
            horizontal_rule(1),
            scrollable(generate_results_table(result_option)).height(Length::FillPortion(1)),
        ]
        .spacing(10)
        .into()
    } else {
        text("Select an item to view.").into()
    }
}

fn make_summary_label<'a, T: Into<String> + iced::widget::text::IntoFragment<'a>>(
    label: T,
    value: T,
    unit: Option<T>,
) -> Element<'a, Message> {
    let mut summary_row = Row::new();
    summary_row = summary_row.push(text(label).shaping(text::Shaping::Advanced).width(170));
    summary_row = summary_row.push(text(" : ").width(20));
    summary_row = summary_row.push(text(value));
    if let Some(some_t) = unit {
        summary_row = summary_row.push(text(some_t))
    }
    summary_row.into()
}

fn make_analysis_label<
    'a,
    T: Into<String> + iced::widget::text::IntoFragment<'a> + std::fmt::Display,
>(
    label: T,
    value: T,
    total: T,
) -> Element<'a, Message> {
    row![
        text(label).shaping(text::Shaping::Advanced).width(100),
        text(" : ").width(20),
        text(format!("{} / {}", value, total)),
    ]
    .into()
}

fn make_shape_analysis<'a>(analysed_shape: &mut ShapeAnalysis, total: u32) -> Element<'a, Message> {
    row![
        text(analysed_shape.main_shape.show_name_en()).width(165),
        text(" : ").width(20),
        {
            analysed_shape.wrong_shapes.iter().fold(
                {
                    let mut in_col = Column::new();
                    in_col = in_col.push(make_analysis_label(
                        analysed_shape.main_shape.show_name_en(),
                        analysed_shape.main_shape_count.to_string(),
                        total.to_string(),
                    ));
                    in_col
                },
                |mut accu, (shape_item, shape_count)| {
                    accu = accu.push(make_analysis_label(
                        shape_item.show_name_en(),
                        shape_count.to_string(),
                        total.to_string(),
                    ));
                    accu
                },
            )
        },
    ]
    .into()
}

fn row_container_space<'a, T: Into<Element<'a, Message>>>(content: T) -> Element<'a, Message> {
    row![
        container(content)
            .width(Length::Fill)
            .style(cont_gray_0_10)
            .padding(5),
        Space::with_width(Length::Fixed(15.0))
    ]
    .into()
}

fn make_analysis_view<'a>(result: ResultExports) -> Element<'a, Message> {
    let mut summary_column = Column::new().spacing(10);

    summary_column = summary_column.push(text("Summary"));

    let mut move_column =
        result
            .parameters
            .into_iter()
            .fold(summary_column, |mut accu, parameter| {
                accu = accu.push(make_summary_label(
                    parameter.parameter_type.show_title(false).to_string(),
                    parameter.value.clone(),
                    parameter.parameter_type.show_unit(),
                ));
                accu
            });

    move_column = move_column.push(make_summary_label(
        "結果・Results".to_string(),
        format!(
            "{} / {}",
            result.avg_correct_answers, result.parameters.test_count.value
        ),
        None,
    ));

    move_column = move_column.push(make_summary_label(
        "正解率・Results Rate".to_string(),
        format!(
            "{}",
            result.avg_correct_answers as f32
                / result.parameters.test_count.value.parse::<f32>().unwrap()
                * 100.0
        ),
        Some(String::from(" [%]")),
    ));

    move_column = move_column.push(make_summary_label(
        "平均時間・Avg. Time".to_string(),
        format!("{}", result.avg_time),
        Some(String::from(" [ms]")),
    ));

    let move_column = result
        .clone()
        .analyses
        .iter_mut()
        .map(|shape_analysis| {
            let totals = shape_analysis.calc_total_self_shape();
            let analysis_view = make_shape_analysis(shape_analysis, totals);
            analysis_view
        })
        .fold(move_column, |mut accu, analysis_elem| {
            accu = accu.push(row_container_space(analysis_elem));
            accu
        });

    move_column.into()
}

fn generate_settings_tab<'a>(appv: &IcedDobotController) -> Element<'a, Message> {
    column![
        checkbox("Toggle Debug View", appv.is_debug_view).on_toggle(Message::DebugCheckboxPressed),
        checkbox("Toggle Unconnected Mode", appv.is_simulate_mode)
            .on_toggle(Message::SimulateModeCheckboxPressed),
        pick_list(Theme::ALL, Some(appv.active_theme.clone()), |selection| {
            Message::ThemeSelected(selection)
        }),
    ]
    .spacing(10)
    .into()
}

/// Makes making text inside buttons easier.
fn text_ccff_container<'a, T: Into<String> + iced::widget::text::IntoFragment<'a>>(
    intext: T,
) -> Container<'a, Message, Theme, iced::Renderer> {
    container(text(intext).shaping(text::Shaping::Advanced))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Just width height fill, align xy center
fn ccff_container<'a, T: Into<Element<'a, Message>>>(
    inwidget: T,
) -> Container<'a, Message, Theme, iced::Renderer> {
    container(inwidget)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn text_size_ccff_container<'a, T: Into<String> + iced::widget::text::IntoFragment<'a>>(
    intext: T,
    textsize: u16,
) -> Container<'a, Message, Theme, iced::Renderer> {
    container(text(intext).size(textsize).shaping(text::Shaping::Advanced))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn handle_tabs(appv: &mut IcedDobotController, selected_tab: Tabpage) -> Task<Message> {
    if appv.active_main_tab != selected_tab {
        if selected_tab == Tabpage::Results {
            appv.active_main_tab = selected_tab;
            return Task::perform(
                update_dir_lists(appv.active_config.results_path.clone()),
                |(_ok, _err)| Message::ResultsListsUpdated(_ok, _err),
            );
        } else {
            appv.active_main_tab = selected_tab;
        }
    }
    Task::none()
}

fn handle_sequencer_tabs(
    appv: &mut IcedDobotController,
    selected_tab: SequencerTabpage,
) -> Task<Message> {
    if appv.active_sequence_tab != selected_tab {
        appv.active_sequence_tab = selected_tab
    }
    Task::none()
}

fn make_exp_prep_input<'a>(
    title: String,
    placeholder: &str,
    invalue: &str,
    message: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(title).width(130),
        text(":").width(15),
        text_input(placeholder, invalue).on_input(message)
    ]
    .width(350)
    .align_y(Alignment::Center)
    .into()
}
