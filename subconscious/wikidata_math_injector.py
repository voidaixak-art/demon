import requests
import json
import time
import sys
sys.stdout.reconfigure(encoding='utf-8')

WIKIDATA_ENDPOINT = "https://query.wikidata.org/sparql"
ENGINE_URL = "http://localhost:8080"

# User-Agent is required by Wikidata to prevent blocking
HEADERS = {
    "User-Agent": "VyoumAGI/1.0 (https://github.com/vyoum/agi; bot@vyoum.local)",
    "Accept": "application/sparql-results+json"
}

def sparql_query(query):
    try:
        response = requests.get(WIKIDATA_ENDPOINT, params={'format': 'json', 'query': query}, headers=HEADERS, timeout=10)
        response.raise_for_status()
        return response.json()['results']['bindings']
    except Exception as e:
        print(f"[-] Wikidata query failed: {e}")
        return []

def get_math_concept(concept_name):
    """
    Finds the exact Wikidata Q-ID for a math concept and extracts its properties.
    Properties we care about:
    P31 = instance of
    P279 = subclass of
    P361 = part of
    P2534 = defining formula
    """
    print(f"\n[?] Searching Wikidata for: {concept_name}")
    
    # We use Wikidata Search API first to get the QID to be precise
    search_url = "https://www.wikidata.org/w/api.php"
    search_params = {
        "action": "wbsearchentities",
        "format": "json",
        "language": "en",
        "search": concept_name,
        "limit": 1
    }
    
    try:
        search_res = requests.get(search_url, params=search_params, headers=HEADERS, timeout=10).json()
        if not search_res.get('search'):
            print(f"  [-] Not found in Wikidata: {concept_name}")
            return None
        
        qid = search_res['search'][0]['id']
        label = search_res['search'][0]['label']
        print(f"  [+] Found QID: {qid} ({label})")
        
        # Now we query its mathematical topology using SPARQL
        query = f"""
        SELECT ?propLabel ?valLabel ?formula WHERE {{
          {{ wd:{qid} wdt:P31 ?val . BIND("instance of" AS ?propLabel) }}
          UNION
          {{ wd:{qid} wdt:P279 ?val . BIND("subclass of" AS ?propLabel) }}
          UNION
          {{ wd:{qid} wdt:P361 ?val . BIND("part of" AS ?propLabel) }}
          UNION
          {{ wd:{qid} wdt:P2534 ?formula . BIND("has formula" AS ?propLabel) }}
          
          OPTIONAL {{ ?val rdfs:label ?valLabel . FILTER(LANG(?valLabel) = "en") }}
        }}
        """
        
        bindings = sparql_query(query)
        
        properties = []
        formulas = []
        
        for b in bindings:
            prop = b.get('propLabel', {}).get('value')
            if prop == "has formula":
                form = b.get('formula', {}).get('value')
                if form:
                    formulas.append(form)
            else:
                val = b.get('valLabel', {}).get('value')
                if val:
                    properties.append({"relation": prop, "target": val})
                    
        return {
            "concept": label,
            "properties": properties,
            "formulas": formulas
        }
        
    except Exception as e:
        print(f"[-] Search API error: {e}")
        return None

def inject_to_engine(data):
    """
    Takes the structured Wikidata data and injects it into VYOUM's Rust Engine
    """
    if not data:
        return
    
    concept = data['concept']
    
    # 1. Create Main Node
    try:
        res = requests.post(f"{ENGINE_URL}/api/nodes", json={"concept": concept, "dimension": "Mathematics"}, timeout=10)
        if res.status_code != 200:
            print(f"  [!] Failed to inject node: {concept}")
            return
        
        source_id = res.json().get('id')
        print(f"  [+] Injected Node: {concept} (ID: {source_id})")
        
        # 2. Inject Topology (Relations)
        for prop in data['properties']:
            target_concept = prop['target']
            relation = prop['relation']
            
            # Map Wikidata relation to VYOUM relation
            vyoum_rel = "Implies"
            if relation == "part of":
                vyoum_rel = "Causes"
            elif relation == "subclass of":
                vyoum_rel = "Equivalent"
                
            # Create Target Node first
            tres = requests.post(f"{ENGINE_URL}/api/nodes", json={"concept": target_concept, "dimension": "Mathematics"}, timeout=10)
            target_id = tres.json().get('id')
            
            # Create Link
            requests.post(f"{ENGINE_URL}/api/links", json={
                "source_id": source_id,
                "target_id": target_id,
                "relation": vyoum_rel,
                "confidence": 0.95
            }, timeout=10)
            print(f"      -> Linked: {concept} --({vyoum_rel})--> {target_concept}")

        # 3. Inject Formulas
        for form in data['formulas']:
            # Create Formula Node
            fres = requests.post(f"{ENGINE_URL}/api/nodes", json={"concept": form, "dimension": "Formula"}, timeout=10)
            form_id = fres.json().get('id')
            
            # Link Formula to Concept
            requests.post(f"{ENGINE_URL}/api/links", json={
                "source_id": source_id,
                "target_id": form_id,
                "relation": "Equivalent",
                "confidence": 1.0
            }, timeout=10)
            print(f"      -> Linked Formula: {concept} == {form}")
            
    except Exception as e:
        print(f"  [-] Engine injection error: {e}")

if __name__ == "__main__":
    # Read the master curriculum artifact
    curriculum = []
    try:
        with open(r"C:\Users\hlo\.gemini\antigravity-ide\brain\51ffbe24-822e-4c6d-b28e-f2eee22cf009\math_curriculum.md", "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line.startswith("- ") and not line.startswith("- **"): 
                    # Extract concepts from bullet points
                    topic = line[2:].strip()
                    if ":" in topic: # Remove definitions if they exist in the line
                        topic = topic.split(":")[0].strip()
                    # Split comma separated topics (e.g., "Set, Element, Empty Set")
                    for sub_topic in topic.split(","):
                        clean_topic = sub_topic.strip()
                        # If there is a parenthesis, remove it and everything after it
                        if "(" in clean_topic:
                            clean_topic = clean_topic.split("(")[0].strip()
                            
                        if clean_topic and len(clean_topic) > 2:
                            curriculum.append(clean_topic)
    except Exception as e:
        print(f"Failed to read curriculum file: {e}")
        
    print(f"Loaded {len(curriculum)} specific mathematical concepts from the Master Curriculum.")

    
    print("Starting Wikidata Math Injector...")
    for topic in curriculum:
        data = get_math_concept(topic)
        inject_to_engine(data)
        time.sleep(2) # Be polite to Wikidata API
        
    print("\nInjection Complete!")
