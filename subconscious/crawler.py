import urllib.request
import re
import json
import time
import sys
from typing import List, Dict, Any

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

from .client import AxiomClient
from .sensory_gemini import GeminiPerception

class AutonomousCrawler:
    def __init__(self):
        self.client = AxiomClient()
        self.gemini = GeminiPerception()
        
        print("🧠 [Crawler] Loading spaCy offline NLP model (System 1)...")
        try:
            import spacy
            self.nlp = spacy.load("en_core_web_sm")
            self.has_spacy = True
            print("✅ [Crawler] spaCy loaded successfully. Ready for offline deep reading.")
        except ImportError:
            self.has_spacy = False
            print("⚠️ [Crawler] spaCy not found. Will rely heavily on Gemini API (Watch out for quotas).")
            print("   -> Run: pip install spacy && python -m spacy download en_core_web_sm")
            
        print("🧠 [Crawler] Loading NLTK WordNet (Offline Dictionary)...")
        try:
            import nltk
            from nltk.corpus import wordnet as wn
            nltk.download('wordnet', quiet=True)
            self.has_nltk = True
            self.wn = wn
            print("✅ [Crawler] NLTK WordNet loaded for offline synonym expansion.")
        except ImportError:
            self.has_nltk = False
            print("⚠️ [Crawler] NLTK not found. Run: pip install nltk")

    def fetch_gutenberg_book(self, book_id: int) -> str:
        url = f"https://www.gutenberg.org/files/{book_id}/{book_id}-0.txt"
        print(f"📡 [Crawler] Fetching book #{book_id} from Project Gutenberg...")
        try:
            req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
            with urllib.request.urlopen(req) as response:
                text = response.read().decode('utf-8', errors='ignore')
                print(f"✅ [Crawler] Fetched {len(text)} characters.")
                return text
        except Exception as e:
            print(f"❌ [Crawler] Failed to fetch book: {e}")
            return ""

    def chunk_text(self, text: str, chunk_size: int = 500) -> List[str]:
        # Remove Project Gutenberg header/footer
        start_match = re.search(r'\*\*\* START OF TH(?:E|IS) PROJECT GUTENBERG EBOOK.*?\*\*\*', text)
        end_match = re.search(r'\*\*\* END OF TH(?:E|IS) PROJECT GUTENBERG EBOOK.*?\*\*\*', text)
        
        if start_match:
            text = text[start_match.end():]
        if end_match:
            text = text[:end_match.start()]

        paragraphs = [p.strip() for p in re.split(r'\n\s*\n', text) if len(p.strip()) > 50]
        return paragraphs

    def process_offline_spacy(self, paragraph: str) -> List[Dict[str, str]]:
        if not self.has_spacy:
            return []
        
        doc = self.nlp(paragraph)
        extracted = []
        
        # 1. Advanced Subject-Verb-Object (SVO) extraction
        for token in doc:
            if token.pos_ == "VERB" and not token.is_stop:
                subj = None
                obj = None
                for child in token.children:
                    if child.dep_ in ("nsubj", "nsubjpass"):
                        subj = child.lemma_.title()
                    elif child.dep_ in ("dobj", "pobj", "attr", "acomp", "prep"):
                        obj = child.lemma_.title()
                
                if subj and obj and len(subj) > 2 and len(obj) > 2:
                    extracted.append({
                        "source": subj,
                        "target": obj,
                        "relation": token.lemma_.capitalize(),
                        "confidence": 0.85
                    })
                    
        # 2. Adjective-Noun pairs (e.g., "Good Citizen" -> [Citizen] --Is--> [Good])
        for token in doc:
            if token.pos_ == "ADJ" and token.dep_ == "amod":
                head_noun = token.head.lemma_.title()
                adj = token.lemma_.title()
                if len(head_noun) > 2 and len(adj) > 2:
                    extracted.append({
                        "source": head_noun,
                        "target": adj,
                        "relation": "Is",
                        "confidence": 0.90
                    })
                    
        # 3. Noun Compounds (e.g., "Knowledge Graph" -> [Graph] --Relates_To--> [Knowledge])
        for token in doc:
            if token.dep_ == "compound" and token.head.pos_ in ("NOUN", "PROPN"):
                modifier = token.lemma_.title()
                head_noun = token.head.lemma_.title()
                if len(modifier) > 2 and len(head_noun) > 2:
                    extracted.append({
                        "source": head_noun,
                        "target": modifier,
                        "relation": "Relates_To",
                        "confidence": 0.80
                    })
                    
        # Filter duplicates
        unique_extracted = { (e['source'], e['target'], e['relation']): e for e in extracted }
        return list(unique_extracted.values())

    def enrich_with_wordnet(self, relations: List[Dict[str, str]]) -> List[Dict[str, str]]:
        if not self.has_nltk:
            return relations
            
        enriched = list(relations)
        concepts = set()
        for r in relations:
            concepts.add(r['source'].lower())
            concepts.add(r['target'].lower())
            
        for concept in concepts:
            synsets = self.wn.synsets(concept)
            if not synsets:
                continue
            
            # Take the most common synonym and antonym
            syn_added = False
            ant_added = False
            
            for syn in synsets:
                for lemma in syn.lemmas():
                    syn_name = lemma.name().replace('_', ' ').title()
                    if syn_name.lower() != concept and not syn_added:
                        enriched.append({
                            "source": concept.title(),
                            "target": syn_name,
                            "relation": "Synonym",
                            "confidence": 0.95
                        })
                        syn_added = True
                        
                    if lemma.antonyms() and not ant_added:
                        ant_name = lemma.antonyms()[0].name().replace('_', ' ').title()
                        enriched.append({
                            "source": concept.title(),
                            "target": ant_name,
                            "relation": "Antonym",
                            "confidence": 0.95
                        })
                        ant_added = True
                
                if syn_added and ant_added:
                    break
                    
        return enriched

    def ingest_paragraph(self, paragraph: str, dimension: str):
        print(f"\n📖 [READING] {paragraph[:100]}...")
        
        relations = []
        
        # 1. Try offline spaCy first (Massively upgraded SVO/Noun/Adj rules)
        if self.has_spacy:
            relations = self.process_offline_spacy(paragraph)
        
        # 2. EXTREME FALLBACK: Only if spaCy completely fails AND text is substantial
        if not relations and len(paragraph.split()) > 15:
            print(f"⚠️ [System 1 Fail] spaCy found nothing. Falling back to Gemini as last resort...")
            prompt = f"""
            Analyze the following text ontologically. Extract 1 or 2 absolute causal or logical truths.
            Return ONLY a valid JSON array of objects with 'source', 'target', and 'relation' (e.g. 'Causes', 'Implies', 'Produces').
            Text: "{paragraph}"
            """
            result = self.gemini._call_gemini_raw(prompt)
            if isinstance(result, list):
                for r in result:
                    if "source" in r and "target" in r and "relation" in r:
                        r["confidence"] = 0.95
                        relations.append(r)

        # 3. Inject into Rust Axiom Engine
        # Enrich with dictionary logic first
        relations = self.enrich_with_wordnet(relations)
        
        for rel in relations:
            s_name = str(rel["source"])[:50]
            t_name = str(rel["target"])[:50]
            r_type = str(rel["relation"])[:20]
            conf = float(rel.get("confidence", 0.9))
            
            s_res = self.client.add_node(s_name, dimension)
            t_res = self.client.add_node(t_name, dimension)
            
            if s_res and t_res:
                s_id = s_res["id"]
                t_id = t_res["id"]
                if self.client.add_link(s_id, t_id, r_type, conf):
                    print(f"   🔗 Synapse Wired: [{s_name}] ===({r_type})===> [{t_name}]")

    def run_book_ingestion(self, book_id: int, dimension: str, limit: int = 20):
        text = self.fetch_gutenberg_book(book_id)
        if not text:
            return

        chunks = self.chunk_text(text)
        print(f"🧩 [Crawler] Chunked into {len(chunks)} paragraphs. Processing first {limit}...")

        for i, chunk in enumerate(chunks[:limit]):
            self.ingest_paragraph(chunk, dimension)
            time.sleep(1) # Breath

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Autonomous Knowledge Crawler")
    parser.add_argument("--gutenberg", type=int, help="Project Gutenberg Book ID (e.g. 1497 for Plato's Republic)", required=True)
    parser.add_argument("--dimension", type=str, default="Human Literature", help="Dimension name")
    parser.add_argument("--limit", type=int, default=50, help="Max paragraphs to read")
    
    args = parser.parse_args()
    
    crawler = AutonomousCrawler()
    crawler.run_book_ingestion(args.gutenberg, args.dimension, args.limit)

if __name__ == "__main__":
    main()
