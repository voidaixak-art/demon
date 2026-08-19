use serde::{Deserialize, Serialize};
use std::fmt;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Relation {
    Implies,
    Contradicts,
    Causes,
    TransformsInto,
    Equivalent,
    IsA,
    Evolved_Vector_Link,
}

impl Relation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Relation::Implies => "Implies",
            Relation::Contradicts => "Contradicts",
            Relation::Causes => "Causes",
            Relation::TransformsInto => "TransformsInto",
            Relation::Equivalent => "Equivalent",
            Relation::IsA => "IsA",
            Relation::Evolved_Vector_Link => "Evolved_Vector_Link",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "implies" => Relation::Implies,
            "contradicts" => Relation::Contradicts,
            "causes" => Relation::Causes,
            "transformsinto" | "transforms_into" => Relation::TransformsInto,
            "equivalent" => Relation::Equivalent,
            "isa" | "is_a" => Relation::IsA,
            "evolved_vector_link" | "vector_link" => Relation::Evolved_Vector_Link,
            _ => Relation::Implies,
        }
    }

    pub fn is_traversable(&self) -> bool {
        matches!(
            self,
            Relation::Implies
                | Relation::Causes
                | Relation::TransformsInto
                | Relation::Equivalent
                | Relation::IsA
                | Relation::Evolved_Vector_Link
        )
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Relation::Implies => "⟹",
            Relation::Contradicts => "⟂ (Contradicts)",
            Relation::Causes => "➔ (Causes)",
            Relation::TransformsInto => "⟿ (Transforms)",
            Relation::Equivalent => "≡ (Equivalent)",
            Relation::IsA => "∈ (IsA)",
            Relation::Evolved_Vector_Link => "⚡ (Synapse)",
        }
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
