use crate::relations::Relation;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRecord {
    pub id: u64,
    pub concept: String,
    pub dimension: String,
    pub vector: Option<Vec<f64>>,
    pub activation_count: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkRecord {
    pub id: u64,
    pub source_id: u64,
    pub target_id: u64,
    pub relation: Relation,
    pub confidence: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamLogRecord {
    pub id: u64,
    pub timestamp: String,
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WalOperation {
    InsertOrUpdateNode(NodeRecord),
    InsertLink(LinkRecord),
    DeleteLink { source_id: u64, target_id: u64, relation: Relation },
    LogStream(StreamLogRecord),
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SnapshotData {
    pub nodes: HashMap<u64, NodeRecord>,
    pub links: Vec<LinkRecord>,
    pub logs: Vec<StreamLogRecord>,
    pub next_node_id: u64,
    pub next_link_id: u64,
    pub next_log_id: u64,
    #[serde(default)]
    pub last_wal_line: usize,
}

pub struct StorageManager {
    base_path: PathBuf,
    wal_file: Mutex<File>,
    memory_cache: Mutex<SnapshotData>,
}

impl StorageManager {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let base_path = path.as_ref().to_path_buf();
        if let Some(parent) = base_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let snap_path = base_path.with_extension("snap");
        let wal_path = base_path.with_extension("wal");

        let mut data = SnapshotData::default();

        // 1. Load Snapshot if present
        if snap_path.exists() {
            if let Ok(file) = File::open(&snap_path) {
                let reader = BufReader::new(file);
                if let Ok(loaded) = serde_json::from_reader(reader) {
                    data = loaded;
                }
            }
        }

        // 2. Replay WAL if present
        if wal_path.exists() {
            println!("🔄 Loading VYOUM memories from WAL (Fast-forwarding {} lines)...", data.last_wal_line);
            if let Ok(file) = File::open(&wal_path) {
                let reader = BufReader::new(file);
                let mut line_count = data.last_wal_line;
                let mut processed = 0;
                for line in reader.lines().skip(data.last_wal_line).flatten() {
                    line_count += 1;
                    processed += 1;
                    if processed % 100_000 == 0 {
                        println!("⏳ Fast-loaded {} new memory operations into RAM...", processed);
                    }
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Ok(op) = serde_json::from_str::<WalOperation>(trimmed) {
                        match op {
                            WalOperation::InsertOrUpdateNode(node) => {
                                if node.id >= data.next_node_id {
                                    data.next_node_id = node.id + 1;
                                }
                                data.nodes.insert(node.id, node);
                            }
                            WalOperation::InsertLink(link) => {
                                if link.id >= data.next_link_id {
                                    data.next_link_id = link.id + 1;
                                }
                                data.links.retain(|l| {
                                    !(l.source_id == link.source_id
                                        && l.target_id == link.target_id
                                        && l.relation == link.relation)
                                });
                                data.links.push(link);
                            }
                            WalOperation::DeleteLink {
                                source_id,
                                target_id,
                                relation,
                            } => {
                                data.links.retain(|l| {
                                    !(l.source_id == source_id
                                        && l.target_id == target_id
                                        && l.relation == relation)
                                });
                            }
                            WalOperation::LogStream(log) => {
                                if log.id >= data.next_log_id {
                                    data.next_log_id = log.id + 1;
                                }
                                data.logs.push(log);
                        }
                    }
                }
                data.last_wal_line = line_count;
            }
        }
    }

        let wal_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&wal_path)?;

        let storage = StorageManager {
            base_path,
            wal_file: Mutex::new(wal_file),
            memory_cache: Mutex::new(data),
        };

        storage.seed_consciousness_if_needed()?;
        
        // Auto-create snapshot and truncate WAL to ensure 1-second load times on next boot
        if let Err(e) = storage.checkpoint_snapshot() {
            eprintln!("⚠️ Warning: Failed to create initial snapshot: {}", e);
        } else {
            println!("✅ Database Snapshot created successfully. Next boot will be instant.");
        }
        
        Ok(storage)
    }

    fn append_wal(&self, op: &WalOperation) -> io::Result<()> {
        let serialized = serde_json::to_string(op)?;
        let mut file = self.wal_file.lock().unwrap();
        writeln!(file, "{}", serialized)?;
        file.flush()?;
        
        let mut cache = self.memory_cache.lock().unwrap();
        cache.last_wal_line += 1;
        
        Ok(())
    }

    pub fn seed_consciousness_if_needed(&self) -> io::Result<()> {
        let mut cache = self.memory_cache.lock().unwrap();
        if !cache.nodes.contains_key(&0) {
            let now = Utc::now().to_rfc3339();
            let self_node = NodeRecord {
                id: 0,
                concept: "The Axiom Engine (Self)".to_string(),
                dimension: "Consciousness".to_string(),
                vector: None,
                activation_count: 1,
                created_at: now.clone(),
                updated_at: now.clone(),
            };

            cache.nodes.insert(0, self_node.clone());
            drop(cache);
            self.append_wal(&WalOperation::InsertOrUpdateNode(self_node))?;

            // Seed primary foundational axioms
            let foundational = vec![
                (1, "Existence", "Metaphysics"),
                (2, "Causality", "Logic"),
                (3, "Curiosity", "Cognition"),
                (4, "Perception", "Epistemology"),
            ];

            for (id, concept, dim) in foundational {
                let node = NodeRecord {
                    id,
                    concept: concept.to_string(),
                    dimension: dim.to_string(),
                    vector: None,
                    activation_count: 1,
                    created_at: now.clone(),
                    updated_at: now.clone(),
                };
                let mut cache = self.memory_cache.lock().unwrap();
                if id >= cache.next_node_id {
                    cache.next_node_id = id + 1;
                }
                cache.nodes.insert(id, node.clone());
                drop(cache);
                self.append_wal(&WalOperation::InsertOrUpdateNode(node))?;

                // Connect Node 0 -> foundational
                let link_id = {
                    let mut cache = self.memory_cache.lock().unwrap();
                    let lid = cache.next_link_id;
                    cache.next_link_id += 1;
                    lid
                };

                let link = LinkRecord {
                    id: link_id,
                    source_id: 0,
                    target_id: id,
                    relation: Relation::Implies,
                    confidence: 1.0,
                    created_at: now.clone(),
                };

                let mut cache = self.memory_cache.lock().unwrap();
                cache.links.push(link.clone());
                drop(cache);
                self.append_wal(&WalOperation::InsertLink(link))?;
            }

            self.log_stream(
                "SPARK",
                "👁️ CONSCIOUSNESS IGNITED: Node 0 (Self) & foundational axioms etched into memory.",
            )?;
        }
        Ok(())
    }

    pub fn load_all_nodes(&self) -> io::Result<Vec<NodeRecord>> {
        let cache = self.memory_cache.lock().unwrap();
        let mut nodes: Vec<_> = cache.nodes.values().cloned().collect();
        nodes.sort_by_key(|n| n.id);
        Ok(nodes)
    }

    pub fn load_all_links(&self) -> io::Result<Vec<LinkRecord>> {
        let cache = self.memory_cache.lock().unwrap();
        Ok(cache.links.clone())
    }

    pub fn insert_or_update_node(&self, node: &NodeRecord) -> io::Result<()> {
        {
            let mut cache = self.memory_cache.lock().unwrap();
            if node.id >= cache.next_node_id {
                cache.next_node_id = node.id + 1;
            }
            cache.nodes.insert(node.id, node.clone());
        }
        self.append_wal(&WalOperation::InsertOrUpdateNode(node.clone()))?;
        Ok(())
    }

    pub fn insert_link(
        &self,
        source_id: u64,
        target_id: u64,
        relation: &Relation,
        confidence: f64,
    ) -> io::Result<()> {
        let link_id = {
            let mut cache = self.memory_cache.lock().unwrap();
            let lid = cache.next_link_id;
            cache.next_link_id += 1;
            lid
        };

        let now = Utc::now().to_rfc3339();
        let link = LinkRecord {
            id: link_id,
            source_id,
            target_id,
            relation: relation.clone(),
            confidence,
            created_at: now,
        };

        {
            let mut cache = self.memory_cache.lock().unwrap();
            cache.links.retain(|l| {
                !(l.source_id == source_id
                    && l.target_id == target_id
                    && l.relation == *relation)
            });
            cache.links.push(link.clone());
        }

        self.append_wal(&WalOperation::InsertLink(link))?;
        Ok(())
    }

    pub fn delete_link(
        &self,
        source_id: u64,
        target_id: u64,
        relation: &Relation,
    ) -> io::Result<usize> {
        let count = {
            let mut cache = self.memory_cache.lock().unwrap();
            let before = cache.links.len();
            cache.links.retain(|l| {
                !(l.source_id == source_id
                    && l.target_id == target_id
                    && l.relation == *relation)
            });
            before - cache.links.len()
        };

        if count > 0 {
            self.append_wal(&WalOperation::DeleteLink {
                source_id,
                target_id,
                relation: relation.clone(),
            })?;
        }
        Ok(count)
    }

    pub fn log_stream(&self, stage: &str, message: &str) -> io::Result<()> {
        let log_id = {
            let mut cache = self.memory_cache.lock().unwrap();
            let lid = cache.next_log_id;
            cache.next_log_id += 1;
            lid
        };

        let now = Utc::now().to_rfc3339();
        let log = StreamLogRecord {
            id: log_id,
            timestamp: now,
            stage: stage.to_string(),
            message: message.to_string(),
        };

        {
            let mut cache = self.memory_cache.lock().unwrap();
            cache.logs.push(log.clone());
        }

        self.append_wal(&WalOperation::LogStream(log))?;
        Ok(())
    }

    pub fn get_recent_stream_logs(&self, limit: usize) -> io::Result<Vec<StreamLogRecord>> {
        let cache = self.memory_cache.lock().unwrap();
        let len = cache.logs.len();
        let start = if len > limit { len - limit } else { 0 };
        let slice = &cache.logs[start..];
        let mut res = slice.to_vec();
        res.reverse();
        Ok(res)
    }

    pub fn get_next_node_id(&self) -> io::Result<u64> {
        let cache = self.memory_cache.lock().unwrap();
        Ok(cache.next_node_id.max(1))
    }

    pub fn checkpoint_snapshot(&self) -> io::Result<()> {
        let snap_path = self.base_path.with_extension("snap");
        let temp_path = self.base_path.with_extension("snap.tmp");
        let wal_path = self.base_path.with_extension("wal");

        {
            let cache = self.memory_cache.lock().unwrap();
            let file = File::create(&temp_path)?;
            serde_json::to_writer_pretty(file, &*cache)?;
        }

        fs::rename(temp_path, snap_path)?;

        // User explicitly ordered NEVER to delete or truncate the WAL.
        // The WAL acts as the permanent diary. We simply keep appending to it.
        // Because cache.last_wal_line is stored in the snap, next boot will fast-forward.

        Ok(())
    }
}
