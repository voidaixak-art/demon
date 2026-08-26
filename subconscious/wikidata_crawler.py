import requests
import time
import sys

# The Rust Engine endpoint
ENGINE_URL = "http://localhost:8080"
SPARQL_URL = "https://query.wikidata.org/sparql"

def query_wikidata(sparql_query):
    """Hits the Wikidata SPARQL endpoint and returns JSON"""
    headers = {
        'User-Agent': 'VYOUM_AGI_Bot/1.0 (vyoum@axiom.engine)',
        'Accept': 'application/sparql-results+json'
    }
    try:
        response = requests.get(SPARQL_URL, params={'format': 'json', 'query': sparql_query}, headers=headers)
        if response.status_code == 200:
            return response.json()
        elif response.status_code == 429:
            print("[Rate Limited] Waiting 5 seconds...")
            time.sleep(5)
            return query_wikidata(sparql_query)
        else:
            print(f"Error querying Wikidata: {response.status_code}")
            return None
    except Exception as e:
        print(f"Connection error: {e}")
        return None

def inject_node(concept, dimension="Universe"):
    """Injects a single concept node into the Rust engine"""
    try:
        res = requests.post(f"{ENGINE_URL}/api/nodes", json={
            "concept": concept,
            "dimension": dimension
        })
        if res.status_code == 200:
            return res.json().get('id')
    except:
        pass
    return None

def inject_link(source_id, target_id, relation_str):
    """Injects an edge between two concepts"""
    try:
        requests.post(f"{ENGINE_URL}/api/links", json={
            "source_id": source_id,
            "target_id": target_id,
            "relation": relation_str,
            "confidence": 1.0 # 100% confidence for Wikidata facts
        })
    except:
        pass

def fetch_core_ontology():
    print("\n\033[96m[VYOUM Subconscious: Initializing Wikidata Knowledge Ingestion]\033[0m")
    
    # Query: Fetch core scientific and philosophical concepts (subclasses of 'science' or 'branch of philosophy')
    # For now, let's just fetch basic Physical Quantities and Fundamental Forces to seed the "Physics" dimension
    query = """
    SELECT ?itemLabel ?classLabel WHERE {
      # Instances of Fundamental Forces (Q1141151) or Physical Quantities (Q107715)
      { ?item wdt:P31 wd:Q1141151. } UNION { ?item wdt:P31 wd:Q107715. }
      ?item wdt:P279 ?class. # Subclass of
      SERVICE wikibase:label { bd:serviceParam wikibase:language "en". }
    }
    LIMIT 200
    """
    
    print("[Fetching Universe Core Logic from Wikidata...]")
    data = query_wikidata(query)
    
    if not data:
        print("Failed to fetch data.")
        return
        
    results = data.get('results', {}).get('bindings', [])
    print(f"[Received {len(results)} axioms. Injecting into Axiom Engine...]")
    
    count = 0
    for row in results:
        child = row.get('itemLabel', {}).get('value', '').lower()
        parent = row.get('classLabel', {}).get('value', '').lower()
        
        # Filter out noisy URIs or Q-codes
        if child.startswith('http') or child.startswith('q') or parent.startswith('http') or parent.startswith('q'):
            continue
            
        print(f"Injecting Rule: [{child}] is a [{parent}]")
        
        # 1. Inject Child Node
        child_id = inject_node(child, "Physics")
        # 2. Inject Parent Node
        parent_id = inject_node(parent, "Physics")
        
        # 3. Form Mathematical Link (Instance/Subclass -> Implies)
        if child_id and parent_id:
            inject_link(child_id, parent_id, "Implies")
            count += 1
            
        time.sleep(0.05) # Prevent overloading Rust Server
        
    print(f"\n\033[92m[Ingestion Complete: {count} Core Axioms Integrated]\033[0m")
    print("Check the Dashboard or 3D Graph to see the pristine structure!")

if __name__ == "__main__":
    fetch_core_ontology()
