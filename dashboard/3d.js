// VYOUM 3D Core Map - Pure Math Engine

// Three.js Setup
// ... [keeping same setup]
// ...
const container = document.getElementById('canvas-container');
const scene = new THREE.Scene();

// Camera
const camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 0.1, 10000);
camera.position.z = 2500;

// Renderer
const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
renderer.setSize(window.innerWidth, window.innerHeight);
renderer.setPixelRatio(window.devicePixelRatio);
container.appendChild(renderer.domElement);

// Controls
const controls = new THREE.OrbitControls(camera, renderer.domElement);
controls.enableDamping = true;
controls.dampingFactor = 0.05;
controls.autoRotate = true;
controls.autoRotateSpeed = 0.5;

// Lighting (Subtle for neon effect)
const ambientLight = new THREE.AmbientLight(0xffffff, 0.2);
scene.add(ambientLight);
const pointLight = new THREE.PointLight(0x00ffcc, 1, 5000);
pointLight.position.set(0, 0, 0);
scene.add(pointLight);

// Math: Fibonacci Sphere Algorithm
function getFibonacciSpherePoints(samples, radius) {
    const points = [];
    const phi = Math.PI * (3 - Math.sqrt(5)); 
    for (let i = 0; i < samples; i++) {
        const y = 1 - (i / (samples - 1)) * 2; 
        const radiusAtY = Math.sqrt(1 - y * y); 
        const theta = phi * i; 
        const x = Math.cos(theta) * radiusAtY;
        const z = Math.sin(theta) * radiusAtY;
        points.push(new THREE.Vector3(x * radius, y * radius, z * radius));
    }
    return points;
}

const domainColors = {
    'Physics': 0x00ffcc, 'Mathematics': 0xff00ff, 'Biology': 0x00ff00,
    'Chemistry': 0xffff00, 'Logic': 0xffffff, 'Default': 0x4444ff
};

let nodesData = {};
let positions = [];
let nodeMeshes = [];
const RADIUS = 1000;

async function initGraph() {
    try {
        const loadingText = document.getElementById('loading-text');
        
        loadingText.innerText = "Fetching Nodes...";
        const nodesRes = await fetch('nodes.json');
        const rawNodes = await nodesRes.json();
        
        const nodeKeys = Object.keys(rawNodes);
        const nodeCount = nodeKeys.length;
        document.getElementById('stat-nodes').innerText = nodeCount;
        
        // Calculate Math Layout
        loadingText.innerText = `Calculating Fibonacci Layout for ${nodeCount} nodes...`;
        const calculatedPoints = getFibonacciSpherePoints(nodeCount, RADIUS);
        
        // Render Nodes using InstancedMesh for extreme performance
        const geometry = new THREE.SphereGeometry(10, 8, 8);
        const material = new THREE.MeshBasicMaterial({ color: 0xffffff });
        
        // Map nodes to coordinates
        nodeKeys.forEach((id, i) => {
            const node = rawNodes[id];
            const pos = calculatedPoints[i];
            
            nodesData[id] = {
                ...node,
                pos: pos
            };
            
            // Create a small sprite/mesh for hit testing (hover)
            const nodeMat = new THREE.MeshBasicMaterial({ 
                color: domainColors[node.dimension] || domainColors['Default']
            });
            const nodeMesh = new THREE.Mesh(geometry, nodeMat);
            nodeMesh.position.copy(pos);
            nodeMesh.userData = { id, concept: node.concept, dimension: node.dimension };
            scene.add(nodeMesh);
            nodeMeshes.push(nodeMesh);
        });

        // 2. Fetch Links
        loadingText.innerText = "Fetching Synapses (This may take a moment)...";
        const linksRes = await fetch('links.json');
        const rawLinks = await linksRes.json();
        
        document.getElementById('stat-links').innerText = rawLinks.length;
        loadingText.innerText = `Drawing ${rawLinks.length} Synapses in single WebGL call...`;

        // Render Links using LineSegments (1 Draw Call for all 60k lines)
        const lineMaterial = new THREE.LineBasicMaterial({ 
            color: 0x00ffcc, 
            transparent: true, 
            opacity: 0.15 
        });
        
        const pointsArray = [];
        rawLinks.forEach(link => {
            const sourceNode = nodesData[link.source_id];
            const targetNode = nodesData[link.target_id];
            if (sourceNode && targetNode) {
                pointsArray.push(sourceNode.pos);
                pointsArray.push(targetNode.pos);
            }
        });
        
        const lineGeometry = new THREE.BufferGeometry().setFromPoints(pointsArray);
        const lineSegments = new THREE.LineSegments(lineGeometry, lineMaterial);
        scene.add(lineSegments);

        // Remove loading overlay
        const overlay = document.getElementById('loading-overlay');
        overlay.style.opacity = '0';
        setTimeout(() => overlay.remove(), 500);

    } catch (err) {
        console.error(err);
        document.getElementById('loading-text').innerText = "ERROR: Is the Rust Engine running?";
        document.getElementById('loading-text').style.color = "red";
    }
}

// Raycaster for Hover interaction
const raycaster = new THREE.Raycaster();
const mouse = new THREE.Vector2();
const hoverInfo = document.getElementById('hover-info');
const hoverConcept = document.getElementById('hover-concept');
const hoverDim = document.getElementById('hover-dim');

window.addEventListener('mousemove', (event) => {
    mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
    mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;
    
    raycaster.setFromCamera(mouse, camera);
    const intersects = raycaster.intersectObjects(nodeMeshes);
    
    if (intersects.length > 0) {
        const obj = intersects[0].object.userData;
        hoverInfo.style.display = 'block';
        hoverInfo.style.left = event.clientX + 'px';
        hoverInfo.style.top = event.clientY + 'px';
        hoverConcept.innerText = obj.concept;
        hoverDim.innerText = obj.dimension;
        document.body.style.cursor = 'pointer';
    } else {
        hoverInfo.style.display = 'none';
        document.body.style.cursor = 'default';
    }
});

// Window Resize
window.addEventListener('resize', () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
});

// Animation Loop
function animate() {
    requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
}

// Start
initGraph();
animate();
