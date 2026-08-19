import urllib.request
import re
import json
import time
import sys
import requests
from typing import List, Dict, Any

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

from .client import AxiomClient
from .sensory_gemini import GeminiPerception

NLP_SERVER_URL = "http://localhost:8081/extract"

class AutonomousCrawler:
    def __init__(self):
        self.client = AxiomClient()
        self.gemini = GeminiPerception()
        print("🧠 [Crawler] Connecting to Central NLP Server at :8081...")

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
        start_match = re.search(r'\*\*\* START OF TH(?:E|IS) PROJECT GUTENBERG EBOOK.*?\*\*\*', text)
        end_match = re.search(r'\*\*\* END OF TH(?:E|IS) PROJECT GUTENBERG EBOOK.*?\*\*\*', text)
        
        if start_match:
            text = text[start_match.end():]
        if end_match:
            text = text[:end_match.start()]

        paragraphs = [p.strip() for p in re.split(r'\n\s*\n', text) if len(p.strip()) > 50]
        return paragraphs

    def process_central_nlp(self, paragraph: str) -> List[Dict[str, str]]:
        try:
            response = requests.post(NLP_SERVER_URL, json={"text": paragraph}, timeout=10)
            if response.status_code == 200:
                return response.json().get("relations", [])
        except requests.exceptions.RequestException:
            pass
        return []

    def ingest_paragraph(self, paragraph: str, dimension: str):
        print(f"\n📖 [READING] {paragraph[:100]}...")
        
        relations = []
        
        # Ask the Central NLP Server (100% Offline, Zero RAM cost on Crawler!)
        relations = self.process_central_nlp(paragraph)
        
        # EXTREME FALLBACK: Only if NLP server fails AND text is substantial
        if not relations and len(paragraph.split()) > 15:
            print(f"⚠️ [System 1 Fail] Central NLP found nothing. Falling back to Gemini as last resort...")
            prompt = f"""
            Analyze the following text ontologically. Extract 1 or 2 absolute causal or logical truths.
            Return ONLY a valid JSON array of objects with 'source', 'target', and 'relation' (e.g. 'Causes', 'Implies', 'Produces').
            Text: "{paragraph}"
            """
            result = self.gemini._call_gemini_raw(prompt)
            if isinstance(result, list):
                for r in result:
                    if "source" in r and "target" in r and "relation" in r:
                        r["confidence"] = "0.95"
                        relations.append(r)

        # Inject into Rust Axiom Engine
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
