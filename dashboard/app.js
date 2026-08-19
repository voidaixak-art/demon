// Axiom Engine 2.0 - 3D Force Graph Controller
const API_BASE = window.location.origin;

const Graph = ForceGraph3D()
  (document.getElementById('3d-graph'))
    .backgroundColor('#050505')
    .showNavInfo(false)
    .nodeAutoColorBy('dimension')
    .nodeVal(node => node.id === 0 ? 30 : Math.max(5, node.activation_count * 2))
    .nodeLabel(node => {
      return `
        <div class="graph-tooltip">
          <strong>${node.concept}</strong><br/>
          Dimension: ${node.dimension}<br/>
          Activations: ${node.activation_count}<br/>
          ID: ${node.id}
        </div>
      `;
    })
    .linkLabel(link => {
      return `
        <div class="graph-tooltip">
          ${link.relation}<br/>
          Confidence: ${(link.confidence * 100).toFixed(1)}%
        </div>
      `;
    })
    .linkDirectionalParticles(2)
    .linkDirectionalParticleWidth(1.5)
    .linkDirectionalParticleSpeed(d => d.confidence * 0.005)
    .linkWidth(d => d.confidence * 0.5)
    .linkColor(() => 'rgba(255, 255, 255, 0.2)');

async function fetchBrainState() {
  try {
    const [statsRes, nodesRes, linksRes] = await Promise.all([
      fetch(`${API_BASE}/api/stats`).then(r => r.json()),
      fetch(`${API_BASE}/api/nodes`).then(r => r.json()),
      fetch(`${API_BASE}/api/links`).then(r => r.json())
    ]);

    // Update Telemetry
    document.getElementById('stat-nodes').innerText = statsRes.nodes || 0;
    document.getElementById('stat-links').innerText = statsRes.links || 0;
    document.getElementById('stat-ego').innerText = statsRes.self_connections || 0;
    
    document.getElementById('daemon-status').innerText = 'ONLINE';
    document.getElementById('daemon-status').style.color = '#00ffcc';

    // Format Data for 3d-force-graph
    const graphData = {
      nodes: nodesRes.nodes.map(n => ({
        ...n,
        // EGO node is larger
        val: n.id === 0 ? 50 : (n.activation_count || 1)
      })),
      links: linksRes.links.map(l => ({
        ...l,
        source: l.source_id,
        target: l.target_id
      }))
    };

    Graph.graphData(graphData);
  } catch (err) {
    console.error("Failed to connect to Axiom Engine:", err);
    document.getElementById('daemon-status').innerText = 'OFFLINE';
    document.getElementById('daemon-status').style.color = '#ff0055';
  }
}

// Initial Load
fetchBrainState();

// Auto-sync every 5 seconds
setInterval(fetchBrainState, 5000);

// Manual Sync Button
document.getElementById('btn-refresh').addEventListener('click', () => {
  fetchBrainState();
});// Voice Box Logic
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

  addVoiceLine(`> ${query}`, 'user');
  voiceInput.value = '';

  try {
    const res = await fetch(`/api/ask?q=${encodeURIComponent(query)}`);
    const data = await res.json();
    
    // The server returns responses separated by \n
    const lines = data.answer.split('\n');
    lines.forEach(line => addVoiceLine(line, 'logic'));
  } catch (err) {
    addVoiceLine(`Error querying Subconscious: ${err.message}`, 'logic');
  }
});
