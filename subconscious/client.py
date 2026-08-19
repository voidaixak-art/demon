import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry
from typing import Dict, List, Optional, Any
from .config import AXIOM_ENGINE_URL

class AxiomClient:
    def __init__(self, base_url: str = AXIOM_ENGINE_URL):
        self.base_url = base_url.rstrip("/")
        
        # Configure robust HTTP session with automatic retries and connection pooling
        self.session = requests.Session()
        retries = Retry(
            total=4,
            backoff_factor=0.5,
            status_forcelist=[500, 502, 503, 504],
            raise_on_status=False
        )
        adapter = HTTPAdapter(max_retries=retries, pool_connections=10, pool_maxsize=20)
        self.session.mount("http://", adapter)
        self.session.mount("https://", adapter)

    def ping(self) -> bool:
        try:
            r = self.session.get(f"{self.base_url}/api/stats", timeout=3)
            return r.status_code == 200
        except Exception:
            return False

    def get_stats(self) -> Dict[str, Any]:
        try:
            r = self.session.get(f"{self.base_url}/api/stats", timeout=6)
            if r.status_code == 200:
                return r.json()
        except Exception as e:
            print(f"[Client] Warning in get_stats: {e}")
        return {"nodes": 0, "links": 0, "density": 0.0, "self_connections": 0}

    def get_nodes(self) -> List[Dict[str, Any]]:
        try:
            r = self.session.get(f"{self.base_url}/api/nodes", timeout=12)
            if r.status_code == 200:
                return r.json().get("nodes", [])
        except Exception as e:
            print(f"[Client] Warning in get_nodes: {e}")
        return []

    def get_node(self, node_id: int) -> Optional[Dict[str, Any]]:
        try:
            r = self.session.get(f"{self.base_url}/api/nodes/{node_id}", timeout=6)
            if r.status_code == 200:
                return r.json()
        except Exception as e:
            print(f"[Client] Warning in get_node({node_id}): {e}")
        return None

    def add_node(self, concept: str, dimension: str = "General", vector: Optional[List[float]] = None) -> Dict[str, Any]:
        payload = {"concept": concept, "dimension": dimension, "vector": vector}
        try:
            r = self.session.post(f"{self.base_url}/api/nodes", json=payload, timeout=8)
            if r.status_code == 200:
                return r.json()
        except Exception as e:
            print(f"[Client] Warning in add_node: {e}")
        return {"id": 0, "concept": concept, "dimension": dimension, "is_new": False}

    def add_link(self, source_id: int, target_id: int, relation: str = "Implies", confidence: float = 1.0) -> bool:
        payload = {
            "source_id": source_id,
            "target_id": target_id,
            "relation": relation,
            "confidence": confidence
        }
        try:
            r = self.session.post(f"{self.base_url}/api/links", json=payload, timeout=8)
            return r.status_code == 200
        except Exception as e:
            print(f"[Client] Warning in add_link: {e}")
            return False

    def deduce(self, start: str, goal: str) -> Dict[str, Any]:
        payload = {"start": start, "goal": goal}
        try:
            r = self.session.post(f"{self.base_url}/api/deduce", json=payload, timeout=10)
            if r.status_code == 200:
                return r.json()
        except Exception as e:
            print(f"[Client] Warning in deduce: {e}")
        return {"proven": False, "explanation": "Deduction call timed out or failed to connect."}

    def get_orphans(self) -> List[Dict[str, Any]]:
        try:
            r = self.session.get(f"{self.base_url}/api/orphans", timeout=6)
            if r.status_code == 200:
                return r.json().get("orphans", [])
        except Exception as e:
            print(f"[Client] Warning in get_orphans: {e}")
        return []

    def run_dream(self, trials: int = 100, threshold: float = 0.52) -> Dict[str, Any]:
        payload = {"trials": trials, "threshold": threshold}
        try:
            r = self.session.post(f"{self.base_url}/api/dream", json=payload, timeout=20)
            if r.status_code == 200:
                return r.json()
        except Exception as e:
            print(f"[Client] Warning in run_dream: {e}")
        return {"total_trials": trials, "successful_synapses": [], "consolidated_orphans": 0}

    def get_contradictions(self) -> List[Dict[str, Any]]:
        try:
            r = self.session.get(f"{self.base_url}/api/contradictions", timeout=6)
            if r.status_code == 200:
                return r.json().get("contradictions", [])
        except Exception as e:
            print(f"[Client] Warning in get_contradictions: {e}")
        return []

    def heal(self, id1: int, id2: int) -> bool:
        payload = {"id1": id1, "id2": id2}
        try:
            r = self.session.post(f"{self.base_url}/api/heal", json=payload, timeout=6)
            return r.status_code == 200
        except Exception as e:
            print(f"[Client] Warning in heal: {e}")
            return False

    def log_thought(self, stage: str, message: str) -> bool:
        try:
            payload = {"stage": stage, "message": message}
            r = self.session.post(f"{self.base_url}/api/logs", json=payload, timeout=4)
            return r.status_code == 200
        except Exception:
            return False

    def get_context(self, query: str, limit: int = 5) -> str:
        payload = {"query": query, "limit": limit}
        try:
            r = self.session.post(f"{self.base_url}/api/context", json=payload, timeout=6)
            if r.status_code == 200:
                return r.json().get("context", "")
        except Exception as e:
            print(f"[Client] Warning in get_context: {e}")
        return ""
