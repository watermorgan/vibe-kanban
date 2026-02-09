use axum::{
    Json,
    extract::State,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use db::models::{
    scratch::{
        CreateScratch, Scratch, ScratchPayload, ScratchType,
        StarbusGlobalStateData, StarbusHistoryEntry, StarbusNextAction, StarbusTaskStateData,
        UpdateScratch,
    },
    task::{CreateTask, Task, TaskStatus},
};
use deployment::Deployment;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utils::response::ApiResponse;
use uuid::Uuid;

use crate::{DeploymentImpl, error::ApiError};

const STARBUS_GLOBAL_ID_STR: &str = "00000000-0000-0000-0000-000000000000";

fn starbus_global_id() -> Uuid {
    Uuid::parse_str(STARBUS_GLOBAL_ID_STR).expect("invalid STARBUS_GLOBAL_ID")
}

fn normalize_status(status: &str) -> String {
    status.trim().to_uppercase()
}

fn is_terminal_status(status: &str) -> bool {
    matches!(
        normalize_status(status).as_str(),
        "DONE" | "FAILED" | "CANCELLED"
    )
}

fn status_priority(status: &str) -> u8 {
    match normalize_status(status).as_str() {
        "EXECUTING" => 0,
        "VERIFYING" => 1,
        "AUDITING" => 2,
        "DESIGNING" => 3,
        "QUEUED" => 4,
        "BLOCKED_HUMAN" => 5,
        _ => 6,
    }
}

fn map_starbus_to_task_status(status: &str) -> TaskStatus {
    match normalize_status(status).as_str() {
        "QUEUED" => TaskStatus::Todo,
        "DESIGNING" | "EXECUTING" => TaskStatus::InProgress,
        "AUDITING" | "VERIFYING" | "BLOCKED_HUMAN" => TaskStatus::InReview,
        "DONE" => TaskStatus::Done,
        "FAILED" => TaskStatus::Cancelled,
        _ => TaskStatus::InProgress,
    }
}

fn is_valid_transition(from_status: &str, to_status: &str) -> bool {
    let from = normalize_status(from_status);
    let to = normalize_status(to_status);
    if to == "BLOCKED_HUMAN" {
        return true;
    }
    match (from.as_str(), to.as_str()) {
        ("QUEUED", "DESIGNING") => true,
        ("DESIGNING", "AUDITING") => true,
        ("AUDITING", "EXECUTING") => true,
        ("AUDITING", "DESIGNING") => true,
        ("EXECUTING", "VERIFYING") => true,
        ("VERIFYING", "DONE") => true,
        ("VERIFYING", "EXECUTING") => true,
        ("BLOCKED_HUMAN", "DESIGNING") => true,
        ("BLOCKED_HUMAN", "AUDITING") => true,
        ("BLOCKED_HUMAN", "EXECUTING") => true,
        ("BLOCKED_HUMAN", "VERIFYING") => true,
        _ => false,
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct StarbusStateResponse {
    pub active_task_id: Option<Uuid>,
    pub tasks: Vec<StarbusTaskStateData>,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub struct StarbusIntakeRequest {
    pub project_id: Uuid,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub acceptance: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub domain_roles: Vec<String>,
    #[serde(default)]
    pub include_recommended_deps: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub set_active: bool,
    #[serde(default)]
    pub default_actor: Option<String>,
    #[serde(default)]
    pub default_role: Option<String>,
}

fn validate_intake(payload: &StarbusIntakeRequest) -> StarbusPreflightResponse {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    if payload.title.trim().is_empty() {
        errors.push("title is required".to_string());
    }
    if payload.priority.is_none() {
        errors.push("priority is required (P0/P1/P2)".to_string());
    } else if !matches!(
        payload.priority.as_deref(),
        Some("P0") | Some("P1") | Some("P2")
    ) {
        errors.push("priority must be P0/P1/P2".to_string());
    }
    if payload.domain_roles.len() > 4 {
        errors.push("domain_roles overflow (>4)".to_string());
    }
    if payload.include_recommended_deps.is_none() {
        errors.push("include_recommended_deps is required".to_string());
    }
    if payload.description.as_deref().unwrap_or("").trim().is_empty() {
        warnings.push("description is empty".to_string());
    }

    StarbusPreflightResponse {
        ok: errors.is_empty(),
        errors,
        warnings,
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct StarbusPreflightResponse {
    pub ok: bool,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct StarbusNextActionUpdate {
    pub task_id: Uuid,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub next_action: Option<StarbusNextAction>,
    #[serde(default)]
    pub set_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct StarbusDecisionResolveRequest {
    pub task_id: Uuid,
    pub decision_id: String,
    pub resolution: String,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub resume_status: Option<String>,
    #[serde(default)]
    pub next_action: Option<StarbusNextAction>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct StarbusTransitionRequest {
    pub task_id: Uuid,
    pub status: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub next_action: Option<StarbusNextAction>,
    #[serde(default)]
    pub set_active: Option<bool>,
}

#[derive(Debug, Clone)]
struct StarbusTaskWithMeta {
    state: StarbusTaskStateData,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn select_active_task_id(
    desired_active_task_id: Option<Uuid>,
    tasks: &[StarbusTaskWithMeta],
) -> Option<Uuid> {
    if let Some(task_id) = desired_active_task_id {
        if tasks.iter().any(|t| t.state.task_id == task_id && !is_terminal_status(&t.state.status))
        {
            return Some(task_id);
        }
    }

    let mut candidates = tasks
        .iter()
        .filter(|t| !is_terminal_status(&t.state.status))
        .collect::<Vec<_>>();

    candidates.sort_by(|a, b| {
        let priority_cmp = status_priority(&a.state.status).cmp(&status_priority(&b.state.status));
        if priority_cmp != std::cmp::Ordering::Equal {
            return priority_cmp;
        }

        let updated_cmp = b.updated_at.cmp(&a.updated_at);
        if updated_cmp != std::cmp::Ordering::Equal {
            return updated_cmp;
        }

        let created_cmp = b.created_at.cmp(&a.created_at);
        if created_cmp != std::cmp::Ordering::Equal {
            return created_cmp;
        }

        a.state.task_id.cmp(&b.state.task_id)
    });

    candidates.first().map(|t| t.state.task_id)
}

async fn get_global_state(
    deployment: &DeploymentImpl,
) -> Result<StarbusGlobalStateData, ApiError> {
    let global = Scratch::find_by_id(
        &deployment.db().pool,
        starbus_global_id(),
        &ScratchType::StarbusGlobalState,
    )
    .await?;
    Ok(match global {
        Some(s) => match s.payload {
            ScratchPayload::StarbusGlobalState(data) => data,
            _ => StarbusGlobalStateData { active_task_id: None },
        },
        None => StarbusGlobalStateData { active_task_id: None },
    })
}

async fn upsert_global_state(
    deployment: &DeploymentImpl,
    data: StarbusGlobalStateData,
) -> Result<(), ApiError> {
    let payload = ScratchPayload::StarbusGlobalState(data);
    let update = UpdateScratch { payload };
    Scratch::update(
        &deployment.db().pool,
        starbus_global_id(),
        &ScratchType::StarbusGlobalState,
        &update,
    )
    .await?;
    Ok(())
}

async fn list_starbus_tasks_with_meta(
    deployment: &DeploymentImpl,
) -> Result<Vec<StarbusTaskWithMeta>, ApiError> {
    let all = Scratch::find_all(&deployment.db().pool).await?;
    let tasks = all
        .into_iter()
        .filter_map(|scratch| match scratch.payload {
            ScratchPayload::StarbusTaskState(data) => Some(StarbusTaskWithMeta {
                state: data,
                created_at: scratch.created_at,
                updated_at: scratch.updated_at,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    Ok(tasks)
}

async fn get_starbus_task(
    deployment: &DeploymentImpl,
    task_id: Uuid,
) -> Result<StarbusTaskStateData, ApiError> {
    let scratch = Scratch::find_by_id(
        &deployment.db().pool,
        task_id,
        &ScratchType::StarbusTaskState,
    )
    .await?
    .ok_or_else(|| ApiError::BadRequest("Starbus task not found".to_string()))?;

    match scratch.payload {
        ScratchPayload::StarbusTaskState(data) => Ok(data),
        _ => Err(ApiError::BadRequest(
            "Scratch payload type mismatch".to_string(),
        )),
    }
}

pub async fn get_starbus_state(
    State(deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<StarbusStateResponse>>, ApiError> {
    let global = get_global_state(&deployment).await?;
    let tasks_with_meta = list_starbus_tasks_with_meta(&deployment).await?;
    let resolved_active_task_id =
        select_active_task_id(global.active_task_id, &tasks_with_meta);
    if resolved_active_task_id != global.active_task_id {
        upsert_global_state(
            &deployment,
            StarbusGlobalStateData {
                active_task_id: resolved_active_task_id,
            },
        )
        .await?;
    }
    let tasks = tasks_with_meta.into_iter().map(|t| t.state).collect();
    Ok(ResponseJson(ApiResponse::success(StarbusStateResponse {
        active_task_id: resolved_active_task_id,
        tasks,
    })))
}

pub async fn intake_preflight(
    State(_deployment): State<DeploymentImpl>,
    Json(payload): Json<StarbusIntakeRequest>,
) -> Result<ResponseJson<ApiResponse<StarbusPreflightResponse>>, ApiError> {
    Ok(ResponseJson(ApiResponse::success(validate_intake(&payload))))
}

pub async fn intake_create(
    State(deployment): State<DeploymentImpl>,
    Json(payload): Json<StarbusIntakeRequest>,
) -> Result<ResponseJson<ApiResponse<StarbusTaskStateData>>, ApiError> {
    let preflight = validate_intake(&payload);
    if !preflight.ok {
        return Err(ApiError::BadRequest(format!(
            "Preflight failed: {}",
            preflight.errors.join("; ")
        )));
    }

    let task_id = Uuid::new_v4();
    let task_status = map_starbus_to_task_status("QUEUED");
    let create_task = CreateTask {
        project_id: payload.project_id,
        title: payload.title.clone(),
        description: payload.description.clone(),
        status: Some(task_status),
        parent_workspace_id: None,
        image_ids: None,
    };
    let _ = Task::create(&deployment.db().pool, &create_task, task_id).await?;

    let next_action = payload.default_actor.as_ref().map(|actor| StarbusNextAction {
        actor: actor.clone(),
        role: payload.default_role.clone().unwrap_or_else(|| "role-product-manager".to_string()),
        action: "Clarify/plan".to_string(),
        inputs: Vec::new(),
        outputs: Vec::new(),
    });

    let task_state = StarbusTaskStateData {
        task_id,
        title: payload.title,
        status: "QUEUED".to_string(),
        active_actor: None,
        active_role: None,
        next_action,
        decision_requests: Vec::new(),
        history: Vec::new(),
        step_count: 0,
        gate: Some("Gate0".to_string()),
        tags: payload.tags,
    };

    let create = CreateScratch {
        payload: ScratchPayload::StarbusTaskState(task_state.clone()),
    };
    let _ = Scratch::create(&deployment.db().pool, task_id, &create).await?;

    let global = get_global_state(&deployment).await?;
    let desired_active_task_id = if payload.set_active {
        Some(task_id)
    } else {
        global.active_task_id
    };
    let tasks_with_meta = list_starbus_tasks_with_meta(&deployment).await?;
    let resolved_active_task_id =
        select_active_task_id(desired_active_task_id, &tasks_with_meta);
    if resolved_active_task_id != global.active_task_id {
        upsert_global_state(
            &deployment,
            StarbusGlobalStateData {
                active_task_id: resolved_active_task_id,
            },
        )
        .await?;
    }

    Ok(ResponseJson(ApiResponse::success(task_state)))
}

pub async fn update_next_action(
    State(deployment): State<DeploymentImpl>,
    Json(update): Json<StarbusNextActionUpdate>,
) -> Result<ResponseJson<ApiResponse<StarbusTaskStateData>>, ApiError> {
    let mut state = get_starbus_task(&deployment, update.task_id).await?;
    if let Some(status) = update.status.as_ref() {
        state.status = normalize_status(status);
        let mapped = map_starbus_to_task_status(&state.status);
        let _ = Task::update_status(&deployment.db().pool, state.task_id, mapped).await;
    }
    if let Some(next_action) = update.next_action {
        state.next_action = Some(next_action);
    }

    let update_payload = UpdateScratch {
        payload: ScratchPayload::StarbusTaskState(state.clone()),
    };
    Scratch::update(
        &deployment.db().pool,
        state.task_id,
        &ScratchType::StarbusTaskState,
        &update_payload,
    )
    .await?;

    let global = get_global_state(&deployment).await?;
    let desired_active_task_id = if update.set_active.unwrap_or(false) {
        Some(state.task_id)
    } else {
        global.active_task_id
    };
    let tasks_with_meta = list_starbus_tasks_with_meta(&deployment).await?;
    let resolved_active_task_id =
        select_active_task_id(desired_active_task_id, &tasks_with_meta);
    if resolved_active_task_id != global.active_task_id {
        upsert_global_state(
            &deployment,
            StarbusGlobalStateData {
                active_task_id: resolved_active_task_id,
            },
        )
        .await?;
    }

    Ok(ResponseJson(ApiResponse::success(state)))
}

pub async fn transition_state(
    State(deployment): State<DeploymentImpl>,
    Json(update): Json<StarbusTransitionRequest>,
) -> Result<ResponseJson<ApiResponse<StarbusTaskStateData>>, ApiError> {
    let mut state = get_starbus_task(&deployment, update.task_id).await?;
    let target = normalize_status(&update.status);
    if !is_valid_transition(&state.status, &target) {
        return Err(ApiError::BadRequest(format!(
            "Invalid transition: {} -> {}",
            state.status, target
        )));
    }
    let history = StarbusHistoryEntry {
        ts: chrono::Utc::now().to_rfc3339(),
        from_status: Some(state.status.clone()),
        to_status: Some(target.clone()),
        actor: None,
        note: update.note.clone(),
    };
    state.history.push(history);
    state.status = target;
    if let Some(next_action) = update.next_action {
        state.next_action = Some(next_action);
    }

    let mapped = map_starbus_to_task_status(&state.status);
    let _ = Task::update_status(&deployment.db().pool, state.task_id, mapped).await;

    let update_payload = UpdateScratch {
        payload: ScratchPayload::StarbusTaskState(state.clone()),
    };
    Scratch::update(
        &deployment.db().pool,
        state.task_id,
        &ScratchType::StarbusTaskState,
        &update_payload,
    )
    .await?;

    let global = get_global_state(&deployment).await?;
    let desired_active_task_id = if update.set_active.unwrap_or(false) {
        Some(state.task_id)
    } else {
        global.active_task_id
    };
    let tasks_with_meta = list_starbus_tasks_with_meta(&deployment).await?;
    let resolved_active_task_id =
        select_active_task_id(desired_active_task_id, &tasks_with_meta);
    if resolved_active_task_id != global.active_task_id {
        upsert_global_state(
            &deployment,
            StarbusGlobalStateData {
                active_task_id: resolved_active_task_id,
            },
        )
        .await?;
    }

    Ok(ResponseJson(ApiResponse::success(state)))
}

pub async fn resolve_decision(
    State(deployment): State<DeploymentImpl>,
    Json(req): Json<StarbusDecisionResolveRequest>,
) -> Result<ResponseJson<ApiResponse<StarbusTaskStateData>>, ApiError> {
    let mut state = get_starbus_task(&deployment, req.task_id).await?;
    let mut found = false;
    for item in state.decision_requests.iter_mut() {
        if item.id == req.decision_id {
            item.resolution = Some(req.resolution.clone());
            item.resolved_at = req.resolved_at.clone().or_else(|| {
                Some(chrono::Utc::now().to_rfc3339())
            });
            found = true;
        }
    }
    if !found {
        return Err(ApiError::BadRequest("Decision request not found".to_string()));
    }

    if let Some(resume_status) = req.resume_status {
        let target = normalize_status(&resume_status);
        if is_valid_transition(&state.status, &target) {
            state.status = target;
        }
    }
    if let Some(next_action) = req.next_action {
        state.next_action = Some(next_action);
    }

    let update_payload = UpdateScratch {
        payload: ScratchPayload::StarbusTaskState(state.clone()),
    };
    Scratch::update(
        &deployment.db().pool,
        state.task_id,
        &ScratchType::StarbusTaskState,
        &update_payload,
    )
    .await?;

    let global = get_global_state(&deployment).await?;
    let tasks_with_meta = list_starbus_tasks_with_meta(&deployment).await?;
    let resolved_active_task_id =
        select_active_task_id(global.active_task_id, &tasks_with_meta);
    if resolved_active_task_id != global.active_task_id {
        upsert_global_state(
            &deployment,
            StarbusGlobalStateData {
                active_task_id: resolved_active_task_id,
            },
        )
        .await?;
    }

    Ok(ResponseJson(ApiResponse::success(state)))
}

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/starbus/state", get(get_starbus_state))
        .route("/starbus/intake/preflight", post(intake_preflight))
        .route("/starbus/intake/create", post(intake_create))
        .route("/starbus/state/next_action", post(update_next_action))
        .route("/starbus/state/transition", post(transition_state))
        .route("/starbus/state/decision/resolve", post(resolve_decision))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn mk(
        task_id: Uuid,
        status: &str,
        created_at_s: i64,
        updated_at_s: i64,
    ) -> StarbusTaskWithMeta {
        StarbusTaskWithMeta {
            state: StarbusTaskStateData {
                task_id,
                title: "t".to_string(),
                status: status.to_string(),
                active_actor: None,
                active_role: None,
                next_action: None,
                decision_requests: Vec::new(),
                history: Vec::new(),
                step_count: 0,
                gate: None,
                tags: Vec::new(),
            },
            created_at: Utc.timestamp_opt(created_at_s, 0).unwrap(),
            updated_at: Utc.timestamp_opt(updated_at_s, 0).unwrap(),
        }
    }

    #[test]
    fn keeps_desired_active_when_present_and_non_terminal() {
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);
        let tasks = vec![
            mk(a, "EXECUTING", 10, 10),
            mk(b, "QUEUED", 11, 11),
        ];
        assert_eq!(select_active_task_id(Some(a), &tasks), Some(a));
    }

    #[test]
    fn heals_missing_active_to_best_candidate() {
        let missing = Uuid::from_u128(1);
        let queued = Uuid::from_u128(2);
        let executing = Uuid::from_u128(3);
        let tasks = vec![mk(queued, "QUEUED", 10, 10), mk(executing, "EXECUTING", 9, 9)];
        assert_eq!(select_active_task_id(Some(missing), &tasks), Some(executing));
    }

    #[test]
    fn heals_terminal_active_to_best_candidate() {
        let done = Uuid::from_u128(1);
        let designing = Uuid::from_u128(2);
        let tasks = vec![mk(done, "DONE", 10, 10), mk(designing, "DESIGNING", 9, 9)];
        assert_eq!(select_active_task_id(Some(done), &tasks), Some(designing));
    }

    #[test]
    fn chooses_priority_then_most_recent_update() {
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);
        let tasks = vec![
            mk(a, "VERIFYING", 10, 100),
            mk(b, "EXECUTING", 11, 1),
        ];
        assert_eq!(select_active_task_id(None, &tasks), Some(b));

        let tasks = vec![
            mk(a, "EXECUTING", 10, 1),
            mk(b, "EXECUTING", 11, 2),
        ];
        assert_eq!(select_active_task_id(None, &tasks), Some(b));
    }

    #[test]
    fn returns_none_when_all_tasks_terminal() {
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);
        let tasks = vec![mk(a, "DONE", 1, 1), mk(b, "FAILED", 2, 2)];
        assert_eq!(select_active_task_id(None, &tasks), None);
    }
}
