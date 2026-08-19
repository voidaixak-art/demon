// Axiom Engine 2.0 - NLP Chat Terminal
const voiceForm = document.getElementById('voice-form');
const voiceInput = document.getElementById('voice-input');
const voiceTerminal = document.getElementById('voice-terminal');

function addVoiceLine(text, type) {
  const div = document.createElement('div');
  div.className = `voice-line ${type}`;
  div.textContent = text;
  voiceTerminal.appendChild(div);
  voiceTerminal.scrollTop = voiceTerminal.scrollHeight;
}

voiceForm.addEventListener('submit', async (e) => {
  e.preventDefault();
  const query = voiceInput.value.trim();
  if (!query) return;

  addVoiceLine(`root@axiom:~$ ${query}`, 'user');
  voiceInput.value = '';

  try {
    // Ping the Offline Python NLP Server
    const res = await fetch(`http://localhost:8081/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query: query })
    });
    const data = await res.json();
    
    // The NLP server formats the response naturally
    const lines = data.answer.split('\n');
    lines.forEach(line => {
      if (line.trim()) addVoiceLine(line, 'logic');
    });
  } catch (err) {
    addVoiceLine(`[ERROR] Connection to Subconscious NLP failed: ${err.message}`, 'system');
  }
});
