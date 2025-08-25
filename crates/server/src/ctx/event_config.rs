use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventType {
    BlockAdded,
    VirtualChainChanged,
    FinalityConflict,
    FinalityConflictResolved,
    UtxosChanged,
    SinkBlueScoreChanged,
    VirtualDaaScoreChanged,
    PruningPointUtxoSetOverride,
    NewBlockTemplate,
}

impl FromStr for EventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "block-added" => Ok(EventType::BlockAdded),
            "virtual-chain-changed" => Ok(EventType::VirtualChainChanged),
            "finality-conflict" => Ok(EventType::FinalityConflict),
            "finality-conflict-resolved" => Ok(EventType::FinalityConflictResolved),
            "utxos-changed" => Ok(EventType::UtxosChanged),
            "sink-blue-score-changed" => Ok(EventType::SinkBlueScoreChanged),
            "virtual-daa-score-changed" => Ok(EventType::VirtualDaaScoreChanged),
            "pruning-point-utxo-set-override" => Ok(EventType::PruningPointUtxoSetOverride),
            "new-block-template" => Ok(EventType::NewBlockTemplate),
            _ => Err(format!("Unknown event type: {}", s)),
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::BlockAdded => write!(f, "block-added"),
            EventType::VirtualChainChanged => write!(f, "virtual-chain-changed"),
            EventType::FinalityConflict => write!(f, "finality-conflict"),
            EventType::FinalityConflictResolved => write!(f, "finality-conflict-resolved"),
            EventType::UtxosChanged => write!(f, "utxos-changed"),
            EventType::SinkBlueScoreChanged => write!(f, "sink-blue-score-changed"),
            EventType::VirtualDaaScoreChanged => write!(f, "virtual-daa-score-changed"),
            EventType::PruningPointUtxoSetOverride => write!(f, "pruning-point-utxo-set-override"),
            EventType::NewBlockTemplate => write!(f, "new-block-template"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    /// 启用的区块链事件类型
    #[serde(default = "default_enabled_events")]
    pub enabled_events: Vec<String>,
    
    /// 事件处理策略
    #[serde(default)]
    pub event_strategy: EventStrategy,
    
    /// 事件缓冲区大小
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    
    /// 是否启用事件去重
    #[serde(default = "default_deduplication")]
    pub enable_deduplication: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventStrategy {
    /// Process all events in real-time
    RealTime,
    /// Process events in batches to reduce database writes
    Batch {
        batch_size: usize,
        batch_timeout_ms: u64,
    },
    /// Process events by priority (high, medium, low)
    Priority {
        high_priority: Vec<String>,
        medium_priority: Vec<String>,
        low_priority: Vec<String>,
    },
}

impl Default for EventStrategy {
    fn default() -> Self {
        EventStrategy::RealTime
    }
}

fn default_enabled_events() -> Vec<String> {
    vec![
        "block-added".to_string(),
        "utxos-changed".to_string(),
        "virtual-chain-changed".to_string(),
    ]
}

fn default_buffer_size() -> usize {
    1000
}

fn default_deduplication() -> bool {
    true
}

impl EventConfig {
    /// Parse configured event type strings into EventType enums
    pub fn parse_event_types(&self) -> Result<HashSet<EventType>, String> {
        let mut event_types = HashSet::new();
        
        for event_str in &self.enabled_events {
            let event_type = EventType::from_str(event_str)
                .map_err(|e| format!("Invalid event type '{}': {}", event_str, e))?;
            event_types.insert(event_type);
        }
        
        Ok(event_types)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check if event types are valid
        self.parse_event_types()?;
        
        // Check batch processing configuration
        if let EventStrategy::Batch { batch_size, batch_timeout_ms } = &self.event_strategy {
            if *batch_size == 0 {
                return Err("Batch size must be greater than 0".to_string());
            }
            if *batch_timeout_ms == 0 {
                return Err("Batch timeout must be greater than 0".to_string());
            }
        }
        
        // Check priority configuration
        if let EventStrategy::Priority { high_priority, medium_priority, low_priority } = &self.event_strategy {
            let all_events: HashSet<_> = high_priority.iter()
                .chain(medium_priority.iter())
                .chain(low_priority.iter())
                .collect();
            
            for event_str in all_events {
                EventType::from_str(event_str)
                    .map_err(|e| format!("Invalid priority event type '{}': {}", event_str, e))?;
            }
        }
        
        Ok(())
    }
    
    /// Get all available event types
    pub fn get_all_event_types() -> Vec<EventType> {
        vec![
            EventType::BlockAdded,
            EventType::VirtualChainChanged,
            EventType::FinalityConflict,
            EventType::FinalityConflictResolved,
            EventType::UtxosChanged,
            EventType::SinkBlueScoreChanged,
            EventType::VirtualDaaScoreChanged,
            EventType::PruningPointUtxoSetOverride,
            EventType::NewBlockTemplate,
        ]
    }
}
