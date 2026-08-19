import sys
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass
from subconscious.client import AxiomClient
import time

def inject_category_theory():
    client = AxiomClient()
    print("🧠 [Category Theory Tutor] Connecting to Axiom Engine...")
    
    concepts = [
        "Category Theory",
        "Functor",
        "Morphism",
        "Object",
        "Natural Transformation",
        "Adjunction",
        "Limit",
        "Colimit",
        "Topos"
    ]
    
    # 1. Add all concepts as nodes
    for concept in concepts:
        client.add_node(concept, "Pure Mathematics")
        
    print("✅ Nodes added.")
    
    # 2. Add relationships
    relations = [
        ("Functor", "Category Theory", "Part_Of"),
        ("Morphism", "Category Theory", "Part_Of"),
        ("Object", "Category Theory", "Part_Of"),
        ("Natural Transformation", "Category Theory", "Part_Of"),
        ("Adjunction", "Category Theory", "Part_Of"),
        ("Limit", "Category Theory", "Part_Of"),
        ("Colimit", "Category Theory", "Part_Of"),
        ("Topos", "Category Theory", "Part_Of"),
        
        # Cross-domain mappings that Functors solve
        ("Algebra", "Category Theory", "Mapped_Via"),
        ("Topology", "Category Theory", "Mapped_Via"),
        ("Geometry", "Category Theory", "Mapped_Via"),
        ("Set Theory", "Category Theory", "Mapped_Via"),
        ("Physics", "Category Theory", "Mapped_Via")
    ]
    
    for s, t, r in relations:
        s_res = client.add_node(s, "Pure Mathematics")
        t_res = client.add_node(t, "Pure Mathematics")
        
        if s_res and t_res:
            client.add_link(s_res["id"], t_res["id"], r, 1.0)
            print(f" injecting logic: [{s}] ==({r})==> [{t}]")
            
    print("✅ [Category Theory Tutor] Knowledge successfully injected into the VYOUM engine.")

if __name__ == "__main__":
    inject_category_theory()
