pub const KEY_BEAMLINE_AVAILABLE_SCANS: &str = "BEAMLINE_AVAILABLE_SCANS_";
pub const KEY_BEAMLINE_QUEUED_SCANS: &str = "BEAMLINE_QUEUED_SCANS_";
pub const KEY_BEAMLINE_DONE_SCANS: &str = "BEAMLINE_DONE_SCANS_";
pub const KEY_BEAMLINE_SCAN_LOGS: &str = "BEAMLINE_SCAN_LOGS_";
pub const KEY_BEAMLINE_EVENT: &str = "BEAMLINE_EVENT_";
// Header prepended (with the ':::' delimiter) to control-log messages published
// on the KEY_BEAMLINE_EVENT channel, identifying the event type.
pub const KEY_CONTROLS_EVENT: &str = "CONTROLS_EVENT";
pub const KEY_TASK_QUEUE_WAITING: &str = "TASK_QUEUE_WAITING_";
pub const KEY_TASK_QUEUE_PROCESSING: &str = "TASK_QUEUE_PROCESSING_";
pub const KEY_TASK_QUEUE_DONE: &str = "TASK_QUEUE_DONE_";
pub const KEY_WORKER_HEARTBEAT: &str = "WORKER_HEARTBEAT_";
// Redis hash of monitored EPICS PV values; suffixed with the beamline_id.
pub const KEY_IOC_MONITOR: &str = "IOC_MONITOR_";
// Live XRF map data streamed from XRF-Maps. One Redis hash per dataset, keyed by
// "<row>_<col>", plus a matching pub/sub channel of the same name for live updates.
pub const KEY_XRF_LIVE_MAP: &str = "XRF_LIVE_MAP_";

// REDIS Channels
pub const BEAMLINE_CONTROLS: &str = "Beamline_Controls";

pub const STR_QUEUED: &str = "QUEUED";
pub const STR_PROCESSING: &str = "PROCESSING";
pub const STR_DONE: &str = "DONE";

pub const STR_ADMIN: &str = "Admin";
pub const STR_STAFF: &str = "Staff";
pub const STR_USER: &str = "User";