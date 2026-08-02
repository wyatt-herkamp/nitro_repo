use nr_badge::Style;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

use super::RepositoryConfigType;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct ProjectConfig {
    #[schemars(title = "Badge Settings")]
    pub badge_settings: BadgeSettings,
    /// Whether to require a semver version for releases
    pub require_semver: bool,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct ProjectConfigType;
impl RepositoryConfigType for ProjectConfigType {
    fn get_type(&self) -> &'static str {
        "project"
    }
    fn get_description(&self) -> super::ConfigDescription {
        super::ConfigDescription {
            name: "Project",
            description: Some("Project settings for the repository"),
            documentation_link: None,
            ..Default::default()
        }
    }
    fn validate_config(
        &self,
        config: serde_json::Value,
    ) -> Result<(), super::RepositoryConfigError> {
        let _config: ProjectConfig = serde_json::from_value(config)?;
        Ok(())
    }
    fn default(&self) -> Result<serde_json::Value, super::RepositoryConfigError> {
        Ok(serde_json::to_value(ProjectConfig::default())?)
    }
    fn schema(&self) -> Option<schemars::Schema> {
        Some(schema_for!(ProjectConfig))
    }
    fn get_type_static() -> &'static str
    where
        Self: Sized,
    {
        "project"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct BadgeSettings {
    pub style: BadgeStyle,
    pub label_color: String,
    pub color: String,
}
impl Default for BadgeSettings {
    fn default() -> Self {
        BadgeSettings {
            style: Default::default(),
            label_color: "#555".parse().unwrap(),
            color: "#33B5E5".parse().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeStyle(pub Style);
impl Default for BadgeStyle {
    fn default() -> Self {
        BadgeStyle(Style::Flat)
    }
}

impl schemars::JsonSchema for BadgeStyle {
    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("BadgeStyle")
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("BadgeStyle")
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // These must match `nr_badge::Style`'s `Display`/`FromStr`, which is what the newtype's
        // `Serialize`/`Deserialize` delegate to. "flatquare" was a typo for "flatsquare", so the
        // schema advertised a style that `Style::from_str` rejects — picking it in the generated
        // form produced a `BadStyleChoice` on save. `every_advertised_badge_style_deserializes`
        // below is what keeps the two in step.
        schemars::json_schema!({
            "type": "string",
            "enum": ["flat", "plastic", "flatsquare"],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The schema is hand-written, so nothing but this stops it drifting from what `BadgeStyle`
    /// will actually accept. It already had: the schema offered "flatquare", which
    /// `Style::from_str` rejects, so choosing that style in the generated form failed on save.
    #[test]
    fn every_advertised_badge_style_deserializes() {
        let schema =
            <BadgeStyle as JsonSchema>::json_schema(&mut schemars::SchemaGenerator::default());
        let advertised = schema
            .as_value()
            .get("enum")
            .and_then(|value| value.as_array())
            .expect("BadgeStyle's schema should advertise an enum")
            .clone();

        assert!(!advertised.is_empty());

        for style in advertised {
            let parsed: Result<BadgeStyle, _> = serde_json::from_value(style.clone());
            assert!(
                parsed.is_ok(),
                "the schema offers {style} but BadgeStyle refuses to deserialize it"
            );
        }
    }

    /// The other direction: a style the config round-trips must still be offered by the schema, or
    /// an existing repository's setting cannot be re-selected in the form.
    #[test]
    fn the_default_badge_style_round_trips() {
        let settings = BadgeSettings::default();
        let encoded = serde_json::to_value(&settings).expect("serializes");
        let decoded: BadgeSettings = serde_json::from_value(encoded).expect("round-trips");
        assert_eq!(decoded.style.0, Style::Flat);
    }
}
