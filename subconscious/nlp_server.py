from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List, Dict, Optional
import spacy
from nltk.corpus import wordnet as wn
import nltk
import sys
import urllib.request
import urllib.parse
import json
import random

if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
    except Exception:
        pass

app = FastAPI(title="Axiom NLP Server")

from fastapi.middleware.cors import CORSMiddleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

print("🧠 [NLP Server] Loading spaCy offline NLP model (System 1)...")
try:
    nlp = spacy.load("en_core_web_sm")
    has_spacy = True
    print("✅ [NLP Server] spaCy loaded successfully. Ready for offline deep reading.")
except ImportError:
    has_spacy = False
    print("⚠️ [NLP Server] spaCy not found.")

print("🧠 [NLP Server] Loading NLTK WordNet (Offline Dictionary)...")
try:
    nltk.download('wordnet', quiet=True)
    nltk.download('vader_lexicon', quiet=True)
    has_nltk = True
    print("✅ [NLP Server] NLTK loaded for offline synonym and sentiment logic.")
except Exception:
    has_nltk = False
    print("⚠️ [NLP Server] NLTK load failed.")
    
if has_nltk:
    from nltk.sentiment.vader import SentimentIntensityAnalyzer
    sia = SentimentIntensityAnalyzer()

class ExtractRequest(BaseModel):
    text: str

class ExtractResponse(BaseModel):
    relations: List[Dict[str, str]]

@app.post("/extract", response_model=ExtractResponse)
def extract_relations(req: ExtractRequest):
    if not has_spacy:
        raise HTTPException(status_code=500, detail="SpaCy is not loaded.")
        
    doc = nlp(req.text)
    extracted = []
    
    # 1. Advanced Subject-Verb-Object (SVO) extraction
    for token in doc:
        if token.pos_ == "VERB" and not token.is_stop:
            subj = None
            obj = None
            for child in token.children:
                if child.dep_ in ("nsubj", "nsubjpass"):
                    subj = child.lemma_.title()
                elif child.dep_ in ("dobj", "pobj", "attr", "acomp", "prep"):
                    obj = child.lemma_.title()
            
            if subj and obj and len(subj) > 2 and len(obj) > 2:
                extracted.append({
                    "source": subj,
                    "target": obj,
                    "relation": token.lemma_.capitalize(),
                    "confidence": "0.85"
                })
                
    # 2. Adjective-Noun pairs
    for token in doc:
        if token.pos_ == "ADJ" and token.dep_ == "amod":
            head_noun = token.head.lemma_.title()
            adj = token.lemma_.title()
            if len(head_noun) > 2 and len(adj) > 2:
                extracted.append({
                    "source": head_noun,
                    "target": adj,
                    "relation": "Is",
                    "confidence": "0.90"
                })
                
    # 3. Noun Compounds
    for token in doc:
        if token.dep_ == "compound" and token.head.pos_ in ("NOUN", "PROPN"):
            modifier = token.lemma_.title()
            head_noun = token.head.lemma_.title()
            if len(modifier) > 2 and len(head_noun) > 2:
                extracted.append({
                    "source": head_noun,
                    "target": modifier,
                    "relation": "Relates_To",
                    "confidence": "0.80"
                })
                
    # Filter duplicates
    unique_extracted = { (e['source'], e['target'], e['relation']): e for e in extracted }
    
    # Enrich with WordNet
    final_relations = list(unique_extracted.values())
    
    # Enrich with VADER Sentiment/Emotion (Symbolic Feeling)
    if has_nltk and final_relations:
        scores = sia.polarity_scores(req.text)
        compound = scores['compound']
        
        # If the text is strongly emotional, link the main concepts to the emotion
        emotion_tag = None
        if compound > 0.5:
            emotion_tag = "Positive_Emotion"
        elif compound < -0.5:
            emotion_tag = "Negative_Emotion"
            
        if emotion_tag:
            concepts = set()
            for r in final_relations:
                concepts.add(r['source'])
                
            for c in concepts:
                final_relations.append({
                    "source": c,
                    "target": emotion_tag,
                    "relation": "Evokes",
                    "confidence": str(abs(compound))
                })
                
    if has_nltk:
        concepts = set()
        for r in final_relations:
            if r['relation'] != 'Evokes':
                concepts.add(r['source'].lower())
                concepts.add(r['target'].lower())
            
        for concept in concepts:
            synsets = wn.synsets(concept)
            if not synsets:
                continue
            
            syn_added = False
            ant_added = False
            
            for syn in synsets:
                for lemma in syn.lemmas():
                    syn_name = lemma.name().replace('_', ' ').title()
                    if syn_name.lower() != concept and not syn_added:
                        final_relations.append({
                            "source": concept.title(),
                            "target": syn_name,
                            "relation": "Synonym",
                            "confidence": "0.95"
                        })
                        syn_added = True
                        
                    if lemma.antonyms() and not ant_added:
                        ant_name = lemma.antonyms()[0].name().replace('_', ' ').title()
                        final_relations.append({
                            "source": concept.title(),
                            "target": ant_name,
                            "relation": "Antonym",
                            "confidence": "0.95"
                        })
                        ant_added = True
                
                if syn_added and ant_added:
                    break
                    
    return {"relations": final_relations}

class ChatRequest(BaseModel):
    query: str

class ChatResponse(BaseModel):
    answer: str

import urllib.request
import json
import urllib.parse
import random

# --- Advanced Symbolic NLP Chat Engine ---
last_subject = None

@app.post("/chat", response_model=ChatResponse)
def offline_chat(req: ChatRequest):
    global last_subject
    if not has_spacy:
        return {"answer": "My language center (SpaCy) is currently offline."}
        
    query = req.query.strip()
    
    # 1. Context Memory (Basic Coreference)
    query_lower = query.lower()
    if last_subject and any(query_lower.startswith(w) for w in ["he", "she", "it", "they"]):
        # Replace pronoun with last known subject
        words = query.split()
        words[0] = last_subject
        query = " ".join(words)
        
    doc = nlp(query)
    
    # 2. Intent Detection
    intent = "DESCRIBE"
    query_lower = query.lower()
    if "wisdom" in query_lower or "core belief" in query_lower or "central" in query_lower:
        intent = "WISDOM"
    else:
        for token in doc:
            if token.tag_ in ["WP", "WRB", "WDT"]: # WH-words
                if token.lemma_.lower() == "who": intent = "PERSON"
                elif token.lemma_.lower() == "where": intent = "LOCATION"
                elif token.lemma_.lower() == "why": intent = "REASON"
                elif token.lemma_.lower() == "what": intent = "DEFINITION"
                
    if intent == "WISDOM":
        try:
            url = "http://localhost:8080/api/centrality"
            req_obj = urllib.request.Request(url)
            with urllib.request.urlopen(req_obj) as response:
                data = json.loads(response.read().decode())
                beliefs = data.get("core_beliefs", [])
                if not beliefs:
                    return {"answer": "My graph is currently empty. I have not developed any core beliefs yet."}
                
                ans = "🧠 **VYOUM's Core Beliefs (Wisdom/Centrality)** 🧠\nBased on mathematical graph density, these are my fundamental anchors of thought:\n\n"
                for i, b in enumerate(beliefs):
                    ans += f"{i+1}. **{b['concept']}** (Centrality Score: {b['centrality_score']})\n"
                return {"answer": ans}
        except Exception as e:
            return {"answer": f"[SYSTEM_ERR] Neural link severed while fetching wisdom: {str(e)}"}
            
    # Extract concepts
    concepts = [token.lemma_.title() for token in doc if token.pos_ in ["NOUN", "PROPN"]]
    
    if not concepts:
        return {"answer": "I couldn't identify a clear subject. Please ask about a specific concept (e.g., 'Who is Alice?')."}
        
    main_concept = concepts[0]
    last_subject = main_concept # Save to short-term memory
    
    # 3. Graph Retrieval / Deduction
    try:
        if len(concepts) >= 2:
            # We have multiple concepts! Fire the Math Engine (Dijkstra)
            start_concept = concepts[0]
            goal_concept = concepts[-1]
            
            url = "http://localhost:8080/api/deduce"
            payload = json.dumps({"start": start_concept, "goal": goal_concept}).encode("utf-8")
            req_obj = urllib.request.Request(url, data=payload, headers={'Content-Type': 'application/json'})
            
            with urllib.request.urlopen(req_obj) as response:
                data = json.loads(response.read().decode())
                proven = data.get("proven", False)
                explanation = data.get("explanation", "")
                
                if proven:
                    final_answer = f"🧠 **VYOUM Mathematical Deduction Complete** 🧠\n\n{explanation}"
                else:
                    final_answer = f"I attempted to mathematically trace a logical path from '{start_concept}' to '{goal_concept}', but the graph currently lacks a continuous path of confidence between them."
                    
                return {"answer": final_answer}
                
        # Fallback to single concept query
        url = f"http://localhost:8080/api/ask?q={urllib.parse.quote(main_concept)}"
        req_obj = urllib.request.Request(url)
        with urllib.request.urlopen(req_obj) as response:
            data = json.loads(response.read().decode())
            rust_answer = data.get("answer", "")
            
            # 4. Synonym Expansion (if missing)
            if "has not yet encountered" in rust_answer and has_nltk:
                synsets = wn.synsets(main_concept.lower())
                if synsets:
                    for lemma in synsets[0].lemmas():
                        syn_name = lemma.name().title()
                        if syn_name != main_concept:
                            url = f"http://localhost:8080/api/ask?q={urllib.parse.quote(syn_name)}"
                            try:
                                with urllib.request.urlopen(urllib.request.Request(url)) as r2:
                                    data2 = json.loads(r2.read().decode())
                                    ans2 = data2.get("answer", "")
                                    if "has not yet encountered" not in ans2:
                                        rust_answer = f"(I don't know {main_concept}, but applying Synonym Expansion to {syn_name}):\n" + ans2
                                        break
                            except: pass

            # 5. Grammar Stitcher
            if "has not yet encountered" in rust_answer:
                final_answer = f"My neural graph currently lacks synaptic connections for '{main_concept}'."
            elif "isolated thought" in rust_answer:
                final_answer = f"I recognize '{main_concept}', but it exists in isolation without connected logic."
            else:
                # Format intros based on Intent
                intros = []
                if intent == "PERSON": intros = [f"Tracing entities related to {main_concept}:", f"Here is the biographical logic for {main_concept}:"]
                elif intent == "LOCATION": intros = [f"Mapping spatial logic for {main_concept}:", f"Geographic synaptic data for {main_concept}:"]
                elif intent == "DEFINITION": intros = [f"Defining '{main_concept}' structurally:", f"Here is the topological shape of {main_concept}:"]
                else: intros = [f"Accessing deep memory for {main_concept}:", f"Logical connections for {main_concept}:"]
                
                natural_text = rust_answer.replace("Through logical deduction, ", "I deduce that ")
                natural_text = natural_text.replace("->", "is linked to")
                final_answer = f"{random.choice(intros)}\n\n{natural_text}"
                
            return {"answer": final_answer}
    except Exception as e:
        return {"answer": f"[SYSTEM_ERR] Neural link severed: {str(e)}"}
# ============================================================
# SMART EAR + SMART MOUTH -- No External API Required
# ============================================================

RUST_ENGINE = 'http://localhost:8080'

HINDI_CONCEPT_MAP = {
    'roshni': ['Light', 'Optics', 'Photon', 'Electromagnetic Radiation'],
    'taakat': ['Force', 'Energy', 'Power', 'Momentum'],
    'waqt':   ['Time', 'Spacetime', 'Temporal'],
    'gravity': ['Gravity', 'Gravitational Force', 'General Relativity'],
    'bijli':  ['Electricity', 'Electromagnetism', 'Electric Field'],
    'dhara':  ['Current', 'Flow', 'Fluid'],
    'aasman': ['Space', 'Universe', 'Cosmology'],
    'pani':   ['Water', 'Fluid Dynamics', 'Hydrodynamics'],
    'aag':    ['Fire', 'Thermodynamics', 'Heat', 'Combustion'],
    'hawa':   ['Air', 'Atmosphere', 'Fluid Dynamics'],
    'sabit':  ['Proof', 'Theorem', 'Proves', 'Axiom'],
    'galat':  ['Contradiction', 'Falsification', 'Error', 'Paradox'],
    'sahi':   ['Truth', 'Valid', 'Axiom', 'Formal Logic'],
    'rishta': ['Relation', 'Connection', 'Link', 'Causality'],
    'sankhya':['Number', 'Number Theory', 'Mathematics'],
    'hisaab': ['Calculation', 'Arithmetic', 'Calculus'],
    'duniya': ['Reality', 'World', 'Universe', 'Physics'],
    'zindagi':['Life', 'Biology', 'Consciousness'],
    'insan':  ['Human', 'Consciousness', 'Anthropology'],
    'khuda':  ['God', 'Existence', 'Metaphysics', 'Causality'],
    'sach':   ['Truth', 'Axiom', 'Reality'],
    'jhoot':  ['Falsification', 'Contradiction', 'Paradox'],
    'mauka':  ['Probability', 'Chance', 'Quantum Mechanics'],
    'vichar': ['Thought', 'Consciousness', 'Cognition'],
    'gyan':   ['Knowledge', 'Epistemology', 'Information Theory'],
}

INTENT_PATTERNS = {
    'CONTRADICT': ['galat sabit', 'jhuta sabit', 'galat hai', 'disprove', 'contradict',
                   'prove wrong', 'sabit karo galat', 'refute', 'falsify', 'jhoot'],
    'CONNECT':    ['rishta', 'juda hai', 'kya sambandh', 'kaise juda', 'connected',
                   'link between', 'relation between', 'farq', 'difference', 'kya fark'],
    'REASON':     ['kyun', 'karan', 'kaise hota', 'why', 'cause', 'reason', 'kya wajah'],
    'PROCESS':    ['kaise kaam', 'kaise chalta', 'process', 'how does', 'mechanism'],
    'WISDOM':     ['sabse important', 'core belief', 'wisdom', 'best', 'central',
                   'sabse bada', 'mukhya', 'fundamental'],
    'DEFINE':     ['kya hai', 'what is', 'define', 'matlab', 'meaning', 'explain'],
}

def detect_intent(query: str) -> str:
    q = query.lower()
    for intent, patterns in INTENT_PATTERNS.items():
        if any(p in q for p in patterns):
            return intent
    return 'DEFINE'

def extract_concepts_smart(query: str) -> list:
    candidates = []
    q_lower = query.lower()
    for hindi_word, english_list in HINDI_CONCEPT_MAP.items():
        if hindi_word in q_lower:
            candidates.extend(english_list)
    if 'nlp' in globals():
        doc = nlp(query)
        for token in doc:
            if token.pos_ in ['NOUN', 'PROPN'] and not token.is_stop and len(token.lemma_) > 2:
                candidates.append(token.lemma_.title())
    seen = set()
    result = []
    for c in candidates:
        if c.lower() not in seen:
            seen.add(c.lower())
            result.append(c)
    return result[:5] if result else []

def call_rust(path: str, payload=None, method='POST') -> dict:
    import urllib.request, json
    try:
        url = RUST_ENGINE + path
        if method == 'GET':
            req = urllib.request.Request(url)
        else:
            data = json.dumps(payload).encode('utf-8')
            req = urllib.request.Request(url, data=data, headers={'Content-Type': 'application/json'})
        with urllib.request.urlopen(req, timeout=10) as r:
            return json.loads(r.read().decode())
    except Exception as e:
        return {'error': str(e)}

def stitch_chain_to_speech(chain: str, intent: str, concepts: list) -> str:
    if not chain:
        return "Chain of thought is empty."
    lines = chain.strip().split('\n')
    steps = []
    for line in lines:
        if line.startswith('Step ') and '[' in line:
            concept = line.split('[')[1].split(']')[0]
            if concept.startswith('Q') and concept[1:].replace('_','').isdigit():
                continue
            steps.append(concept)
    if not steps:
        return 'Mera neural graph mein is topic pe abhi kafi connections nahi hain.'
    if intent == 'CONNECT':
        if len(steps) >= 2:
            path_str = ' -> '.join(steps[:8])
            return 'Logical connection found:\n' + path_str
        return 'Connected to: ' + ', '.join(steps[:5])
    elif intent == 'CONTRADICT':
        return ('Ye concepts contradict karte hain:\n' + '\n'.join('  - ' + c for c in steps[:12]))
    elif intent == 'REASON':
        return ('Ye hain kaaran/wajah:\n' + '\n'.join('  - ' + c for c in steps[:12]))
    elif intent == 'PROCESS':
        return ('Process/mechanism:\n' + '\n'.join('  - ' + c for c in steps[:12]))
    elif intent == 'WISDOM':
        return ('Core beliefs:\n' + '\n'.join('  ' + str(i+1) + '. ' + c for i, c in enumerate(steps[:10])))
    else:
        return ('Main connections:\n' + '\n'.join('  - ' + c for c in steps[:15]))


class SmartQueryRequest(BaseModel):
    query: str

@app.post('/smart_query')
def smart_query(req: SmartQueryRequest):
    query = req.query.strip()
    print(f"\n[{'='*40}]")
    print(f"🎙️ [SMART EAR] Received Query: '{query}'")
    
    intent = detect_intent(query)
    print(f"🎯 [INTENT DETECTED]: {intent}")
    
    concepts = extract_concepts_smart(query)
    print(f"🧠 [CONCEPTS EXTRACTED]: {concepts}")
    
    if not concepts:
        print("❌ [WARNING] No concepts extracted. Returning generic fallback.")
        return {'answer': 'Sawal samajh nahi aaya. Thoda aur specific karein.', 'intent': intent, 'concepts': []}
    
    thought_chain = ''
    if intent == 'CONNECT' and len(concepts) >= 2:
        print(f"🔗 [RUST ENGINE] Firing /api/deduce with start='{concepts[0]}' and goal='{concepts[-1]}'")
        rust_response = call_rust('/api/deduce', {'start': concepts[0], 'goal': concepts[-1]})
        if rust_response.get('proven'):
            print("✅ [RUST ENGINE] Proof Path Found!")
            return {'answer': 'Path found:\n' + rust_response.get('explanation', ''), 'intent': intent, 'concepts': concepts}
        print("⚠️ [RUST ENGINE] No direct proof found. Falling back to Thought Stream (No Filter).")
        rust_response = call_rust('/api/thought_stream_filtered', {'concepts': concepts, 'relation_filter': None})
        thought_chain = rust_response.get('thought_chain', '')
    elif intent == 'CONTRADICT':
        print(f"⚔️ [RUST ENGINE] Firing /api/thought_stream_filtered with relation_filter='Contradicts'")
        rust_response = call_rust('/api/thought_stream_filtered', {'concepts': concepts, 'relation_filter': 'Contradicts'})
        thought_chain = rust_response.get('thought_chain', '')
    elif intent == 'REASON':
        print(f"❓ [RUST ENGINE] Firing /api/thought_stream_filtered with relation_filter='Causes'")
        rust_response = call_rust('/api/thought_stream_filtered', {'concepts': concepts, 'relation_filter': 'Causes'})
        thought_chain = rust_response.get('thought_chain', '')
    elif intent == 'WISDOM':
        print(f"🦉 [RUST ENGINE] Firing /api/centrality")
        rust_response = call_rust('/api/centrality', method='GET')
        beliefs = rust_response.get('core_beliefs', [])
        if beliefs:
            ans = 'Core beliefs:\n' + '\n'.join('  ' + str(i+1) + '. ' + b['concept'] for i, b in enumerate(beliefs[:10]))
            print(f"✅ [RUST ENGINE] Extracted {len(beliefs)} core beliefs.")
            return {'answer': ans, 'intent': intent, 'concepts': concepts}
        thought_chain = ''
    else:
        print(f"🔍 [RUST ENGINE] Firing /api/thought_stream_filtered with relation_filter='Implies'")
        rust_response = call_rust('/api/thought_stream_filtered', {'concepts': concepts, 'relation_filter': 'Implies'})
        thought_chain = rust_response.get('thought_chain', '')
        if not thought_chain or 'No ' in thought_chain:
            print("⚠️ [RUST ENGINE] No implications found. Falling back to un-filtered thought_stream.")
            rust_response = call_rust('/api/thought_stream', {'concepts': concepts})
            thought_chain = rust_response.get('thought_chain', '')
            
    print(f"⚙️ [RUST ENGINE RAW RESPONSE PREVIEW]:\n{thought_chain[:150]}...")
    answer = stitch_chain_to_speech(thought_chain, intent, concepts)
    print(f"🗣️ [SMART MOUTH] Final Spoken Output:\n{answer}")
    print(f"[{'='*40}]\n")
    return {'answer': answer, 'intent': intent, 'concepts': concepts, 'raw_preview': thought_chain[:300] if thought_chain else ''}

if __name__ == '__main__':
    import uvicorn
    uvicorn.run(app, host='0.0.0.0', port=8081)
