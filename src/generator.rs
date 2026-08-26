use crate::engine::AxiomEngine;

pub struct LanguageGenerator;

impl LanguageGenerator {
    /// Converts a raw mathematical path of node IDs back into an English sentence
    pub fn generate_sentence(engine: &AxiomEngine, path: &[u64]) -> String {
        let graph = engine.graph.read().unwrap();
        let mut sentence_parts = Vec::new();

        for (i, &node_id) in path.iter().enumerate() {
            if let Some(node) = graph.get(&node_id) {
                let mut word = node.concept.clone();

                if i == 0 {
                    // Subject: Capitalize and prefix
                    word = Self::capitalize(&word);
                    sentence_parts.push(word);
                } else if i == path.len() - 1 {
                    // Object: End of sentence
                    sentence_parts.push(word);
                } else {
                    // Bridge (Verb representation for now)
                    sentence_parts.push(format!("is connected to {}", word));
                }
            }
        }

        let mut final_sentence = sentence_parts.join(" ");
        if !final_sentence.ends_with('.') {
            final_sentence.push('.');
        }

        final_sentence
    }

    fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}
