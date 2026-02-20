# ant-core

`ant-core` defines the stable ANT engine contract used by GitForge.

## Stable API (Stage A)

- `create_goal(goal_id, task)`
- `subscribe_events()`
- `get_goal_status(goal_id)`
- `cancel_goal(goal_id)`

## SystemEvent versioning

Current schema version: **v1** (`SYSTEM_EVENT_SCHEMA_VERSION = 1`).

Compatibility rules:

1. Major schema version must match exactly between producer and consumer.
2. Within the same major version, event variants may only be added (non-breaking additive change).
3. Existing variant names/fields/semantics are backwards-compatible within the same major version.
4. Any breaking change requires incrementing the major schema version.
