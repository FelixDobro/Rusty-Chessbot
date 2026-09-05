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
    RFP_BIAS: i16 = 47, UciOptionType::new_spin(47, 0, 300), 1;
    RFP_LINEAR: i16 = 28, UciOptionType::new_spin(28, 0, 300), 1;
    RFP_QUADRATIC: i16 = 3, UciOptionType::new_spin(3, 0, 300), 1;
    // Futility Pruning
    FUTILITY_BIAS: i16 = 53, UciOptionType::new_spin(53, 0, 400), 1;
    FUTILITY_LINEAR: i16 = 59, UciOptionType::new_spin(59, 0, 300), 1;
    FUTILITY_QUADRATIC: i16 = 62, UciOptionType::new_spin(62, 0, 300), 1;
    FUTILITY_DEPTH: u8 = 4, UciOptionType::new_spin(4, 0, 10), 1;
    // Prob Cut
    PROB_MIN_DEPTH: u8 = 5, UciOptionType::new_spin(5, 0, 15), 1;
    PROB_DEPTH_REDUCTION: u8 = 6, UciOptionType::new_spin(6, 1, 15), 1;
    PROB_BIAS: i16 = 32, UciOptionType::new_spin(32, 0, 300), 1;
    PROB_LINEAR: i16 = 3, UciOptionType::new_spin(3, 0, 200), 1;
    PROB_QUADRATIC: i16 = 12, UciOptionType::new_spin(12, 0, 100), 1;
    // Razoring
    RAZORING_MAX_DEPTH: u8 = 4, UciOptionType::new_spin(4, 0, 10), 1;
    RAZORING_BIAS: i16 = 383, UciOptionType::new_spin(383, 0, 600), 1;
    RAZORING_LINEAR: i16 = 146, UciOptionType::new_spin(146, 0, 400), 1;
    RAZORING_QUADRATIC: i16 = 36, UciOptionType::new_spin(36, 0, 150), 1;

    // LMR
    LMR_MIN_DEPTH: u8 = 1, UciOptionType::new_spin(1, 0, 10), 1;
    LMR_NUM_MOVES_PLAYED: usize = 5, UciOptionType::new_spin(5, 0, 20), 1;
    LMR_FACTOR: f64 = 1.51, UciOptionType::new_spin(151, 100, 600), 100;

    // Late Move Pruning
    LMP_MAX_DEPTH: u8 = 11, UciOptionType::new_spin(11, 0, 15), 1;
    LMP_BIAS: usize = 0, UciOptionType::new_spin(0, 0, 15), 1;
    LMP_QUADRATIC: usize = 3, UciOptionType::new_spin(3, 0, 15), 1;
    LMP_DIV: usize = 4, UciOptionType::new_spin(4, 1, 15), 1;

    // Extensions
    CHECK_EXTENSION_MAX_PLY: usize = 13, UciOptionType::new_spin(13, 0, 63), 1;

    // Quiesence Search
    DELTA_MARGIN: i16 = 177, UciOptionType::new_spin(177, 100, 275), 1;

    // Aspiration windows
    FIRST_WINDOW: i16 = 27, UciOptionType::new_spin(27, 5, 50), 1;
    SECOND_WINDOW: i16 = 88, UciOptionType::new_spin(88, 80, 150), 1;

    // HISTORY_UPDATES
    CAPTURE_PUNISH_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 2), 1;
    CAPTURE_PUNISH_LINEAR: i32 = 46, UciOptionType::new_spin(46, 0, 200), 1;
    CAPTURE_REWARD_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 2), 1;
    CAPTURE_REWARD_LINEAR: i32 = 33, UciOptionType::new_spin(33, 0, 200), 1;

    MAIN_PUNISH_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 2), 1;
    MAIN_PUNISH_LINEAR: i32 = 42, UciOptionType::new_spin(42, 0, 200), 1;
    MAIN_REWARD_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 2), 1;
    MAIN_REWARD_LINEAR: i32 = 52, UciOptionType::new_spin(52, 0, 200), 1;

    CONTINUATION_PUNISH_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 2), 1;
    CONTINUATION_PUNISH_LINEAR: i32 = 119, UciOptionType::new_spin(119, 0, 200), 1;
    CONTINUATION_REWARD_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 2), 1;
    CONTINUATION_REWARD_LINEAR: i32 = 37, UciOptionType::new_spin(37, 0, 200), 1;

    // History reduction thresholds
    CAPTURE_HISTORY_EXTENSION_THRESHOLD: i32 = 473, UciOptionType::new_spin(473, 0, 2isize.pow(13) - 1), 1;
    HISTORY_EXTENSION_SCORE: i32 = 1853, UciOptionType::new_spin(1853, 0, 2isize.pow(13) - 1), 1;

    // SEE Pruning
    SEE_QUIET: i32 = 118272, UciOptionType::new_spin(118272, 0, 2isize.pow(18)), 1;
    SEE_CAPTURE: i32 = 107598, UciOptionType::new_spin(107598, 0, 2isize.pow(18)), 1;
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
