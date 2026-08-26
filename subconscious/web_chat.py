import uvicorn
from fastapi import FastAPI, Request
from fastapi.staticfiles import StaticFiles
from fastapi.responses import JSONResponse, HTMLResponse
import os
import sys

# Ensure UTF-8 output on Windows consoles
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

from .chat import AxiomChat
from pydantic import BaseModel

app = FastAPI(title="Axiom Engine Web Interface")

# Initialize Chat
engine_chat = AxiomChat()

class ChatRequest(BaseModel):
    message: str

# Create static directory if not exists
static_dir = os.path.join(os.path.dirname(__file__), "static")
os.makedirs(static_dir, exist_ok=True)

# Mount static files
app.mount("/static", StaticFiles(directory=static_dir), name="static")

@app.post("/api/chat")
async def chat_endpoint(req: ChatRequest):
    if not engine_chat.client.ping():
        return JSONResponse(status_code=500, content={"error": "Axiom Core daemon is not running."})

    try:
        # PRIMARY: Route through NATIVE RUST VECTOR RESONANCE (Direct Ear & Mouth)
        import urllib.request, json as _json
        nlp_payload = _json.dumps({"query": req.message}).encode("utf-8")
        nlp_req = urllib.request.Request(
            "http://localhost:8080/api/smart_ask",
            data=nlp_payload,
            headers={"Content-Type": "application/json"}
        )
        with urllib.request.urlopen(nlp_req, timeout=12) as r:
            nlp_data = _json.loads(r.read().decode())
            answer = nlp_data.get("thought_chain", "")
            concepts = nlp_data.get("seed_concepts", [])
            if answer:
                return {
                    "response": answer,
                    "meta": {"intent": "NATIVE_RESONANCE", "concepts": concepts, "source": "RUST_ENGINE"}
                }
    except Exception as e:
        print(f"   -> [Resonance fallback] Rust smart_ask unavailable ({e}), using Gemini pipeline...")

    # FALLBACK: Original chat.py + Gemini pipeline
    try:
        response_text = engine_chat.ask(req.message)
        return {"response": response_text, "meta": {"source": "GEMINI_FALLBACK"}}
    except Exception as e:
        return JSONResponse(status_code=500, content={"error": str(e)})

@app.get("/", response_class=HTMLResponse)
async def serve_index():
    with open(os.path.join(static_dir, "index.html"), "r", encoding="utf-8") as f:
        return f.read()

if __name__ == "__main__":
    print("🚀 Starting Axiom Engine Web Chat on http://127.0.0.1:8000")
    uvicorn.run(app, host="127.0.0.1", port=8000)
