//! Core PII types: category tag, entity-type newtype, validation outcome, built-in entity constants.

pub mod category;
pub mod entity;
pub mod entity_type;
pub mod validation;

pub use category::{Category, ParseCategoryError};
pub use entity_type::EntityType;
pub use validation::ValidationOutcome;
