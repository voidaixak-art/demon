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

class SymbolicLearner:
    def __init__(self):
        self.client = AxiomClient()
        print("🧠 [Symbolic Learner] Connecting to Axiom Engine...")

    def learn_curriculum(self, file_path: str):
        if not os.path.exists(file_path):
            print(f"❌ [Symbolic Learner] Curriculum file not found: {file_path}")
            return

        print(f"📡 [Symbolic Learner] Loading absolute axioms from {file_path}...")
        try:
            with open(file_path, 'r') as f:
                axioms = json.load(f)
        except json.JSONDecodeError as e:
            print(f"❌ [Symbolic Learner] Invalid JSON format: {e}")
            return

        print(f"🧩 [Symbolic Learner] Processing {len(axioms)} axiomatic rules...")
        
        # In mathematics, truth is absolute. Confidence is 1.0.
        ABSOLUTE_CONFIDENCE = 1.0
        DIMENSION = "Pure Mathematics"

        for axiom in axioms:
            source = axiom.get("source")
            target = axiom.get("target")
            relation = axiom.get("relation")

            if not source or not target or not relation:
                print(f"⚠️ Skipping invalid axiom: {axiom}")
                continue

            print(f" injecting logic: [{source}] ==({relation})==> [{target}]")

            # 1. Add Nodes
            s_res = self.client.add_node(source, DIMENSION)
            t_res = self.client.add_node(target, DIMENSION)

            if s_res and t_res:
                s_id = s_res["id"]
                t_id = t_res["id"]
                # 2. Add Link with 1.0 confidence
                if self.client.add_link(s_id, t_id, relation, ABSOLUTE_CONFIDENCE):
                    print(f"   🔗 Universal Truth Wired: [{source}] ===({relation})===> [{target}]")
                else:
                    print(f"   ❌ Failed to wire truth.")
            else:
                print(f"   ❌ Failed to create conceptual nodes.")

def main():
    parser = argparse.ArgumentParser(description="Symbolic Engine - Strict Mathematical Ingestion")
    parser.add_argument("--curriculum", type=str, required=True, help="Path to the JSON curriculum file")
    
    args = parser.parse_args()
    
    learner = SymbolicLearner()
    learner.learn_curriculum(args.curriculum)

if __name__ == "__main__":
    main()
