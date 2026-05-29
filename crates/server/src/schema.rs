//! JSON-schema generators for MCP tool input and output schemas.
//!
//! These helpers replace the default [`rmcp::handler::server::router::tool::ToolBase`]
//! schema generators with versions that strip the four metadata fields no MCP
//! client consumes — root `$schema`, root `title`, root `description`, and the
//! `$defs` / `$ref` indirection — while preserving every keyword the model and
//! clients actually use. Each backend's `ToolBase` impls override
//! [`input_schema`] / [`output_schema`] to call into this module so the input
//! and output shapes are established at schema-generation time, not by
//! post-processing the `tools/list` response.
//!
//! [`input_schema`]: rmcp::handler::server::router::tool::ToolBase::input_schema
//! [`output_schema`]: rmcp::handler::server::router::tool::ToolBase::output_schema

use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use rmcp::model::JsonObject;
use schemars::JsonSchema;
use schemars::Schema;
use schemars::generate::SchemaSettings;
use schemars::transform::Transform;
use serde_json::Value;

thread_local! {
    static INPUT_SCHEMA_CACHE: RwLock<HashMap<TypeId, Arc<JsonObject>>> = RwLock::new(HashMap::new());
    static OUTPUT_SCHEMA_CACHE: RwLock<HashMap<TypeId, Arc<JsonObject>>> = RwLock::new(HashMap::new());
}

/// Returns the input JSON schema for the tool parameter type `T`.
///
/// Strips root `$schema`, `title`, and `description`, and inlines every
/// `$defs` / `$ref` indirection. Results are cached per [`TypeId`] in a
/// thread-local map, so repeated calls for the same `T` return the same
/// [`Arc`].
///
/// # Panics
///
/// Panics if `T`'s `JsonSchema` impl produces a non-object JSON value, which
/// would indicate a broken derive.
#[must_use]
pub fn input_schema<T: JsonSchema + Any>() -> Arc<JsonObject> {
    INPUT_SCHEMA_CACHE.with(|cache| {
        if let Some(cached) = cache
            .read()
            .expect("input schema cache poisoned")
            .get(&TypeId::of::<T>())
        {
            return cached.clone();
        }
        let schema = Arc::new(build::<T>());
        cache
            .write()
            .expect("input schema cache poisoned")
            .insert(TypeId::of::<T>(), schema.clone());
        schema
    })
}

/// Returns the output JSON schema for the tool result type `T`.
///
/// Same generation pipeline as [`input_schema`], plus an MCP-spec invariant:
/// the schema's root `type` must be `"object"`. Results are cached per
/// [`TypeId`].
///
/// # Panics
///
/// Panics when `T`'s schema does not have `"type": "object"` at the root, or
/// when `T`'s `JsonSchema` impl produces a non-object JSON value.
#[must_use]
pub fn output_schema<T: JsonSchema + Any>() -> Arc<JsonObject> {
    OUTPUT_SCHEMA_CACHE.with(|cache| {
        if let Some(cached) = cache
            .read()
            .expect("output schema cache poisoned")
            .get(&TypeId::of::<T>())
        {
            return cached.clone();
        }
        let object = build::<T>();
        match object.get("type") {
            Some(Value::String(t)) if t == "object" => {}
            other => panic!(
                "Invalid output schema for type `{}`: root `type` must be \"object\", got {:?}",
                type_name::<T>(),
                other
            ),
        }
        let schema = Arc::new(object);
        cache
            .write()
            .expect("output schema cache poisoned")
            .insert(TypeId::of::<T>(), schema.clone());
        schema
    })
}

/// Builds the JSON schema object for `T` via schemars draft-2020-12 with
/// `inline_subschemas = true` and the [`StripRootMetadata`] transform that
/// drops the root `$schema`, `title`, and `description` keys.
fn build<T: JsonSchema>() -> JsonObject {
    let schema = SchemaSettings::draft2020_12()
        .with(|s| s.inline_subschemas = true)
        .with_transform(StripRootMetadata)
        .into_generator()
        .into_root_schema_for::<T>();

    let value = serde_json::to_value(schema).expect("schema serialises to JSON");
    let Value::Object(object) = value else {
        panic!("schema for `{}` did not produce a JSON object", type_name::<T>());
    };
    object
}

/// Schemars [`Transform`] that drops root-only metadata keys MCP clients ignore.
#[derive(Clone)]
struct StripRootMetadata;

impl Transform for StripRootMetadata {
    fn transform(&mut self, schema: &mut Schema) {
        if let Some(object) = schema.as_object_mut() {
            object.remove("$schema");
            object.remove("title");
            object.remove("description");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build, input_schema, output_schema};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    /// Outer fixture struct whose schemars docs we expect to be stripped.
    #[derive(Deserialize, Serialize, JsonSchema)]
    #[schemars(title = "FixtureTitle", description = "Fixture root description.")]
    struct Fixture {
        /// Doc-comment kept on the property — must survive schema generation.
        name: String,
        nested: Nested,
    }

    #[derive(Deserialize, Serialize, JsonSchema)]
    struct Nested {
        value: u32,
    }

    /// Distinct fixture to confirm the cache keys by type, not by name.
    #[derive(Deserialize, Serialize, JsonSchema)]
    struct OtherFixture {
        value: u32,
    }

    fn contains_key(value: &Value, key: &str) -> bool {
        match value {
            Value::Object(map) => map.contains_key(key) || map.values().any(|v| contains_key(v, key)),
            Value::Array(items) => items.iter().any(|v| contains_key(v, key)),
            _ => false,
        }
    }

    #[test]
    fn input_schema_strips_dollar_schema_title_and_description() {
        let schema = input_schema::<Fixture>();
        assert!(!schema.contains_key("$schema"), "root $schema not stripped: {schema:?}");
        assert!(!schema.contains_key("title"), "root title not stripped: {schema:?}");
        assert!(
            !schema.contains_key("description"),
            "root description not stripped: {schema:?}"
        );
        assert_eq!(schema.get("type"), Some(&Value::String("object".into())));
    }

    #[test]
    fn input_schema_inlines_nested_subschemas() {
        let schema = input_schema::<Fixture>();
        let value = Value::Object((*schema).clone());
        assert!(!contains_key(&value, "$defs"), "$defs not inlined: {value}");
        assert!(!contains_key(&value, "$ref"), "$ref not inlined: {value}");
    }

    #[test]
    fn input_schema_caches_by_type() {
        let first = input_schema::<Fixture>();
        let second = input_schema::<Fixture>();
        assert!(
            std::sync::Arc::ptr_eq(&first, &second),
            "same type should return cached Arc"
        );
        let other = input_schema::<OtherFixture>();
        assert!(
            !std::sync::Arc::ptr_eq(&first, &other),
            "different types must not share cache entry"
        );
    }

    #[test]
    fn output_schema_accepts_object_root() {
        let schema = output_schema::<Fixture>();
        assert_eq!(schema.get("type"), Some(&Value::String("object".into())));
        let again = output_schema::<Fixture>();
        assert!(std::sync::Arc::ptr_eq(&schema, &again));
    }

    #[test]
    #[should_panic(expected = "root `type` must be \"object\"")]
    fn output_schema_panics_on_non_object_root() {
        let _ = output_schema::<u32>();
    }

    #[test]
    fn build_preserves_properties() {
        let schema = build::<Fixture>();
        let properties = schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties survive generation");
        assert!(properties.contains_key("name"));
        assert!(properties.contains_key("nested"));
        let name = properties.get("name").and_then(Value::as_object).unwrap();
        assert_eq!(
            name.get("description").and_then(Value::as_str),
            Some("Doc-comment kept on the property — must survive schema generation."),
            "per-property descriptions must survive schema generation"
        );
    }
}
