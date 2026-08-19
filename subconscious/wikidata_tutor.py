import json
import os
import sys
import argparse
import requests
import time
from subconscious.client import AxiomClient

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

WIKIDATA_SPARQL_URL = "https://query.wikidata.org/sparql"

class WikidataTutor:
    def __init__(self):
        self.client = AxiomClient()
        print("🧠 [Wikidata Tutor] Connecting to Axiom Engine...")

    def query_wikidata(self, qid: str, limit: int = 50) -> list:
        print(f"🌐 [Wikidata Tutor] Querying Global Knowledge Graph for {qid}...")
        
        # We look for things that are Subclass Of (P279) or Part Of (P361) the given QID
        query = f"""
        SELECT ?concept ?conceptLabel ?rel ?relLabel WHERE {{
          {{
            ?concept wdt:P279 wd:{qid}.
            BIND("Subclass_Of" AS ?relLabel)
          }} UNION {{
            ?concept wdt:P361 wd:{qid}.
            BIND("Part_Of" AS ?relLabel)
          }}
          SERVICE wikibase:label {{ bd:serviceParam wikibase:language "[AUTO_LANGUAGE],en". }}
        }}
        LIMIT {limit}
        """

        headers = {
            "Accept": "application/json",
            "User-Agent": "Antigravity_VYOUM_Agent/1.0"
        }
        
        try:
            response = requests.get(WIKIDATA_SPARQL_URL, params={'query': query}, headers=headers, timeout=20)
            if response.status_code == 200:
                data = response.json()
                return data.get("results", {}).get("bindings", [])
            else:
                print(f"❌ [Wikidata Tutor] API Error: {response.status_code} - {response.text}")
        except Exception as e:
            print(f"❌ [Wikidata Tutor] Request failed: {e}")
        
        return []

    def teach_module(self, module: dict, target_domain: str = "Pure Mathematics"):
        qid = module.get("qid")
        name = module.get("name")
        
        print(f"\n==============================================")
        print(f"🎓 [Wikidata Tutor] Starting Module: {name} ({qid})")
        print(f"==============================================")
        
        bindings = self.query_wikidata(qid, limit=1000)
        
        if not bindings:
            print(f"⚠️ [Wikidata Tutor] No structural relations found for {name}.")
            return
            
        print(f"🧩 [Wikidata Tutor] Extracted {len(bindings)} absolute axioms from Wikidata.")
        
        ABSOLUTE_CONFIDENCE = 1.0
        
        for binding in bindings:
            source = binding.get("conceptLabel", {}).get("value")
            relation = binding.get("relLabel", {}).get("value")
            target = name
            
            # Skip if Wikidata didn't have an english label and just returned a QID URL
            if not source or source.startswith("http://"):
                continue
                
            print(f" injecting logic: [{source}] ==({relation})==> [{target}]")
            
            # 1. Add Nodes
            s_res = self.client.add_node(source, target_domain)
            t_res = self.client.add_node(target, target_domain)

            if s_res and t_res:
                s_id = s_res["id"]
                t_id = t_res["id"]
                # 2. Add Link with 1.0 confidence
                self.client.add_link(s_id, t_id, relation, ABSOLUTE_CONFIDENCE)
                
        print(f"✅ [Wikidata Tutor] Completed Module: {name}")

    def run_syllabus(self, syllabus_path: str):
        if not os.path.exists(syllabus_path):
            print(f"❌ [Wikidata Tutor] Syllabus not found at {syllabus_path}")
            return
            
        with open(syllabus_path, 'r') as f:
            syllabus = json.load(f)
            
        for module in syllabus:
            self.teach_module(module)
            time.sleep(2) # Breath between modules to respect API limits

def main():
    parser = argparse.ArgumentParser(description="Wikidata Tutor - Automated Syllabus Ingestion")
    parser.add_argument("--syllabus", type=str, default="subconscious/syllabus.json", help="Path to the JSON syllabus")
    
    args = parser.parse_args()
    
    tutor = WikidataTutor()
    tutor.run_syllabus(args.syllabus)

if __name__ == "__main__":
    main()
