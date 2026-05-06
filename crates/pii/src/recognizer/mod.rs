//! Recognizer abstraction, entity-type newtype, validator hook, and built-in registry.

mod category;
mod types;
mod validator;

pub mod entity;
pub mod rule;

pub use category::{Category, ParseCategoryError};
pub use rule::Rule;
pub use types::{EntityType, Recognizer, ValidationOutcome, Validator};
pub use validator::{
    AbaRoutingValidator, AndValidator, EinPrefixValidator, IbanValidator, IpAddressValidator, ItinRangeValidator,
    JwtHeaderValidator, KeywordValidator, LuhnSinValidator, LuhnValidator, Mod11NhsValidator, NinoBlocklistValidator,
    NoopValidator, PrivateKeyTypeValidator, UsSsnValidator, VatCountryLengthValidator,
};
