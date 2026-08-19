import json
import os
import sys
import argparse
from subconscious.client import AxiomClient

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

class LatexTutor:
    def __init__(self):
        self.client = AxiomClient()
        print("🧠 [LaTeX Tutor] Connecting to Axiom Engine...")

    def teach_symbols(self, dict_path: str):
        if not os.path.exists(dict_path):
            print(f"❌ [LaTeX Tutor] Dictionary not found at {dict_path}")
            return
            
        print(f"📡 [LaTeX Tutor] Loading LaTeX symbols from {dict_path}...")
        with open(dict_path, 'r') as f:
            symbols = json.load(f)
            
        print(f"🧩 [LaTeX Tutor] Mapping {len(symbols)} structural concepts to physical notation...")
        
        ABSOLUTE_CONFIDENCE = 1.0
        DIMENSION = "Pure Mathematics"
        
        for item in symbols:
            concept = item.get("concept")
            symbol = item.get("symbol")
            
            if not concept or not symbol:
                continue
                
            print(f" injecting logic: [{concept}] ==(Symbolized_By)==> [{symbol}]")
            
            # Add Nodes
            c_res = self.client.add_node(concept, DIMENSION)
            s_res = self.client.add_node(symbol, DIMENSION)
            
            if c_res and s_res:
                c_id = c_res["id"]
                s_id = s_res["id"]
                # Add Link
                self.client.add_link(c_id, s_id, "Symbolized_By", ABSOLUTE_CONFIDENCE)
                
        print("✅ [LaTeX Tutor] Mathematical notation layer applied.")

def main():
    parser = argparse.ArgumentParser(description="LaTeX Tutor - Map concepts to symbols")
    parser.add_argument("--dict", type=str, default="subconscious/latex_dictionary.json", help="Path to the JSON dictionary")
    
    args = parser.parse_args()
    
    tutor = LatexTutor()
    tutor.teach_symbols(args.dict)

if __name__ == "__main__":
    main()
