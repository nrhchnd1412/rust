use arc_swap::ArcSwap;
use blake3::Hasher;
use serde::{Deserialize,Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

pub type NodeId = String;
pub type Hash = u64;

#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub struct Node {
    pub id: NodeId,
    pub addr:String,
    pub weight:u32,
    pub metadata:Option<HashMap<String, String>>,
}

#[derive(Clone,Serialize,Deserialize)]
struct RingState{
    ring:Vec<(Hash,NodeId)>,
    nodes:HashMap<NodeId,Node>,
    replicas:usize
}

impl RingState{
    fn new(replicas:usize)->Self{
        Self{
            ring:Vec::new(),
            nodes: HashMap::new(),
            replicas,
        }
    }
}

#[derive(Clone)]
pub struct HashRing{
    inner: Arc<ArcSwap<RingState>>,
    default_replicas:usize,
}

#[derive(thiserror::Error,Debug)]
pub enum RingError{
    #[error("node not found")]
    NotFound,
}

impl HashRing{
    pub fn new(default_replicas:usize)->Self{
        let state = RingState::new(default_replicas);
        Self{
            inner:Arc::new(ArcSwap::new(Arc::new(state))),
            default_replicas,
        }
    }

    fn hash_key(key:&[u8])->Hash{
        let mut h = Hasher::new();
        h.update(key);
        let out = h.finalize();
        let bytes = out.as_bytes();
        let mut arr = [0u8;8];
        arr.copy_from_slice(&bytes[..8]);
        u64::from_le_bytes(arr)
    }

    fn make_virtual_names(node_id:&NodeId,replicas:usize)->impl Iterator<Item=Vec<u8>>{
        // produce distinct virtual node keys deterministically
        (0..replicas).map(move |i|{
            let s = format!("{}#{}",node_id,i);
            s.into_bytes()
        })
    }

    pub fn add_node(&self,node:Node){
        loop{
            let old = self.inner.load_full();
            let mut new_state = (*old).clone();
            if new_state.nodes.contains_key(&node.id) {
                // replace existing node's info (weight etc)
                new_state.nodes.insert(node.id.clone(),node.clone());
                // remove old vnode entries for this node
                new_state.ring.retain(|(_,nid)|nid!=&node.id);
            }else{
                new_state.nodes.insert(node.id.clone(),node.clone());
            }
            // compute replicas from weight*default
            let replicas = (self.default_replicas as u32).saturating_mul(node.weight).max(1) as usize;
            for key in Self::make_virtual_names(&node.id,replicas){
                let h = Self::hash_key(&key);
                new_state.ring.push((h,node.id.clone()));
            }
            // sort ring by hash
            new_state.ring.sort_by(|a,b|a.0.cmp(&b.0));
            let new_arc = Arc::new(new_state);
            // attempt swap - arc-swap ensures lock-free reads still see valid old state
            self.inner.compare_and_swap(&old,new_arc);
            // success (compare_and_swap returns the previous value but it's ok to loop)
            break;
        }
    }

    pub fn remove_node(&self,node_id:&NodeId)->Result<Node,RingError>{
        loop{
            let old = self.inner.load_full();
            if !old.nodes.contains_key(node_id) {
                return Err(RingError::NotFound);
            }
            let mut new_state = (*old).clone();
            let removed = new_state.nodes.remove(node_id).expect("node not found");
            new_state.ring.retain(|(_,nid)|nid!=&removed.id);
            new_state.ring.sort_by(|a,b|a.0.cmp(&b.0));
            let new_arc = Arc::new(new_state);
            self.inner.compare_and_swap(&old,new_arc);
            return Ok(removed);
        }
    }

    pub fn get_node_for_key(&self, key: &[u8]) -> Option<Node> {
        let state = self.inner.load();
        let ring = &state.ring;
        if ring.is_empty() {
            return None;
        }
        let h = Self::hash_key(key);

        match ring.binary_search_by(|(rh, _)| rh.cmp(&h)) {
            Ok(index) => {
                let node_id = &ring[index].1;
                state.nodes.get(node_id).cloned()
            }
            Err(index) => {
                let idx = if index >= ring.len() { 0 } else { index };
                let node_id = &ring[idx].1;
                state.nodes.get(node_id).cloned()
            }
        }
    }

    pub fn snapshot_nodes(&self)->Vec<Node>{
        let state = self.inner.load();
        state.nodes.values().cloned().collect()
    }

    // persisted representation of ring (nodes + replicas)
    pub fn persist_state_json(&self)->String{
        let state = self.inner.load_full();
        serde_json::to_string(&*state).expect("state should be valid JSON")
    }

    pub fn load_state_json(&self,json:&str)->Result<(),serde_json::Error>{
        let parsed: RingState = serde_json::from_str(json)?;
        self.inner.store(Arc::new(parsed));
        Ok(())
    }

}

fn main() {

    let ring = HashRing::new(10); // base replicas
    ring.add_node(Node { id: "n1".into(), addr: "10.0.0.1:9000".into(), weight: 1, metadata: None });
    ring.add_node(Node { id: "n2".into(), addr: "10.0.0.2:9000".into(), weight: 2, metadata: None });

    let key = b"user:11234";
    if let Some(node) = ring.get_node_for_key(key) {
        println!("route to {}", node.addr);
    }

    println!("{}", ring.persist_state_json());

}
