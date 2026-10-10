use std::collections::HashMap;
use std::sync::LazyLock;

#[allow(dead_code)]
pub enum UciOptionType {
    Spin {
        default: isize,
        min: isize,
        max: isize,
    },
    Check {
        default: bool,
    },
    String {
        default: &'static str,
    },
    Button,
}

impl UciOptionType {
    pub fn new_spin(default: isize, min: isize, max: isize) -> Self {
        UciOptionType::Spin { default, min, max }
    }
}

pub struct UciOption {
    pub name: &'static str,
    pub config: UciOptionType,
}

impl UciOption {
    pub fn validate(&self, val: &str) -> Result<(), String> {
        match self.config {
            UciOptionType::Spin { default, min, max } => {
                if let Ok(parsed) = val.parse::<isize>() {
                    if parsed < min || parsed > max {
                        return Err(format!(
                            "Option of type spin not in valid range [{}, {}], set to default {}",
                            min, max, default
                        ));
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }

    pub fn uci(&self) -> String {
        match self.config {
            UciOptionType::Spin { default, min, max } => {
                format!(
                    "option name {} type spin default {} min {} max {}",
                    self.name, default, min, max
                )
            }
            UciOptionType::Button => {
                format!("option name {} type button", self.name)
            }
            UciOptionType::Check { default } => {
                format!("option name {} type default {}", self.name, default)
            }
            UciOptionType::String { default } => {
                format!("option name {} type string default {}", self.name, default)
            }
        }
    }
}

pub static ENGINE_SETTINGS: LazyLock<HashMap<&'static str, UciOption>> = LazyLock::new(|| {
    HashMap::from([(
        "Hash",
        UciOption {
            name: "Hash",
            config: UciOptionType::Spin {
                default: 100,
                min: 5,
                max: 1024 * 1024,
            },
        },
    )])
});

macro_rules! define_tuning_params {
    (
        $( $name:ident : $type:ty = $default:expr, $config:expr, $scale:expr);* $(;)?
    ) => {


        mod tuning_storage {
            $(
                #[cfg(feature = "tuning")]
                pub static mut $name: $type = $default;
            )*
        }


        $(
            #[inline(always)]
            #[allow(non_snake_case)]
            pub fn $name() -> $type {
                #[cfg(not(feature = "tuning"))]
                { $default }

                #[cfg(feature = "tuning")]
                { unsafe { tuning_storage::$name }}
            }
        )*

        #[cfg(feature = "tuning")]
        pub fn get_tuning_options() -> HashMap<&'static str, UciOption> {
            HashMap::from([
                $(
                    (stringify!($name), UciOption {name: stringify!($name), config: $config}),
                )*
            ])
        }


        #[cfg(feature = "tuning")]
        pub fn set_tuning_option(opt_name: &str, val: &str) -> Result<(), String> {
            if let Some(uci_option) = get_tuning_options().get(opt_name) {
                match opt_name {
                    $(
                        stringify!($name) => {
                        if let Ok(parsed) = val.parse::<$type>() {
                            match uci_option.config {
                                UciOptionType::Spin { default, min, max } => {
                                    if parsed < min as $type || parsed > max as $type {
                                        return Err(format!("Option not in valid range [{}, {}], set to default {}", min, max, default));
                                    } else {
                                        unsafe {tuning_storage::$name = parsed / ($scale as $type);}
                                        Ok(())
                                    }
                                }
                                _ => {
                                    unsafe {tuning_storage::$name = parsed;}
                                    Ok(())
                                }
                            }
                        } else {
                            Err("Error while parsing".to_string())
                        }
                    }
                )*,

                 _ => Err("Option not found".to_string()),
                }
            } else {
                Err("Option not found".to_string())
            }
        }

    };
}
define_tuning_params!(
    // Reverse Futility Pruning
    RFP_BIAS: i16 = 21, UciOptionType::new_spin(21, 0, 300), 1;
    RFP_LINEAR: i16 = 11, UciOptionType::new_spin(11, 0, 300), 1;
    RFP_QUADRATIC: i16 = 31, UciOptionType::new_spin(31, 0, 300), 1;

    // Futility Pruning
    FUTILITY_BIAS: i16 = 38, UciOptionType::new_spin(38, 0, 400), 1;
    FUTILITY_LINEAR: i16 = 34, UciOptionType::new_spin(34, 0, 300), 1;
    FUTILITY_QUADRATIC: i16 = 105, UciOptionType::new_spin(105, 0, 300), 1;
    FUTILITY_DEPTH: u8 = 7, UciOptionType::new_spin(7, 0, 10), 1;

    // Prob Cut
    PROB_MIN_DEPTH: u8 = 6, UciOptionType::new_spin(6, 0, 15), 1;
    PROB_DEPTH_REDUCTION: u8 = 13, UciOptionType::new_spin(13, 1, 15), 1;
    PROB_BIAS: i16 = 68, UciOptionType::new_spin(68, 0, 300), 1;
    PROB_LINEAR: i16 = 21, UciOptionType::new_spin(21, 0, 200), 1;
    PROB_QUADRATIC: i16 = 20, UciOptionType::new_spin(20, 0, 100), 1;

    // Razoring
    RAZORING_MAX_DEPTH: u8 = 5, UciOptionType::new_spin(5, 0, 10), 1;
    RAZORING_BIAS: i16 = 339, UciOptionType::new_spin(339, 0, 600), 1;
    RAZORING_LINEAR: i16 = 153, UciOptionType::new_spin(153, 0, 400), 1;
    RAZORING_QUADRATIC: i16 = 31, UciOptionType::new_spin(31, 0, 150), 1;

    // LMR
    LMR_MIN_DEPTH: u8 = 3, UciOptionType::new_spin(3, 0, 10), 1;
    LMR_NUM_MOVES_PLAYED: usize = 3, UciOptionType::new_spin(3, 0, 20), 1;
    LMR_FACTOR: f64 = 3.19, UciOptionType::new_spin(319, 100, 600), 100;

    // LMP
    LMP_MAX_DEPTH: u8 = 13, UciOptionType::new_spin(13, 0, 15), 1;
    LMP_BIAS: usize = 2, UciOptionType::new_spin(2, 0, 15), 1;
    LMP_QUADRATIC: usize = 2, UciOptionType::new_spin(2, 0, 15), 1;
    LMP_DIV: usize = 6, UciOptionType::new_spin(6, 1, 15), 1;

    // Extensions
    CHECK_EXTENSION_MAX_PLY: usize = 20, UciOptionType::new_spin(20, 0, 63), 1;

    // Quiescence Search
    DELTA_MARGIN: i16 = 220, UciOptionType::new_spin(220, 100, 275), 1;

    // Aspiration Windows
    FIRST_WINDOW: i16 = 25, UciOptionType::new_spin(25, 5, 50), 1;
    SECOND_WINDOW: i16 = 91, UciOptionType::new_spin(91, 80, 150), 1;

    // History Updates
    CAPTURE_PUNISH_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 2), 1;
    CAPTURE_PUNISH_LINEAR: i32 = 19, UciOptionType::new_spin(19, 0, 200), 1;
    CAPTURE_REWARD_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 2), 1;
    CAPTURE_REWARD_LINEAR: i32 = 25, UciOptionType::new_spin(25, 0, 200), 1;

    MAIN_PUNISH_QUADRATIC: i32 = 2, UciOptionType::new_spin(2, 0, 2), 1;
    MAIN_PUNISH_LINEAR: i32 = 79, UciOptionType::new_spin(79, 0, 200), 1;
    MAIN_REWARD_QUADRATIC: i32 = 2, UciOptionType::new_spin(2, 0, 2), 1;
    MAIN_REWARD_LINEAR: i32 = 72, UciOptionType::new_spin(72, 0, 200), 1;

    CONTINUATION_PUNISH_QUADRATIC: i32 = 0, UciOptionType::new_spin(0, 0, 2), 1;
    CONTINUATION_PUNISH_LINEAR: i32 = 181, UciOptionType::new_spin(181, 0, 200), 1;
    CONTINUATION_REWARD_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 2), 1;
    CONTINUATION_REWARD_LINEAR: i32 = 51, UciOptionType::new_spin(51, 0, 200), 1;

    // History Reduction Thresholds
    CAPTURE_HISTORY_EXTENSION_THRESHOLD: i32 = 599, UciOptionType::new_spin(599, 0, 2isize.pow(13) - 1), 1;
    HISTORY_EXTENSION_SCORE: i32 = 2088, UciOptionType::new_spin(2088, 0, 2isize.pow(13) - 1), 1;

    // SEE Pruning
    SEE_QUIET: i32 = 12486, UciOptionType::new_spin(12486, 0, 2isize.pow(18)), 1;
    SEE_CAPTURE: i32 = 66402, UciOptionType::new_spin(66402, 0, 2isize.pow(18)), 1;
);

#[cfg(feature = "tuning")]
pub fn print_engine_parameters() {
    get_tuning_options()
        .values()
        .for_each(|val| println!("{}", val.uci()));
}

#[cfg(not(feature = "tuning"))]
pub fn print_engine_parameters() {
    ENGINE_SETTINGS
        .values()
        .for_each(|option| println!("{}", option.uci()));
}
