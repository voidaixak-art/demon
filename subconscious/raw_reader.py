import re
import sys
import os
from typing import Dict, Any, List, Tuple
from .client import AxiomClient

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

class RawKnowledgeReader:
    def __init__(self):
        self.client = AxiomClient()

    def read_text_and_ingest(self, raw_text: str, dimension: str = "Ingested Knowledge") -> Dict[str, Any]:
        """Parses raw text and extracts concepts and causal relationships into the Axiom Graph."""
        print(f"\n📖 [RAW INGESTION] Reading text ({len(raw_text)} chars) under dimension [{dimension}]...")

        # 1. Break into sentences
        sentences = re.split(r'[.\n;]+', raw_text)
        sentences = [s.strip() for s in sentences if len(s.strip()) > 10]

        extracted_concepts = set()
        extracted_links = []

        # Causal and logical keywords mapping
        patterns = [
            (r'(.+?)\s+(?:causes|leads to|results in)\s+(.+)', "Causes"),
            (r'(.+?)\s+(?:implies|necessitates|means that)\s+(.+)', "Implies"),
            (r'(.+?)\s+(?:contradicts|opposes|conflicts with)\s+(.+)', "Contradicts"),
            (r'(.+?)\s+(?:transforms into|evolves into|becomes)\s+(.+)', "TransformsInto"),
            (r'(.+?)\s+(?:is equivalent to|is synonymous with)\s+(.+)', "Equivalent"),
            (r'(.+?)\s+(?:is a type of|is a form of|is a kind of)\s+(.+)', "IsA"),
        ]

        for s in sentences:
            matched_pattern = False
            for pat, rel in patterns:
                m = re.search(pat, s, re.IGNORECASE)
                if m:
                    c1 = m.group(1).strip()
                    c2 = m.group(2).strip()

                    # Clean words
                    c1_clean = self._clean_concept_name(c1)
                    c2_clean = self._clean_concept_name(c2)

                    if c1_clean and c2_clean and len(c1_clean) > 3 and len(c2_clean) > 3:
                        extracted_concepts.add(c1_clean)
                        extracted_concepts.add(c2_clean)
                        extracted_links.append((c1_clean, c2_clean, rel))
                        matched_pattern = True
                        break

            if not matched_pattern:
                # Key noun-phrase heuristic
                words = [w for w in s.split() if len(w) > 4]
                if len(words) >= 2:
                    c = " ".join(words[:3]).title()
                    extracted_concepts.add(c)

        # Ingest nodes into Rust SQLite engine
        concept_to_id = {}
        for c in extracted_concepts:
            res = self.client.add_node(c, dimension)
            concept_to_id[c] = res["id"]
            if res.get("is_new"):
                print(f"   🌱 Extracted Axiom: [#{res['id']}] {c}")

        # Ingest links
        links_wired = 0
        for c1, c2, rel in extracted_links:
            if c1 in concept_to_id and c2 in concept_to_id:
                s_id = concept_to_id[c1]
                t_id = concept_to_id[c2]
                if self.client.add_link(s_id, t_id, rel, 0.95):
                    links_wired += 1
                    print(f"   🔗 Synapse Wired: [{c1}] ===({rel})===> [{c2}]")

        self.client.log_thought(
            "INGEST",
            f"Raw text ingestion complete: +{len(concept_to_id)} concepts, +{links_wired} causal synapses."
        )

        return {
            "concepts_extracted": len(concept_to_id),
            "links_wired": links_wired
        }

    def ingest_file(self, file_path: str, dimension: str = "File Ingestion") -> Dict[str, Any]:
        if not os.path.exists(file_path):
            print(f"❌ File not found: {file_path}")
            return {"error": "File not found"}

        with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
            content = f.read()

        return self.read_text_and_ingest(content, dimension)

    def _clean_concept_name(self, text: str) -> str:
        # Remove common filler phrases
        text = re.sub(r'^(the|a|an|because|therefore|however|in fact)\s+', '', text, flags=re.IGNORECASE)
        # Limit to 5 words max for a clean conceptual node
        words = text.split()[:5]
        return " ".join(words).strip().title()

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Axiom Engine 2.0 Raw Text & Document Ingestor")
    parser.add_argument("--file", type=str, help="Path to text or markdown file to ingest")
    parser.add_argument("--text", type=str, help="Raw text string to ingest directly")
    parser.add_argument("--dimension", type=str, default="Direct Ingestion", help="Category / Dimension name")
    args = parser.parse_args()

    reader = RawKnowledgeReader()

    if args.file:
        reader.ingest_file(args.file, args.dimension)
    elif args.text:
        reader.read_text_and_ingest(args.text, args.dimension)
    else:
        print("Usage: python -m subconscious.raw_reader --file <path> OR --text \"<content>\"")

if __name__ == "__main__":
    main()
