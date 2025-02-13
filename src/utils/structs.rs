use serde::{Deserialize, Serialize};

pub struct DobotPath {
    pub dobotpath: String,
}

pub struct RelayPath {
    pub relaypath: String,
}

/// <-- FOR LOGGING PURPOSES
#[derive(Clone, Copy)]
pub enum LogType {
    I,
    W,
    E,
}

#[derive(Clone)]
pub struct LogMessage {
    pub index : u128,
    pub kind  : LogType,
    pub logmsg: String,
}
/// -->

/// PAGES
#[derive(Clone, Debug, PartialEq)]
pub enum Tabpage {
    Sequencer,
    Experiment,
    Results,
    Settings,
}

impl Tabpage {
    pub fn into_iter() -> core::array::IntoIter<Tabpage, 4> {
        [
            Tabpage::Sequencer,
            Tabpage::Experiment,
            Tabpage::Results,
            Tabpage::Settings,
        ]
        .into_iter()
    }
}

impl std::fmt::Display for Tabpage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tabpage::Sequencer  => write!(f, "⁂・運動"),
            Tabpage::Experiment => write!(f, "⟁・実験"),
            Tabpage::Results    => write!(f, "⧉・結果"),
            Tabpage::Settings   => write!(f, "⚙・設定"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SequencerTabpage {
    Sequencer,
    Sequences,
}

impl SequencerTabpage {
    pub fn into_iter() -> core::array::IntoIter<SequencerTabpage, 2> {
        [SequencerTabpage::Sequencer, SequencerTabpage::Sequences].into_iter()
    }
}

impl std::fmt::Display for SequencerTabpage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sequencer => write!(f, "⚫・作成"),
            Self::Sequences => write!(f, "▷・再生"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Coordinate {
    X,
    Y,
    Z,
    R,
    V,
    A,
}

impl Coordinate {
    pub fn into_iter() -> core::array::IntoIter<Coordinate, 6> {
        [
            Coordinate::X,
            Coordinate::Y,
            Coordinate::Z,
            Coordinate::R,
            Coordinate::V,
            Coordinate::A,
        ]
        .into_iter()
    }
}

impl std::fmt::Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Coordinate::X => write!(f, "X"),
            Coordinate::Y => write!(f, "Y"),
            Coordinate::Z => write!(f, "Z"),
            Coordinate::R => write!(f, "R"),
            Coordinate::V => write!(f, "V"),
            Coordinate::A => write!(f, "A"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionItem {
    pub in_string: String,
    pub in_float : f32,
}

impl PositionItem {
    pub fn new() -> Self {
        Self {
            in_string: String::new(),
            in_float : 0.0,
        }
    }
    pub fn new_float_only(value: f32) -> Self {
        Self {
            in_string: String::new(),
            in_float : value,
        }
    }
    pub fn update_s_f32(&mut self, value_string: String, value_f32: f32) {
        self.in_string = value_string;
        self.in_float  = value_f32;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: PositionItem,
    pub y: PositionItem,
    pub z: PositionItem,
    pub r: PositionItem,
    pub v: PositionItem,
    pub a: PositionItem,
}

impl Position {
    pub fn default() -> Self {
        Self {
            x: PositionItem::new(),
            y: PositionItem::new(),
            z: PositionItem::new(),
            r: PositionItem::new(),
            v: PositionItem::new(),
            a: PositionItem::new(),
        }
    }
    pub fn position<T: Into<f32>>(x: T, y: T, z: T, r: T, v: T, a: T) -> Self {
        Self {
            x: PositionItem::new_float_only(x.into()),
            y: PositionItem::new_float_only(y.into()),
            z: PositionItem::new_float_only(z.into()),
            r: PositionItem::new_float_only(r.into()),
            v: PositionItem::new_float_only(v.into()),
            a: PositionItem::new_float_only(a.into()),
        }
    }
    pub fn into_iter(&self) -> core::array::IntoIter<&PositionItem, 6> {
        [&self.x, &self.y, &self.z, &self.r, &self.v, &self.a].into_iter()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NamedSequence {
    pub name: String,
    pub sequences: Vec<Position>,
}

/// Shapes is very specific to my use case.
/// This is probably not useful for a general purpose sequencing.
/// However! You can re-implement this with your desired known sequence of position.
/// Just need to make sure to update the shapes in the enum and iter, position sequences, display text accoridingly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shapes {
    Triangle,
    Square,
    Pentagon,
    Hexagon,
}

impl Shapes {
    pub fn create_vec_shape_bool() -> Vec<(Shapes, bool)> {
        Shapes::into_iter()
            .map(|shape| (shape, false))
            .collect::<Vec<(Shapes, bool)>>()
    }
    pub fn into_iter() -> core::array::IntoIter<Shapes, 4> {
        [
            Shapes::Triangle,
            Shapes::Square,
            Shapes::Pentagon,
            Shapes::Hexagon,
        ]
        .into_iter()
    }
    pub fn motion_data(&self, velocity: f32, acceleration: f32) -> Vec<Position> {
        match self {
          /// Posititon needs to be in order.
            Shapes::Triangle => vec![
                Position::position(200.0, 15.0, -22.5, 0.0, velocity, acceleration),
                Position::position(200.0, -15.0, -22.5, 0.0, velocity, acceleration),
                Position::position(200.0, 0.0, 12.5, 0.0, velocity, acceleration),
            ],
            Shapes::Square => vec![
                Position::position(200.0, 15.0, 12.5, 0.0, velocity, acceleration),
                Position::position(200.0, 15.0, -22.5, 0.0, velocity, acceleration),
                Position::position(200.0, -15.0, -22.5, 0.0, velocity, acceleration),
                Position::position(200.0, -15.0, 12.5, 0.0, velocity, acceleration),
            ],
            Shapes::Pentagon => vec![
                Position::position(200.0, 16.0, 0.0, 0.0, velocity, acceleration),
                Position::position(200.0, 10.0, -19.0, 0.0, velocity, acceleration),
                Position::position(200.0, -10.0, -19.0, 0.0, velocity, acceleration),
                Position::position(200.0, -16.0, 0.0, 0.0, velocity, acceleration),
                Position::position(200.0, 0.0, 12.0, 0.0, velocity, acceleration),
            ],
            Shapes::Hexagon => vec![
                Position::position(200.0, 8.0, 10.0, 0.0, velocity, acceleration),
                Position::position(200.0, 17.0, -5.0, 0.0, velocity, acceleration),
                Position::position(200.0, 8.0, -20.0, 0.0, velocity, acceleration),
                Position::position(200.0, -9.0, -20.0, 0.0, velocity, acceleration),
                Position::position(200.0, -17.0, -5.0, 0.0, velocity, acceleration),
                Position::position(200.0, -8.0, 10.0, 0.0, velocity, acceleration),
            ],
        }
    }
    pub fn show_name_en(&self) -> String {
        match self {
            Shapes::Triangle => String::from("Triangle"),
            Shapes::Square   => String::from("Square"),
            Shapes::Pentagon => String::from("Pentagon"),
            Shapes::Hexagon  => String::from("Hexagon"),
        }
    }
    pub fn show_name_symbol(&self) -> String {
        match self {
            Shapes::Triangle => String::from("T・▲・三"),
            Shapes::Square   => String::from("S・■・四"),
            Shapes::Pentagon => String::from("P・⬟・五"),
            Shapes::Hexagon  => String::from("H・⬢・六"),
        }
    }
}

// Redundant, but legacy.
impl std::fmt::Display for Shapes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shapes::Triangle => write!(f, " ▲ 三角形 ▲ "),
            Shapes::Square   => write!(f, " ■ 正方形 ■ "),
            Shapes::Pentagon => write!(f, " ⬟ 五角形 ⬟ "),
            Shapes::Hexagon  => write!(f, " ⬢ 六角形 ⬢ "),
        }
    }
}

/// CONFIG
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub font_path     : String,
    pub results_path  : String,
    pub sequences_path: String,
    pub dobot_path    : String,
    pub relay_path    : String,
}

impl Config {
    pub fn default() -> Self {
        Config {
            dobot_path    : String::from("/dev/ttyUSB0"),
            relay_path    : String::from("/dev/ttyACM0"),
            font_path     : String::from("./fonts"),
            results_path  : String::from("./results"),
            sequences_path: String::from("./sequences"),
        }
    }
}

/// SHAPES EXPERIMENT
#[derive(Clone, Debug, PartialEq)]
pub enum ExperimentStage {
    NotInExperiment,
    BeginTiming,
    Preparation,
    Answering,
}

#[derive(Clone)]
pub struct ExperimentInstance {
    pub shapes_selection: Vec<Shapes>,
    pub list_of_shapes  : Vec<Shapes>,
    pub list_of_guesses : Vec<Shapes>,
    pub list_of_retries : Vec<u32>,
    pub list_of_time    : Vec<u128>,
    pub experiment_index: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultExports {
    pub parameters         : Parameters,
    pub avg_time           : u128,
    pub avg_correct_answers: u32,
    pub analyses           : Vec<ShapeAnalysis>,
    pub results            : Vec<ResultItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultItem {
    pub time       : u128,
    pub true_shape : Shapes,
    pub guess_shape: Shapes,
    pub retries    : u32,
    pub is_correct : bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShapeAnalysis {
    pub time            : u128,
    pub av_time         : u128,
    pub main_shape      : Shapes,
    pub main_shape_count: u32,
    pub wrong_shapes    : Vec<(Shapes, u32)>,
}

impl ShapeAnalysis {
    pub fn new(main_shape: Shapes, used_shapes: Vec<Shapes>) -> Self {
        Self {
            main_shape,
            main_shape_count: 0,
            wrong_shapes: used_shapes
                .iter()
                .filter(|shape| **shape != main_shape)
                .map(|shape| (shape.clone(), 0))
                .collect::<Vec<(Shapes, u32)>>(),
            time: 0,
            av_time: 0,
        }
    }
    pub fn calc_avg_time(&mut self) {
        if self.time == 0 {
            self.av_time = 0;
        } else {
            self.av_time = {
                let totals = self.calc_total_self_shape();
                match totals {
                    0 => 0,
                    _ => self.time / totals as u128,
                }
            }
        }
    }
    pub fn calc_total_self_shape(&self) -> u32 {
        self.wrong_shapes.iter().fold(0, |mut accu, (_, count)| {
            accu += count;
            accu
        }) + self.main_shape_count
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ParameterType {
    SubjectName,
    TestCount,
    Voltage,
    Acceleration,
    Speed,
    ModulationType,
    ModulationFrequency,
    CarrierType,
    CarrierFrequency,
}

impl ParameterType {
    pub fn show_title(&self, no_english: bool) -> String {
        match self {
            ParameterType::SubjectName => String::from({
                if no_english {
                    "被験者名"
                } else {
                    "被験者名・Name"
                }
            }),
            ParameterType::TestCount => String::from({
                if no_english {
                    "実験回数"
                } else {
                    "実験回数・Test Count"
                }
            }),
            ParameterType::Voltage => String::from({
                if no_english {
                    "駆動電圧"
                } else {
                    "振駆電圧・Voltage"
                }
            }),
            ParameterType::Speed => String::from({
                if no_english {
                    "運動速度"
                } else {
                    "運動速度・Speed"
                }
            }),
            ParameterType::Acceleration => String::from({
                if no_english {
                    "運動加速度"
                } else {
                    "運動加速度・Acceleration"
                }
            }),
            ParameterType::ModulationType => String::from({
                if no_english {
                    "変調波種類"
                } else {
                    "変調波種類・Modulation Type"
                }
            }),
            ParameterType::ModulationFrequency => String::from({
                if no_english {
                    "変調波周波数"
                } else {
                    "変調波周波数・Modulation Frequency"
                }
            }),
            ParameterType::CarrierType => String::from({
                if no_english {
                    "搬送波種類"
                } else {
                    "搬送波種類・Carrier Type"
                }
            }),
            ParameterType::CarrierFrequency => String::from({
                if no_english {
                    "搬送波周波数"
                } else {
                    "搬送波周波数・Carrier Frequency"
                }
            }),
        }
    }

    pub fn show_unit(&self) -> Option<String> {
        match self {
            ParameterType::SubjectName         => None,
            ParameterType::TestCount           => None,
            ParameterType::Voltage             => Some(String::from(" [Vpp]")),
            ParameterType::Acceleration        => Some(String::from(" [cm / s^2]")),
            ParameterType::Speed               => Some(String::from(" [cm / s]")),
            ParameterType::ModulationType      => None,
            ParameterType::ModulationFrequency => Some(String::from(" [Hz]")),
            ParameterType::CarrierType         => None,
            ParameterType::CarrierFrequency    => Some(String::from(" [Hz]")),
        }
    }

    pub fn show_title_en(&self) -> String {
        match self {
            ParameterType::SubjectName         => String::from("Subject name"),
            ParameterType::TestCount           => String::from("Test count"),
            ParameterType::Voltage             => String::from("Voltage"),
            ParameterType::Acceleration        => String::from("Acceleration"),
            ParameterType::Speed               => String::from("Speed"),
            ParameterType::ModulationType      => String::from("Modulation type"),
            ParameterType::ModulationFrequency => String::from("Modulation frequency"),
            ParameterType::CarrierType         => String::from("Carrier type"),
            ParameterType::CarrierFrequency    => String::from("Carrier frequency"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParameterItem {
    pub parameter_type: ParameterType,
    pub value         : String,
}

impl ParameterItem {
    fn new(parameter_type: ParameterType) -> Self {
        Self {
            parameter_type,
            value: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameters {
    pub subject_name        : ParameterItem,
    pub test_count          : ParameterItem,
    pub voltage             : ParameterItem,
    pub acceleration        : ParameterItem,
    pub speed               : ParameterItem,
    pub modulation_type     : ParameterItem,
    pub modulation_frequency: ParameterItem,
    pub carrier_type        : ParameterItem,
    pub carrier_frequency   : ParameterItem,
}

impl Parameters {
    pub fn new() -> Self {
        Self {
            subject_name        : ParameterItem::new(ParameterType::SubjectName),
            test_count          : ParameterItem::new(ParameterType::TestCount),
            voltage             : ParameterItem::new(ParameterType::Voltage),
            acceleration        : ParameterItem::new(ParameterType::Acceleration),
            speed               : ParameterItem::new(ParameterType::Speed),
            modulation_type     : ParameterItem::new(ParameterType::ModulationType),
            modulation_frequency: ParameterItem::new(ParameterType::ModulationFrequency),
            carrier_type        : ParameterItem::new(ParameterType::CarrierType),
            carrier_frequency   : ParameterItem::new(ParameterType::CarrierFrequency),
        }
    }

    pub fn into_iter(&self) -> core::array::IntoIter<&ParameterItem, 9> {
        [
            &self.subject_name,
            &self.test_count,
            &self.voltage,
            &self.acceleration,
            &self.speed,
            &self.modulation_type,
            &self.modulation_frequency,
            &self.carrier_type,
            &self.carrier_frequency,
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> std::vec::IntoIter<&mut ParameterItem> {
        vec![
            &mut self.subject_name,
            &mut self.test_count,
            &mut self.voltage,
            &mut self.acceleration,
            &mut self.speed,
            &mut self.modulation_type,
            &mut self.modulation_frequency,
            &mut self.carrier_type,
            &mut self.carrier_frequency,
        ]
        .into_iter()
    }
}
