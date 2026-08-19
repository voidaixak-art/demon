import os
import glob
import time
import requests
from typing import List

NLP_SERVER_URL = "http://localhost:8081/extract"

try:
    from .client import AxiomClient
except ImportError:
    import sys
    sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    from subconscious.client import AxiomClient

class BulkReader:
    def __init__(self, data_dir: str = "data"):
        self.data_dir = data_dir
        self.client = AxiomClient()
        if not os.path.exists(self.data_dir):
            os.makedirs(self.data_dir)
            print(f"📁 Created '{self.data_dir}' directory. Please place PDF or TXT files inside it.")

    def extract_text_from_pdf(self, pdf_path: str) -> str:
        try:
            import fitz  # PyMuPDF
            doc = fitz.open(pdf_path)
            text = ""
            for page in doc:
                text += page.get_text("text") + "\n\n"
            return text
        except ImportError:
            print("⚠️ PyMuPDF not installed. Cannot read PDFs. Run: pip install pymupdf")
            return ""
        except Exception as e:
            print(f"❌ Failed to read PDF {pdf_path}: {e}")
            return ""

    def chunk_text(self, text: str) -> List[str]:
        # Split into paragraphs roughly
        paragraphs = [p.strip() for p in text.split('\n\n') if len(p.strip()) > 50]
        return paragraphs

    def process_central_nlp(self, paragraph: str) -> List[dict]:
        try:
            response = requests.post(NLP_SERVER_URL, json={"text": paragraph}, timeout=10)
            if response.status_code == 200:
                return response.json().get("relations", [])
        except requests.exceptions.RequestException:
            pass
        return []

    def ingest_paragraph(self, paragraph: str, dimension: str):
        relations = self.process_central_nlp(paragraph)
        
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
                    pass # Silent to avoid flooding terminal on bulk

    def run(self, dimension: str = "Knowledge Base", limit_paragraphs: int = 0):
        files = glob.glob(os.path.join(self.data_dir, "*.*"))
        if not files:
            print(f"⚠️ No files found in {self.data_dir}/")
            return

        for file_path in files:
            ext = file_path.lower().split('.')[-1]
            text = ""
            if ext == "pdf":
                print(f"📄 Reading PDF: {os.path.basename(file_path)}")
                text = self.extract_text_from_pdf(file_path)
            elif ext == "txt":
                print(f"📄 Reading TXT: {os.path.basename(file_path)}")
                with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                    text = f.read()
            else:
                continue

            if not text:
                continue

            chunks = self.chunk_text(text)
            print(f"🧩 {os.path.basename(file_path)} chunked into {len(chunks)} paragraphs.")
            
            count = 0
            for chunk in chunks:
                if limit_paragraphs > 0 and count >= limit_paragraphs:
                    break
                self.ingest_paragraph(chunk, dimension)
                count += 1
                if count % 10 == 0:
                    print(f"   Processed {count}/{len(chunks)} paragraphs...")
                time.sleep(0.5)

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Bulk Reader for PDFs/TXTs via NLP Server")
    parser.add_argument("--dir", type=str, default="data", help="Directory containing files")
    parser.add_argument("--dimension", type=str, default="Documents", help="Dimension name")
    parser.add_argument("--limit", type=int, default=0, help="Max paragraphs per file to process (0 = infinite)")
    
    args = parser.parse_args()
    reader = BulkReader(data_dir=args.dir)
    reader.run(dimension=args.dimension, limit_paragraphs=args.limit)

if __name__ == "__main__":
    main()
