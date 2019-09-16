use std::{
    collections::HashSet,
};

pub const SCRIPT_HASH_LENGTH: usize = 32;

/// Holds the VM configuration, currently this is only the publishing options for scripts and
/// modules, but in the future this may need to be expanded to hold more information.
#[derive(Clone, Debug)]
pub struct VMConfig {
    pub publishing_options: VMPublishingOption,
}

impl Default for VMConfig {
    fn default() -> VMConfig {
        VMConfig {
            publishing_options: VMPublishingOption::Open,
        }
    }
}

/// Defines and holds the publishing policies for the VM. There are three possible configurations:
/// 1. No module publishing, only whitelisted scripts are allowed.
/// 2. No module publishing, custom scripts are allowed.
/// 3. Both module publishing and custom scripts are allowed.
/// We represent these as an enum instead of a struct since whitelisting and module/script
/// publishing are mutually exclusive options.
#[derive(Clone, Debug)]
// #[serde(tag = "type", content = "whitelist")]
pub enum VMPublishingOption {
    /// Only allow scripts on a whitelist to be run
    // #[serde(deserialize_with = "deserialize_whitelist")]
    // #[serde(serialize_with = "serialize_whitelist")]
    Locked(HashSet<[u8; SCRIPT_HASH_LENGTH]>),
    /// Allow custom scripts, but _not_ custom module publishing
    CustomScripts,
    /// Allow both custom scripts and custom module publishing
    Open,
}

impl VMPublishingOption {
    pub fn custom_scripts_only(&self) -> bool {
        !self.is_open() && !self.is_locked()
    }

    pub fn is_open(&self) -> bool {
        match self {
            VMPublishingOption::Open => true,
            _ => false,
        }
    }

    pub fn is_locked(&self) -> bool {
        match self {
            VMPublishingOption::Locked { .. } => true,
            _ => false,
        }
    }

    pub fn get_whitelist_set(&self) -> Option<&HashSet<[u8; SCRIPT_HASH_LENGTH]>> {
        match self {
            VMPublishingOption::Locked(whitelist) => Some(&whitelist),
            _ => None,
        }
    }
}

// impl VMConfig {
//     /// Creates a new `VMConfig` where the whitelist is empty. This should only be used for testing.
//     #[allow(non_snake_case)]
//     #[doc(hidden)]
//     pub fn empty_whitelist_FOR_TESTING() -> Self {
//         VMConfig {
//             publishing_options: VMPublishingOption::Locked(HashSet::new()),
//         }
//     }

//     pub fn save_config<P: AsRef<Path>>(&self, output_file: P) {
//         let contents = toml::to_vec(&self).expect("Error serializing");

//         let mut file = File::create(output_file).expect("Error opening file");

//         file.write_all(&contents).expect("Error writing file");
//     }
// }