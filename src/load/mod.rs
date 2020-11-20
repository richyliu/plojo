mod dictionary;

pub use dictionary::load as load_dictionary;
pub use dictionary::ParseError;

#[cfg(test)]
pub use crate::testing_resources::testing_dict;
