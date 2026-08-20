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
        response_text = engine_chat.ask(req.message)
        return {"response": response_text}
    except Exception as e:
        return JSONResponse(status_code=500, content={"error": str(e)})

@app.get("/", response_class=HTMLResponse)
async def serve_index():
    with open(os.path.join(static_dir, "index.html"), "r", encoding="utf-8") as f:
        return f.read()

if __name__ == "__main__":
    print("🚀 Starting Axiom Engine Web Chat on http://127.0.0.1:8000")
    uvicorn.run(app, host="127.0.0.1", port=8000)
