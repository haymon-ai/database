//! Core PII types: category tag, entity-type enum, validation outcome.

pub mod category;
pub mod entity;
pub mod validation;

pub use category::{Category, ParseCategoryError};
pub use entity::{Entity, ParseEntityError};
pub use validation::ValidationOutcome;
