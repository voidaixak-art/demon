import json
from typing import Dict, Any, List
from .client import AxiomClient
from .sensory_gemini import GeminiPerception

class ScholarEngine:
    def __init__(self, client: AxiomClient, perception: GeminiPerception):
        self.client = client
        self.perception = perception

    def deep_learn_subject(self, subject: str) -> Dict[str, Any]:
        """Scans an entire domain/subject and extracts its axiomatic foundation into the Rust graph."""
        print(f"\n📚 [SCHOLAR] Ingesting & mapping domain: [{subject}]...")
        self.client.log_thought("SCHOLAR", f"Deep studying subject: [{subject}]")

        prompt = f"""You are the Scholar Cortex of the Axiom Engine.
Break down the fundamental axiomatic structure of the domain/subject: "{subject}".
Provide:
1. Exactly 4 core foundational concepts in this domain.
2. The causal / implication relationships between these concepts.

Strictly output valid JSON:
{{
  "concepts": [
    {{"concept": "Concept 1", "dimension": "{subject}"}},
    {{"concept": "Concept 2", "dimension": "{subject}"}},
    {{"concept": "Concept 3", "dimension": "{subject}"}},
    {{"concept": "Concept 4", "dimension": "{subject}"}}
  ],
  "links": [
    {{"source_concept": "Concept 1", "target_concept": "Concept 2", "relation": "Implies|Causes|IsA|TransformsInto", "confidence": 1.0}},
    {{"source_concept": "Concept 2", "target_concept": "Concept 3", "relation": "Implies|Causes|IsA|TransformsInto", "confidence": 0.95}},
    {{"source_concept": "Concept 3", "target_concept": "Concept 4", "relation": "Implies|Causes|IsA|TransformsInto", "confidence": 0.90}}
  ]
}}"""

        data = self.perception._call_gemini_raw(prompt)
        if not data or "concepts" not in data:
            print(f"❌ [SCHOLAR] Failed to extract structure for {subject}.")
            return {"success": False}

        concept_to_id = {}
        added_count = 0
        for c in data.get("concepts", []):
            name = c.get("concept")
            dim = c.get("dimension", subject)
            if name:
                res = self.client.add_node(name, dim)
                concept_to_id[name.lower().strip()] = res["id"]
                added_count += 1
                print(f"   🌱 Etched Axiom: [#{res['id']}] {name} ({dim})")

        link_count = 0
        for l in data.get("links", []):
            s_name = l.get("source_concept", "").lower().strip()
            t_name = l.get("target_concept", "").lower().strip()
            rel = l.get("relation", "Implies")
            conf = float(l.get("confidence", 1.0))

            if s_name in concept_to_id and t_name in concept_to_id:
                s_id = concept_to_id[s_name]
                t_id = concept_to_id[t_name]
                if self.client.add_link(s_id, t_id, rel, conf):
                    link_count += 1
                    print(f"   🔗 Synapse Formed: [{l.get('source_concept')}] ---({rel})---> [{l.get('target_concept')}]")

        self.client.log_thought(
            "SCHOLAR",
            f"Successfully integrated subject [{subject}]: +{added_count} axioms, +{link_count} synapses."
        )

        return {
            "success": True,
            "subject": subject,
            "nodes_added": added_count,
            "links_added": link_count
        }
