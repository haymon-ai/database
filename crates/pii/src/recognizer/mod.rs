//! Recognizer abstraction, entity-type newtype, validator hook, and built-in registry.

mod category;
mod deny_list;
mod keyword_context;
mod severity;
mod types;
mod validators;

pub mod entity;
pub mod pattern;

pub use category::{Category, ParseCategoryError};
pub use deny_list::deny_list_recognizer;
pub use keyword_context::KeywordContextValidator;
pub use pattern::Pattern;
pub use severity::Severity;
pub use types::{EntityType, Recognizer, ValidationOutcome, Validator};
pub use validators::{AndValidator, IbanValidator, IpAddressValidator, LuhnValidator, NoopValidator, UsSsnValidator};
