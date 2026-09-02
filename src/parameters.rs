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
    RFP_BIAS: i16 = 1, UciOptionType::new_spin(1, 0, 300), 1;
    RFP_LINEAR: i16 = 31, UciOptionType::new_spin(31, 0, 300), 1;
    RFP_QUADRATIC: i16 = 15, UciOptionType::new_spin(15, 0, 300), 1;
    // Futility Pruning
    FUTILITY_BIAS: i16 = 3, UciOptionType::new_spin(3, 0, 400), 1;
    FUTILITY_LINEAR: i16 = 22, UciOptionType::new_spin(22, 0, 300), 1;
    FUTILITY_QUADRATIC: i16 = 3, UciOptionType::new_spin(3, 0, 300), 1;
    FUTILITY_DEPTH: u8 = 5, UciOptionType::new_spin(5, 0, 10), 1;
    // Prob Cut
    PROB_MIN_DEPTH: u8 = 5, UciOptionType::new_spin(5, 0, 15), 1;
    PROB_DEPTH_REDUCTION: u8 = 5, UciOptionType::new_spin(5, 1, 15), 1;
    PROB_BIAS: i16 = 55, UciOptionType::new_spin(55, 0, 300), 1;
    PROB_LINEAR: i16 = 10, UciOptionType::new_spin(10, 0, 200), 1;
    PROB_QUADRATIC: i16 = 4, UciOptionType::new_spin(4, 0, 100), 1;
    // Razoring
    RAZORING_MAX_DEPTH: u8 = 2, UciOptionType::new_spin(2, 0, 10), 1;
    RAZORING_BIAS: i16 = 414, UciOptionType::new_spin(414, 0, 600), 1;
    RAZORING_LINEAR: i16 = 167, UciOptionType::new_spin(167, 0, 400), 1;
    RAZORING_QUADRATIC: i16 = 49, UciOptionType::new_spin(49, 0, 150), 1;

    // LMR
    LMR_MIN_DEPTH: u8 = 2, UciOptionType::new_spin(2, 0, 10), 1;
    LMR_NUM_MOVES_PLAYED: usize = 11, UciOptionType::new_spin(11, 0, 20), 1;
    LMR_FACTOR: f64 = 1.52, UciOptionType::new_spin(152, 100, 600), 100;

    // Extensions
    CHECK_EXTENSION_MAX_PLY: usize = 38, UciOptionType::new_spin(38, 0, 63), 1;

    // Quiesence Search
    DELTA_MARGIN: i16 = 161, UciOptionType::new_spin(161, 100, 275), 1;

    // Aspiration windows
    FIRST_WINDOW: i16 = 33, UciOptionType::new_spin(33, 5, 50), 1;
    SECOND_WINDOW: i16 = 104, UciOptionType::new_spin(104, 80, 150), 1;

    // HISTORY_UPDATES
    CAPTURE_PUNISH_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 5), 1;
    CAPTURE_PUNISH_LINEAR: i32 = 51, UciOptionType::new_spin(51, 0, 200), 1;
    CAPTURE_REWARD_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 5), 1;
    CAPTURE_REWARD_LINEAR: i32 = 17, UciOptionType::new_spin(17, 0, 200), 1;

    MAIN_PUNISH_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 5), 1;
    MAIN_PUNISH_LINEAR: i32 = 35, UciOptionType::new_spin(35, 0, 200), 1;
    MAIN_REWARD_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 5), 1;
    MAIN_REWARD_LINEAR: i32 = 25, UciOptionType::new_spin(25, 0, 200), 1;

    CONTINUATION_PUNISH_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 5), 1;
    CONTINUATION_PUNISH_LINEAR: i32 = 55, UciOptionType::new_spin(55, 0, 200), 1;
    CONTINUATION_REWARD_QUADRATIC: i32 = 1, UciOptionType::new_spin(1, 0, 5), 1;
    CONTINUATION_REWARD_LINEAR: i32 = 38, UciOptionType::new_spin(38, 0, 200), 1;


    // History reduction thresholds
    CAPTURE_HISTORY_EXTENSION_THRESHOLD: i32 = 3762, UciOptionType::new_spin(3762, 0, 2isize.pow(13) - 1), 1;
    HISTORY_EXTENSION_SCORE: i32 = 1625, UciOptionType::new_spin(1625, 0, 2isize.pow(13) - 1), 1;


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
