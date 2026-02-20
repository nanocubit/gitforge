use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::broadcast;

pub const SYSTEM_EVENT_SCHEMA_VERSION: u16 = 1;

/// Compatibility rules for `SystemEvent`:
/// - Major event schema version must match exactly.
/// - New event variants are additive within the same major version.
/// - Existing variant field names and semantics are backwards-compatible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedSystemEvent {
    pub schema_version: u16,
    pub event: SystemEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SystemEvent {
    GoalCreated { goal_id: String, task: String },
    GoalCancelled { goal_id: String },
    GoalStatusChanged { goal_id: String, status: GoalStatus },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Error)]
pub enum AntError {
    #[error("goal already exists: {0}")]
    GoalAlreadyExists(String),
    #[error("goal not found: {0}")]
    GoalNotFound(String),
}

#[derive(Clone)]
pub struct AntEngine {
    bus: broadcast::Sender<VersionedSystemEvent>,
    goals: Arc<Mutex<HashMap<String, GoalStatus>>>,
}

impl AntEngine {
    pub fn new() -> Self {
        let (bus, _) = broadcast::channel(1024);
        Self {
            bus,
            goals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_goal(
        &self,
        goal_id: impl Into<String>,
        task: impl Into<String>,
    ) -> Result<(), AntError> {
        let goal_id = goal_id.into();
        let task = task.into();

        let mut goals = self.goals.lock().expect("goals lock poisoned");
        if goals.contains_key(&goal_id) {
            return Err(AntError::GoalAlreadyExists(goal_id));
        }

        goals.insert(goal_id.clone(), GoalStatus::Pending);
        drop(goals);

        self.emit(SystemEvent::GoalCreated {
            goal_id: goal_id.clone(),
            task,
        });
        self.emit(SystemEvent::GoalStatusChanged {
            goal_id,
            status: GoalStatus::Pending,
        });

        Ok(())
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<VersionedSystemEvent> {
        self.bus.subscribe()
    }

    pub fn get_goal_status(&self, goal_id: &str) -> Result<GoalStatus, AntError> {
        let goals = self.goals.lock().expect("goals lock poisoned");
        goals
            .get(goal_id)
            .cloned()
            .ok_or_else(|| AntError::GoalNotFound(goal_id.to_string()))
    }

    pub fn cancel_goal(&self, goal_id: &str) -> Result<(), AntError> {
        let mut goals = self.goals.lock().expect("goals lock poisoned");
        let status = goals
            .get_mut(goal_id)
            .ok_or_else(|| AntError::GoalNotFound(goal_id.to_string()))?;

        *status = GoalStatus::Cancelled;
        drop(goals);

        self.emit(SystemEvent::GoalCancelled {
            goal_id: goal_id.to_string(),
        });
        self.emit(SystemEvent::GoalStatusChanged {
            goal_id: goal_id.to_string(),
            status: GoalStatus::Cancelled,
        });

        Ok(())
    }

    fn emit(&self, event: SystemEvent) {
        let _ = self.bus.send(VersionedSystemEvent {
            schema_version: SYSTEM_EVENT_SCHEMA_VERSION,
            event,
        });
    }
}

impl Default for AntEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_get_goal_status() {
        let engine = AntEngine::new();
        engine
            .create_goal("G-1", "Analyze repository")
            .expect("goal created");

        let status = engine.get_goal_status("G-1").expect("status exists");
        assert_eq!(status, GoalStatus::Pending);
    }

    #[test]
    fn cancel_goal_changes_status() {
        let engine = AntEngine::new();
        engine
            .create_goal("G-2", "Refactor module")
            .expect("goal created");

        engine.cancel_goal("G-2").expect("goal cancelled");
        let status = engine.get_goal_status("G-2").expect("status exists");

        assert_eq!(status, GoalStatus::Cancelled);
    }

    #[test]
    fn subscribe_events_receives_v1_event() {
        let engine = AntEngine::new();
        let mut rx = engine.subscribe_events();

        engine
            .create_goal("G-3", "Plan tasks")
            .expect("goal created");

        let event = rx.try_recv().expect("event received");
        assert_eq!(event.schema_version, SYSTEM_EVENT_SCHEMA_VERSION);
    }
}
