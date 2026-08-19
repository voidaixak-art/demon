use serde::{Deserialize, Serialize, Deserializer, Serializer};
use serde::de::{self, Visitor};
use std::fmt;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Relation {
    Implies,
    Proves,
    Contradicts,
    Causes,
    TransformsInto,
    Equivalent,
    IsA,
    Evolved_Vector_Link,
    TemporalBefore,
    TemporalAfter,
    Mapped_Via,
    Custom(String),
}

impl Relation {
    pub fn as_str(&self) -> String {
        match self {
            Relation::Implies => "Implies".to_string(),
            Relation::Proves => "Proves".to_string(),
            Relation::Contradicts => "Contradicts".to_string(),
            Relation::Causes => "Causes".to_string(),
            Relation::TransformsInto => "TransformsInto".to_string(),
            Relation::Equivalent => "Equivalent".to_string(),
            Relation::IsA => "IsA".to_string(),
            Relation::Evolved_Vector_Link => "Evolved_Vector_Link".to_string(),
            Relation::TemporalBefore => "TemporalBefore".to_string(),
            Relation::TemporalAfter => "TemporalAfter".to_string(),
            Relation::Mapped_Via => "Mapped_Via".to_string(),
            Relation::Custom(s) => s.clone(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "implies" => Relation::Implies,
            "proves" => Relation::Proves,
            "contradicts" => Relation::Contradicts,
            "causes" => Relation::Causes,
            "transformsinto" | "transforms_into" => Relation::TransformsInto,
            "equivalent" => Relation::Equivalent,
            "isa" | "is_a" => Relation::IsA,
            "evolved_vector_link" | "vector_link" => Relation::Evolved_Vector_Link,
            "temporalbefore" | "temporal_before" | "before" => Relation::TemporalBefore,
            "temporalafter" | "temporal_after" | "after" => Relation::TemporalAfter,
            "mapped_via" => Relation::Mapped_Via,
            _ => Relation::Custom(s.trim().to_string()),
        }
    }

    pub fn is_traversable(&self) -> bool {
        match self {
            Relation::Contradicts => false,
            _ => true,
        }
    }

    pub fn symbol(&self) -> String {
        match self {
            Relation::Implies => "⟹".to_string(),
            Relation::Proves => "∎ (Proves)".to_string(),
            Relation::Contradicts => "⟂ (Contradicts)".to_string(),
            Relation::Causes => "➔ (Causes)".to_string(),
            Relation::TransformsInto => "⟿ (Transforms)".to_string(),
            Relation::Equivalent => "≡ (Equivalent)".to_string(),
            Relation::IsA => "∈ (IsA)".to_string(),
            Relation::Evolved_Vector_Link => "⚡ (Synapse)".to_string(),
            Relation::TemporalBefore => "⏳ (Before)".to_string(),
            Relation::TemporalAfter => "⌛ (After)".to_string(),
            Relation::Mapped_Via => "🌀 (Functor Map)".to_string(),
            Relation::Custom(s) => format!("~ ({})", s),
        }
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for Relation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_str())
    }
}

struct RelationVisitor;

impl<'de> Visitor<'de> for RelationVisitor {
    type Value = Relation;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing a Relation")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Relation::from_str(value))
    }
}

impl<'de> Deserialize<'de> for Relation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(RelationVisitor)
    }
}
