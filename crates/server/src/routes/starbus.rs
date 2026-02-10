use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
};

use axum::{
    Json, Router,
    extract::{Query, State},
    response::Json as ResponseJson,
    routing::{get, post},
};
use db::models::{
    execution_process::{ExecutionProcess, ExecutionProcessStatus},
    project::{CreateProject, Project},
    project_repo::CreateProjectRepo,
    scratch::{
        CreateScratch, Scratch, ScratchPayload, ScratchType, StarbusGlobalStateData,
        StarbusHistoryEntry, StarbusNextAction, StarbusTaskStateData, UpdateScratch,
    },
    task::{CreateTask, Task, TaskStatus},
    workspace::{CreateWorkspace, Workspace},
    workspace_repo::{CreateWorkspaceRepo, WorkspaceRepo},
};
use deployment::Deployment;
use executors::{executors::BaseCodingAgent, profile::ExecutorProfileId};
use serde::{Deserialize, Serialize};
use services::services::container::ContainerService;
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

fn initial_status_for_role(role: &str) -> String {
    match role {
        "role-product-manager" => "DESIGNING",
        "role-qa-security" => "AUDITING",
        "role-project-ops" | "role-technology" => "EXECUTING",
        _ => "DESIGNING",
    }
    .to_string()
}

fn starbus_status_from_task_status(status: &TaskStatus) -> String {
    match status {
        TaskStatus::Todo => "QUEUED",
        TaskStatus::InProgress => "EXECUTING",
        TaskStatus::InReview => "VERIFYING",
        TaskStatus::Done => "DONE",
        TaskStatus::Cancelled => "FAILED",
    }
    .to_string()
}

fn infer_dispatch_from_title(title: &str) -> (String, String, String, String) {
    let t = title.to_lowercase();
    if t.contains("req") || t.contains("clarif") || t.contains("gate definition") {
        return (
            "role-product-manager".to_string(),
            "DESIGNING".to_string(),
            "Gate0".to_string(),
            "Requirement clarification and planning".to_string(),
        );
    }
    if t.contains("test") || t.contains("evidence") || t.contains("contract") {
        return (
            "role-qa-security".to_string(),
            "AUDITING".to_string(),
            "Gate2".to_string(),
            "Test and evidence validation".to_string(),
        );
    }
    if t.contains("accept") || t.contains("audit") || t.contains("release") {
        return (
            "role-product-manager".to_string(),
            "VERIFYING".to_string(),
            "Gate3".to_string(),
            "Final audit and release decision".to_string(),
        );
    }
    if t.contains("control room") || t.contains("evidence wall") || t.contains("ui") {
        return (
            "role-project-ops".to_string(),
            "EXECUTING".to_string(),
            "Gate2".to_string(),
            "Frontend and control-room implementation".to_string(),
        );
    }
    (
        "role-technology".to_string(),
        "EXECUTING".to_string(),
        "Gate2".to_string(),
        "Backend and integration implementation".to_string(),
    )
}

fn normalize_actor_choice(raw: &str) -> Option<String> {
    let up = raw.trim().to_uppercase();
    match up.as_str() {
        "ACTOR_CLAUDE" | "CLAUDE" => Some("ACTOR_CLAUDE".to_string()),
        "ACTOR_CODEX" | "CODEX" => Some("ACTOR_CODEX".to_string()),
        "ACTOR_CURSOR" | "CURSOR" => Some("ACTOR_CURSOR".to_string()),
        "ACTOR_OPENCODE" | "OPENCODE" => Some("ACTOR_OPENCODE".to_string()),
        "ACTOR_TRAE" | "TRAE" => Some("ACTOR_TRAE".to_string()),
        "ACTOR_QODER" | "QODER" => Some("ACTOR_QODER".to_string()),
        "ACTOR_HUMAN" | "HUMAN" => Some("ACTOR_HUMAN".to_string()),
        _ => None,
    }
}

fn default_actor_options() -> Vec<String> {
    vec![
        "ACTOR_CLAUDE".to_string(),
        "ACTOR_CODEX".to_string(),
        "ACTOR_CURSOR".to_string(),
        "ACTOR_OPENCODE".to_string(),
    ]
}

fn actor_to_executor_profile(actor: &str) -> Option<ExecutorProfileId> {
    match actor {
        "ACTOR_CLAUDE" => Some(ExecutorProfileId::new(BaseCodingAgent::ClaudeCode)),
        "ACTOR_CODEX" => Some(ExecutorProfileId::new(BaseCodingAgent::Codex)),
        "ACTOR_CURSOR" => Some(ExecutorProfileId::new(BaseCodingAgent::CursorAgent)),
        "ACTOR_OPENCODE" => Some(ExecutorProfileId::new(BaseCodingAgent::Opencode)),
        // TRAE/QODER/HUMAN are handled outside of auto-start.
        _ => None,
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

fn is_tool_actor(actor: &str) -> bool {
    matches!(actor, "ACTOR_TRAE" | "ACTOR_QODER")
}

fn actor_from_state(state: &StarbusTaskStateData) -> Option<String> {
    state
        .active_actor
        .clone()
        .or_else(|| state.next_action.as_ref().map(|na| na.actor.clone()))
}

fn enforce_tool_actor_completion_guard(
    state: &StarbusTaskStateData,
    target_status: &str,
) -> Result<(), ApiError> {
    let target = normalize_status(target_status);
    if !matches!(target.as_str(), "DONE" | "FAILED") {
        return Ok(());
    }
    if let Some(actor) = actor_from_state(state)
        && is_tool_actor(&actor)
    {
        return Err(ApiError::BadRequest(format!(
            "Tool actor {actor} cannot set {target}; route to VERIFYING for PM/Director decision"
        )));
    }
    Ok(())
}

fn infer_resume_status_by_gate(gate: Option<&str>) -> String {
    match gate.unwrap_or("Gate3") {
        "Gate0" => "DESIGNING",
        "Gate1" => "AUDITING",
        "Gate2" => "EXECUTING",
        "Gate3" => "VERIFYING",
        _ => "VERIFYING",
    }
    .to_string()
}

fn default_next_action_for_status(status: &str) -> Option<StarbusNextAction> {
    match normalize_status(status).as_str() {
        "DESIGNING" => Some(StarbusNextAction {
            actor: "ACTOR_CODEX".to_string(),
            role: "role-technology".to_string(),
            action: "Design and implementation planning".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }),
        "AUDITING" => Some(StarbusNextAction {
            actor: "ACTOR_CLAUDE".to_string(),
            role: "role-qa-security".to_string(),
            action: "Audit latest design/dev outputs".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }),
        "EXECUTING" => Some(StarbusNextAction {
            actor: "ACTOR_TRAE".to_string(),
            role: "role-project-ops".to_string(),
            action: "Implement and test".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }),
        "VERIFYING" => Some(StarbusNextAction {
            actor: "ACTOR_CLAUDE".to_string(),
            role: "role-product-manager".to_string(),
            action: "PM verification and release decision".to_string(),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }),
        _ => None,
    }
}

fn discover_workspace_root() -> PathBuf {
    if let Ok(explicit) = env::var("STARBUS_WORKSPACE_ROOT") {
        return PathBuf::from(explicit);
    }
    let start = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // Prefer the nearest git root so artifacts stay inside the current repo.
    for dir in start.ancestors() {
        if dir.join(".git").exists() {
            return dir.to_path_buf();
        }
    }
    for dir in start.ancestors() {
        if dir.join("tasks").is_dir() && dir.join("artifacts").is_dir() {
            return dir.to_path_buf();
        }
    }
    start
}

fn starbus_runs_root(workspace_root: &PathBuf) -> PathBuf {
    workspace_root.join("docs").join("starbus").join("runs")
}

fn write_task_skeleton_files(
    task_id: Uuid,
    payload: &StarbusIntakeRequest,
    status: &str,
) -> Result<PathBuf, ApiError> {
    let workspace_root = discover_workspace_root();
    let task_dir = starbus_runs_root(&workspace_root).join(task_id.to_string());
    fs::create_dir_all(&task_dir)
        .map_err(|e| ApiError::BadRequest(format!("Failed to create task directory: {e}")))?;

    let domain_roles = if payload.domain_roles.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            payload
                .domain_roles
                .iter()
                .map(|r| format!(r#""{r}""#))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let tags = if payload.tags.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            payload
                .tags
                .iter()
                .map(|t| format!(r#""{t}""#))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let task_md = format!(
        "---\ntask_id: {task_id}\ntitle: \"{title}\"\npriority: {priority}\nstatus: {status}\ndomain_roles: {domain_roles}\ninclude_recommended_deps: {include_deps}\ntags: {tags}\n---\n\n# Goal\n{description}\n\n# Acceptance\n{acceptance}\n",
        title = payload.title.replace('"', "'"),
        priority = payload.priority.clone().unwrap_or_else(|| "P1".to_string()),
        include_deps = payload.include_recommended_deps.unwrap_or(false),
        description = payload
            .description
            .clone()
            .unwrap_or_else(|| "TBD".to_string()),
        acceptance = payload
            .acceptance
            .clone()
            .unwrap_or_else(|| "TBD".to_string()),
    );
    fs::write(task_dir.join("task.md"), task_md)
        .map_err(|e| ApiError::BadRequest(format!("Failed to write task.md: {e}")))?;
    fs::write(
        task_dir.join("context.md"),
        "# Context\n\n- Source of truth: Vibe DB\n- state.json is mirror only\n",
    )
    .map_err(|e| ApiError::BadRequest(format!("Failed to write context.md: {e}")))?;
    fs::write(
        task_dir.join("playbook.md"),
        "# Playbook\n\n## Gate0\n- Clarify and align requirements\n\n## Gate1\n- Design and audit\n\n## Gate2\n- Implement and test\n\n## Gate3\n- Verify and release\n",
    )
    .map_err(|e| ApiError::BadRequest(format!("Failed to write playbook.md: {e}")))?;
    Ok(task_dir)
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct StarbusStateResponse {
    pub active_task_id: Option<Uuid>,
    pub tasks: Vec<StarbusTaskStateData>,
}

#[derive(Debug, Deserialize)]
pub struct StarbusStateQuery {
    pub project_id: Option<Uuid>,
    pub title_prefix: Option<String>,
    pub active_only: Option<bool>,
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
    let mut blocked_human_reasons = Vec::new();
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
        blocked_human_reasons.push("domain_roles overflow (>4), requires HITL".to_string());
    }
    if payload.include_recommended_deps.is_none() {
        errors.push("include_recommended_deps is required".to_string());
    }
    if payload
        .description
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        warnings.push("description is empty".to_string());
    }

    StarbusPreflightResponse {
        ok: errors.is_empty(),
        errors,
        warnings,
        blocked_human_reasons,
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct StarbusPreflightResponse {
    pub ok: bool,
    #[serde(default)]
    pub errors: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub blocked_human_reasons: Vec<String>,
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

#[derive(Debug, Deserialize, TS)]
pub struct StarbusProjectStatusSyncRequest {
    pub project_id: Uuid,
    #[serde(default)]
    pub title_prefixes: Vec<String>,
    #[serde(default)]
    pub dry_run: Option<bool>,
    #[serde(default)]
    pub prune_nonmatching_scratch: Option<bool>,
    #[serde(default)]
    pub set_active_to_latest: Option<bool>,
}

#[derive(Debug, Serialize, TS)]
pub struct StarbusProjectStatusSyncResponse {
    pub project_id: Uuid,
    pub dry_run: bool,
    pub matched_task_ids: Vec<Uuid>,
    pub updated_task_ids: Vec<Uuid>,
    pub pruned_scratch_ids: Vec<Uuid>,
    pub active_task_id: Option<Uuid>,
}

#[derive(Debug, Serialize, TS)]
pub struct StarbusStatusMappingResponse {
    pub starbus_to_task: HashMap<String, String>,
    pub allowed_blocked_resume_targets: Vec<String>,
    pub canonical_statuses: Vec<String>,
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
    #[serde(default)]
    pub gate: Option<String>,
}

#[derive(Debug, Deserialize, TS, Clone)]
pub struct StarbusProjectSeed {
    pub name: String,
    pub repositories: Vec<CreateProjectRepo>,
}

#[derive(Debug, Deserialize, TS, Clone)]
pub struct StarbusRunRoleTaskRequest {
    #[serde(default)]
    pub project_id: Option<Uuid>,
    #[serde(default)]
    pub project_seed: Option<StarbusProjectSeed>,
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
    pub actor: Option<String>,
    pub role: String,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default = "default_true")]
    pub set_active: bool,
    #[serde(default)]
    pub executor_profile_id: Option<ExecutorProfileId>,
    #[serde(default)]
    pub target_branches: HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub struct StarbusRunRoleTaskResponse {
    pub project_id: Uuid,
    pub task_id: Uuid,
    #[serde(default)]
    pub workspace_id: Option<Uuid>,
    pub status: String,
    pub started: bool,
    pub actor: String,
    pub role: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub struct StarbusWorkspaceRunInfo {
    pub workspace_id: Uuid,
    pub branch: String,
    #[serde(default)]
    pub container_ref: Option<String>,
    #[serde(default)]
    pub latest_execution_process_id: Option<Uuid>,
    #[serde(default)]
    pub latest_session_id: Option<Uuid>,
    #[serde(default)]
    pub latest_status: Option<String>,
    #[serde(default)]
    pub latest_completed_at: Option<String>,
    pub is_running: bool,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub struct StarbusRunsResponse {
    pub task_id: Uuid,
    pub workspace_count: usize,
    pub workspaces: Vec<StarbusWorkspaceRunInfo>,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub struct StarbusHandoffRequest {
    pub task_id: Uuid,
    pub summary: String,
    #[serde(default)]
    pub results: Vec<String>,
    #[serde(default)]
    pub next_steps: Vec<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub struct StarbusHandoffResponse {
    pub task_id: Uuid,
    pub handoff_path: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub struct StarbusDispatchRequest {
    pub task_id: Uuid,
    #[serde(default)]
    pub actor: Option<String>,
    #[serde(default)]
    pub actor_options: Option<Vec<String>>,
    #[serde(default)]
    pub hitl_select_actor: Option<bool>,
    #[serde(default = "default_true")]
    pub set_active: bool,
    #[serde(default = "default_true")]
    pub auto_start: bool,
    #[serde(default)]
    pub role_override: Option<String>,
    #[serde(default)]
    pub action_override: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, TS, Clone)]
pub struct StarbusDispatchResponse {
    pub task_id: Uuid,
    pub actor: String,
    pub role: String,
    pub status: String,
    pub gate: String,
    pub prompt_path: String,
    #[serde(default)]
    pub workspace_id: Option<Uuid>,
    pub started: bool,
    #[serde(default)]
    pub note: Option<String>,
}

fn render_dispatch_prompt(title: &str, role: &str, action: &str) -> String {
    format!(
        "# Dispatch Prompt\n\n- Task: {title}\n- Role: {role}\n- Action: {action}\n\n## Read First\n1. docs/product/Architecture/vibe-starbus-architecture-prd.md\n2. docs/product/Architecture/v2-i1-key-rules-memory.md\n3. docs/product/Architecture/vibe-starbus-ai-handoff-pack.md\n4. docs/guides/prompts/<selected-v2-i1-prompt>.md\n\n## Output Contract\n- Keep outputs markdown-first.\n- Write phase artifacts under docs/starbus/runs/<task-id>/.\n- Write evidence index updates to artifacts/TASK-010-STARBUS-VIBE-INTEGRATION-MVP/README.md.\n"
    )
}

fn infer_prompt_template_relpath(title: &str, role: &str) -> &'static str {
    let t = title.to_lowercase();
    let r = role.to_lowercase();
    if t.contains("req")
        || t.contains("clarif")
        || t.contains("gate definition")
        || r.contains("product-manager")
    {
        "docs/guides/prompts/v2-i1-req-prompt.md"
    } else if t.contains("test") || t.contains("evidence") || r.contains("qa-security") {
        "docs/guides/prompts/v2-i1-test-prompt.md"
    } else if t.contains("accept") || t.contains("audit") || t.contains("release") {
        "docs/guides/prompts/v2-i1-accept-prompt.md"
    } else {
        "docs/guides/prompts/v2-i1-dev-prompt.md"
    }
}

fn existing_prompt_relpath(workspace_root: &Path, title: &str, role: &str) -> Option<String> {
    let rel = infer_prompt_template_relpath(title, role);
    let abs = workspace_root.join(rel);
    if abs.exists() {
        Some(rel.to_string())
    } else {
        None
    }
}

fn role_domain_defaults(role: &str) -> Vec<String> {
    match role {
        "role-product-manager" => vec!["role-product-manager".to_string()],
        "role-qa-security" => vec!["role-qa-security".to_string()],
        "role-project-ops" => vec!["role-project-ops".to_string()],
        _ => vec!["role-technology".to_string()],
    }
}

async fn get_global_state(deployment: &DeploymentImpl) -> Result<StarbusGlobalStateData, ApiError> {
    let global = Scratch::find_by_id(
        &deployment.db().pool,
        starbus_global_id(),
        &ScratchType::StarbusGlobalState,
    )
    .await?;
    Ok(match global {
        Some(s) => match s.payload {
            ScratchPayload::StarbusGlobalState(data) => data,
            _ => StarbusGlobalStateData {
                active_task_id: None,
            },
        },
        None => StarbusGlobalStateData {
            active_task_id: None,
        },
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

async fn list_starbus_tasks(
    deployment: &DeploymentImpl,
) -> Result<Vec<StarbusTaskStateData>, ApiError> {
    let all = Scratch::find_all(&deployment.db().pool).await?;
    let tasks = all
        .into_iter()
        .filter_map(|scratch| match scratch.payload {
            ScratchPayload::StarbusTaskState(data) => Some(data),
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
    Query(query): Query<StarbusStateQuery>,
) -> Result<ResponseJson<ApiResponse<StarbusStateResponse>>, ApiError> {
    let global = get_global_state(&deployment).await?;
    let mut tasks = list_starbus_tasks(&deployment).await?;

    let mut project_task_ids: Option<HashSet<Uuid>> = None;
    if let Some(project_id) = query.project_id {
        let project_tasks =
            Task::find_by_project_id_with_attempt_status(&deployment.db().pool, project_id)
                .await?;
        project_task_ids = Some(project_tasks.into_iter().map(|t| t.id).collect());
    }

    let prefixes: Vec<String> = query
        .title_prefix
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect();

    tasks.retain(|state| {
        if query.active_only.unwrap_or(false) && Some(state.task_id) != global.active_task_id {
            return false;
        }
        if let Some(ids) = project_task_ids.as_ref() && !ids.contains(&state.task_id) {
            return false;
        }
        if !prefixes.is_empty() {
            let title = state.title.to_lowercase();
            if !prefixes.iter().any(|p| title.starts_with(p)) {
                return false;
            }
        }
        true
    });

    Ok(ResponseJson(ApiResponse::success(StarbusStateResponse {
        active_task_id: global.active_task_id,
        tasks,
    })))
}

pub async fn get_status_mapping(
    State(_deployment): State<DeploymentImpl>,
) -> Result<ResponseJson<ApiResponse<StarbusStatusMappingResponse>>, ApiError> {
    let mut mapping = HashMap::new();
    mapping.insert("QUEUED".to_string(), "todo".to_string());
    mapping.insert("DESIGNING".to_string(), "inprogress".to_string());
    mapping.insert("EXECUTING".to_string(), "inprogress".to_string());
    mapping.insert("AUDITING".to_string(), "inreview".to_string());
    mapping.insert("VERIFYING".to_string(), "inreview".to_string());
    mapping.insert("BLOCKED_HUMAN".to_string(), "inreview".to_string());
    mapping.insert("DONE".to_string(), "done".to_string());
    mapping.insert("FAILED".to_string(), "cancelled".to_string());

    Ok(ResponseJson(ApiResponse::success(StarbusStatusMappingResponse {
        starbus_to_task: mapping,
        allowed_blocked_resume_targets: vec![
            "DESIGNING".to_string(),
            "AUDITING".to_string(),
            "EXECUTING".to_string(),
            "VERIFYING".to_string(),
        ],
        canonical_statuses: vec![
            "QUEUED".to_string(),
            "DESIGNING".to_string(),
            "AUDITING".to_string(),
            "EXECUTING".to_string(),
            "VERIFYING".to_string(),
            "DONE".to_string(),
            "BLOCKED_HUMAN".to_string(),
            "FAILED".to_string(),
        ],
    })))
}

pub async fn sync_project_statuses(
    State(deployment): State<DeploymentImpl>,
    Json(req): Json<StarbusProjectStatusSyncRequest>,
) -> Result<ResponseJson<ApiResponse<StarbusProjectStatusSyncResponse>>, ApiError> {
    let dry_run = req.dry_run.unwrap_or(false);
    let project_tasks =
        Task::find_by_project_id_with_attempt_status(&deployment.db().pool, req.project_id)
            .await?;
    let project_task_map: HashMap<Uuid, Task> =
        project_tasks.into_iter().map(|t| (t.id, t.task)).collect();
    if project_task_map.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "No tasks found for project {}",
            req.project_id
        )));
    }

    let mut state_tasks = list_starbus_tasks(&deployment).await?;
    state_tasks.retain(|s| project_task_map.contains_key(&s.task_id));

    let normalized_prefixes: Vec<String> = req
        .title_prefixes
        .iter()
        .map(|p| p.trim().to_lowercase())
        .filter(|p| !p.is_empty())
        .collect();
    if !normalized_prefixes.is_empty() {
        state_tasks.retain(|s| {
            let title = s.title.to_lowercase();
            normalized_prefixes.iter().any(|p| title.starts_with(p))
        });
    }

    let matched_ids: HashSet<Uuid> = state_tasks.iter().map(|s| s.task_id).collect();
    let mut updated_task_ids = Vec::new();
    for state in &state_tasks {
        let Some(task) = project_task_map.get(&state.task_id) else {
            continue;
        };
        let mapped = map_starbus_to_task_status(&state.status);
        if task.status != mapped {
            updated_task_ids.push(task.id);
            if !dry_run {
                let _ = Task::update_status(&deployment.db().pool, task.id, mapped).await;
            }
        }
    }

    let mut pruned_scratch_ids = Vec::new();
    if req.prune_nonmatching_scratch.unwrap_or(false) {
        let all_state_tasks = list_starbus_tasks(&deployment).await?;
        for state in all_state_tasks {
            if project_task_map.contains_key(&state.task_id) && !matched_ids.contains(&state.task_id)
            {
                pruned_scratch_ids.push(state.task_id);
                if !dry_run {
                    let _ = Scratch::delete(
                        &deployment.db().pool,
                        state.task_id,
                        &ScratchType::StarbusTaskState,
                    )
                    .await;
                }
            }
        }
    }

    let mut active_task_id = get_global_state(&deployment).await?.active_task_id;
    if req.set_active_to_latest.unwrap_or(false) {
        let latest = state_tasks
            .iter()
            .filter_map(|s| project_task_map.get(&s.task_id).map(|t| (t.updated_at, t.id)))
            .max_by_key(|(updated_at, _)| *updated_at)
            .map(|(_, id)| id);
        if let Some(latest_id) = latest {
            active_task_id = Some(latest_id);
            if !dry_run {
                upsert_global_state(
                    &deployment,
                    StarbusGlobalStateData {
                        active_task_id: Some(latest_id),
                    },
                )
                .await?;
            }
        }
    }

    Ok(ResponseJson(ApiResponse::success(
        StarbusProjectStatusSyncResponse {
            project_id: req.project_id,
            dry_run,
            matched_task_ids: matched_ids.into_iter().collect(),
            updated_task_ids,
            pruned_scratch_ids,
            active_task_id,
        },
    )))
}

pub async fn intake_preflight(
    State(_deployment): State<DeploymentImpl>,
    Json(payload): Json<StarbusIntakeRequest>,
) -> Result<ResponseJson<ApiResponse<StarbusPreflightResponse>>, ApiError> {
    Ok(ResponseJson(ApiResponse::success(validate_intake(
        &payload,
    ))))
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
    let requires_hitl = payload.domain_roles.len() > 4;
    let initial_status = if requires_hitl {
        "BLOCKED_HUMAN"
    } else {
        "QUEUED"
    };
    let task_status = map_starbus_to_task_status(initial_status);
    let create_task = CreateTask {
        project_id: payload.project_id,
        title: payload.title.clone(),
        description: payload.description.clone(),
        status: Some(task_status),
        parent_workspace_id: None,
        image_ids: None,
    };
    let _ = Task::create(&deployment.db().pool, &create_task, task_id).await?;

    let next_action = if requires_hitl {
        Some(StarbusNextAction {
            actor: "ACTOR_HUMAN".to_string(),
            role: "role-product-manager".to_string(),
            action: "Resolve domain_roles overflow decision".to_string(),
            inputs: vec!["docs/starbus/runs/<task-id>/task.md".to_string()],
            outputs: vec![],
        })
    } else {
        payload
            .default_actor
            .as_ref()
            .map(|actor| StarbusNextAction {
                actor: actor.clone(),
                role: payload
                    .default_role
                    .clone()
                    .unwrap_or_else(|| "role-product-manager".to_string()),
                action: "Clarify/plan".to_string(),
                inputs: Vec::new(),
                outputs: Vec::new(),
            })
    };

    let decision_requests = if requires_hitl {
        vec![db::models::scratch::StarbusDecisionRequest {
            id: format!("DR-{}", chrono::Utc::now().timestamp_millis()),
            question: "domain_roles exceeds 4. Choose roles to keep (<=4) before execution."
                .to_string(),
            options: vec![
                "Keep first 4 roles".to_string(),
                "Manually select 4 roles".to_string(),
                "Cancel task".to_string(),
            ],
            recommended: Some("Keep first 4 roles".to_string()),
            context_refs: vec!["Gate0".to_string()],
            resolved_at: None,
            resolution: None,
        }]
    } else {
        Vec::new()
    };

    let task_state = StarbusTaskStateData {
        task_id,
        title: payload.title.clone(),
        status: initial_status.to_string(),
        priority: payload.priority.clone(),
        active_actor: None,
        active_role: None,
        next_action,
        decision_requests,
        history: vec![StarbusHistoryEntry {
            ts: chrono::Utc::now().to_rfc3339(),
            from_status: Some("VOID".to_string()),
            to_status: Some(initial_status.to_string()),
            actor: Some("ACTOR_HUMAN".to_string()),
            note: Some("Created via intake API".to_string()),
        }],
        step_count: 0,
        gate: Some("Gate0".to_string()),
        tags: payload.tags.clone(),
        domain_roles: payload.domain_roles.clone(),
        include_recommended_deps: payload.include_recommended_deps,
    };

    let task_dir = match write_task_skeleton_files(task_id, &payload, initial_status) {
        Ok(dir) => dir,
        Err(err) => {
            let _ = Task::delete(&deployment.db().pool, task_id).await;
            return Err(err);
        }
    };

    let create = CreateScratch {
        payload: ScratchPayload::StarbusTaskState(task_state.clone()),
    };
    if let Err(err) = Scratch::create(&deployment.db().pool, task_id, &create).await {
        let _ = fs::remove_dir_all(&task_dir);
        let _ = Task::delete(&deployment.db().pool, task_id).await;
        return Err(ApiError::BadRequest(format!(
            "Failed to create starbus scratch after task create: {err}"
        )));
    }

    if payload.set_active {
        upsert_global_state(
            &deployment,
            StarbusGlobalStateData {
                active_task_id: Some(task_id),
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
        enforce_tool_actor_completion_guard(&state, status)?;
        state.status = normalize_status(status);
        let mapped = map_starbus_to_task_status(&state.status);
        let _ = Task::update_status(&deployment.db().pool, state.task_id, mapped).await;
    }
    if let Some(next_action) = update.next_action {
        state.next_action = Some(next_action);
    }
    if update.set_active.unwrap_or(false) {
        upsert_global_state(
            &deployment,
            StarbusGlobalStateData {
                active_task_id: Some(state.task_id),
            },
        )
        .await?;
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

    Ok(ResponseJson(ApiResponse::success(state)))
}

pub async fn transition_state(
    State(deployment): State<DeploymentImpl>,
    Json(update): Json<StarbusTransitionRequest>,
) -> Result<ResponseJson<ApiResponse<StarbusTaskStateData>>, ApiError> {
    let mut state = get_starbus_task(&deployment, update.task_id).await?;
    let target = normalize_status(&update.status);
    enforce_tool_actor_completion_guard(&state, &target)?;
    if !is_valid_transition(&state.status, &target) {
        return Err(ApiError::BadRequest(format!(
            "Invalid transition: {} -> {}",
            state.status, target
        )));
    }
    if target == "BLOCKED_HUMAN" {
        let gate = update.gate.clone().or_else(|| state.gate.clone());
        if gate.is_none() {
            return Err(ApiError::BadRequest(
                "BLOCKED_HUMAN transition must include gate binding".to_string(),
            ));
        }
        state.gate = gate;
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

    if update.set_active.unwrap_or(false) {
        upsert_global_state(
            &deployment,
            StarbusGlobalStateData {
                active_task_id: Some(state.task_id),
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
    let mut resolved_actor: Option<String> = None;
    let mut actor_selection_decision = false;
    for item in state.decision_requests.iter_mut() {
        if item.id == req.decision_id {
            actor_selection_decision = item
                .context_refs
                .iter()
                .any(|ref_item| ref_item == "HITL_ACTOR_SELECT");
            if actor_selection_decision {
                let actor_choice = normalize_actor_choice(&req.resolution).ok_or_else(|| {
                    ApiError::BadRequest(format!(
                        "Invalid actor selection '{}'. Expected one of ACTOR_CLAUDE/ACTOR_CODEX/ACTOR_CURSOR/ACTOR_OPENCODE/ACTOR_TRAE/ACTOR_QODER",
                        req.resolution
                    ))
                })?;
                item.resolution = Some(actor_choice.clone());
                resolved_actor = Some(actor_choice);
            } else {
                item.resolution = Some(req.resolution.clone());
            }
            item.resolved_at = req
                .resolved_at
                .clone()
                .or_else(|| Some(chrono::Utc::now().to_rfc3339()));
            found = true;
        }
    }
    if !found {
        return Err(ApiError::BadRequest(
            "Decision request not found".to_string(),
        ));
    }

    if let Some(selected_actor) = resolved_actor.clone() {
        if let Some(next_action) = state.next_action.as_mut() {
            next_action.actor = selected_actor.clone();
        } else {
            state.next_action = default_next_action_for_status(&state.status);
            if let Some(next_action) = state.next_action.as_mut() {
                next_action.actor = selected_actor.clone();
            }
        }
        state.active_actor = Some(selected_actor.clone());
        state.history.push(StarbusHistoryEntry {
            ts: chrono::Utc::now().to_rfc3339(),
            from_status: Some(state.status.clone()),
            to_status: Some(state.status.clone()),
            actor: Some("ACTOR_HUMAN".to_string()),
            note: Some(format!("actor selected via HITL: {selected_actor}")),
        });
    }

    state.history.push(StarbusHistoryEntry {
        ts: chrono::Utc::now().to_rfc3339(),
        from_status: Some(state.status.clone()),
        to_status: Some(state.status.clone()),
        actor: Some("ACTOR_HUMAN".to_string()),
        note: Some(format!("decision {} resolved", req.decision_id)),
    });

    if let Some(resume_status) = req.resume_status {
        let target = normalize_status(&resume_status);
        enforce_tool_actor_completion_guard(&state, &target)?;
        if is_valid_transition(&state.status, &target) {
            state.history.push(StarbusHistoryEntry {
                ts: chrono::Utc::now().to_rfc3339(),
                from_status: Some(state.status.clone()),
                to_status: Some(target.clone()),
                actor: Some("ACTOR_HUMAN".to_string()),
                note: Some(format!(
                    "decision {} resolved, manual resume",
                    req.decision_id
                )),
            });
            state.status = target;
        }
    } else if normalize_status(&state.status) == "BLOCKED_HUMAN"
        && state
            .decision_requests
            .iter()
            .all(|d| d.resolved_at.is_some())
    {
        let inferred = infer_resume_status_by_gate(state.gate.as_deref());
        if is_valid_transition(&state.status, &inferred) {
            state.history.push(StarbusHistoryEntry {
                ts: chrono::Utc::now().to_rfc3339(),
                from_status: Some(state.status.clone()),
                to_status: Some(inferred.clone()),
                actor: Some("ORCHESTRATOR".to_string()),
                note: Some(format!(
                    "auto_resume after all decisions resolved (gate={})",
                    state.gate.clone().unwrap_or_else(|| "Gate3".to_string())
                )),
            });
            state.status = inferred.clone();
            if state.next_action.is_none() {
                state.next_action = default_next_action_for_status(&inferred);
            }
        }
    }
    if let Some(next_action) = req.next_action {
        state.next_action = Some(next_action);
    }

    if actor_selection_decision && normalize_status(&state.status) != "BLOCKED_HUMAN" {
        let task = Task::find_by_id(&deployment.db().pool, state.task_id)
            .await?
            .ok_or_else(|| ApiError::BadRequest(format!("Task not found: {}", state.task_id)))?;
        let actor_for_start = state
            .next_action
            .as_ref()
            .map(|n| n.actor.clone())
            .unwrap_or_else(|| "ACTOR_CLAUDE".to_string());
        let (started, workspace_id, note) =
            auto_start_task_workspace(&deployment, &task, &actor_for_start).await?;
        state.history.push(StarbusHistoryEntry {
            ts: chrono::Utc::now().to_rfc3339(),
            from_status: Some(state.status.clone()),
            to_status: Some(state.status.clone()),
            actor: Some("ORCHESTRATOR".to_string()),
            note: Some(if started {
                format!(
                    "auto_start_after_hitl: actor={} workspace_id={}",
                    actor_for_start,
                    workspace_id
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "n/a".to_string())
                )
            } else {
                format!(
                    "auto_start_after_hitl skipped/failed for actor={}: {}",
                    actor_for_start,
                    note.unwrap_or_else(|| "no details".to_string())
                )
            }),
        });
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

    Ok(ResponseJson(ApiResponse::success(state)))
}

pub async fn run_role_task(
    State(deployment): State<DeploymentImpl>,
    Json(payload): Json<StarbusRunRoleTaskRequest>,
) -> Result<ResponseJson<ApiResponse<StarbusRunRoleTaskResponse>>, ApiError> {
    if payload.title.trim().is_empty() {
        return Err(ApiError::BadRequest("title is required".to_string()));
    }
    if payload.role.trim().is_empty() {
        return Err(ApiError::BadRequest("role is required".to_string()));
    }

    let project = if let Some(project_id) = payload.project_id {
        Project::find_by_id(&deployment.db().pool, project_id)
            .await?
            .ok_or_else(|| ApiError::BadRequest(format!("Project not found: {project_id}")))?
    } else {
        let seed = payload
            .project_seed
            .clone()
            .ok_or_else(|| ApiError::BadRequest("project_seed is required when project_id is not set".to_string()))?;
        if seed.repositories.is_empty() {
            return Err(ApiError::BadRequest(
                "project_seed.repositories must not be empty".to_string(),
            ));
        }
        deployment
            .project()
            .create_project(
                &deployment.db().pool,
                deployment.repo(),
                CreateProject {
                    name: seed.name,
                    repositories: seed.repositories,
                },
            )
            .await
            .map_err(|e| ApiError::BadRequest(format!("Failed to create project: {e}")))?
    };

    let actor = payload
        .actor
        .clone()
        .unwrap_or_else(|| "ACTOR_CLAUDE".to_string());
    let initial_status = initial_status_for_role(&payload.role);

    let preflight_payload = StarbusIntakeRequest {
        project_id: project.id,
        title: payload.title.clone(),
        description: payload.description.clone(),
        acceptance: payload.acceptance.clone(),
        priority: payload.priority.clone().or(Some("P1".to_string())),
        domain_roles: payload.domain_roles.clone(),
        include_recommended_deps: payload.include_recommended_deps,
        tags: payload.tags.clone(),
        set_active: payload.set_active,
        default_actor: Some(actor.clone()),
        default_role: Some(payload.role.clone()),
    };
    let preflight = validate_intake(&preflight_payload);
    if !preflight.ok {
        return Err(ApiError::BadRequest(format!(
            "Preflight failed: {}",
            preflight.errors.join("; ")
        )));
    }

    let task_id = Uuid::new_v4();
    let task_status = map_starbus_to_task_status(&initial_status);
    let create_task = CreateTask {
        project_id: project.id,
        title: payload.title.clone(),
        description: payload.description.clone(),
        status: Some(task_status),
        parent_workspace_id: None,
        image_ids: None,
    };
    let _ = Task::create(&deployment.db().pool, &create_task, task_id).await?;

    let next_action = StarbusNextAction {
        actor: actor.clone(),
        role: payload.role.clone(),
        action: payload
            .action
            .clone()
            .unwrap_or_else(|| "Auto started from starbus/run-role-task".to_string()),
        inputs: vec![
            format!("docs/starbus/runs/{task_id}/task.md"),
            format!("docs/starbus/runs/{task_id}/context.md"),
        ],
        outputs: vec![
            format!("docs/starbus/runs/{task_id}/03-dev.md"),
            format!("docs/starbus/runs/{task_id}/04-test.md"),
            format!("docs/starbus/runs/{task_id}/06-audit.md"),
        ],
    };

    let task_state = StarbusTaskStateData {
        task_id,
        title: payload.title.clone(),
        status: initial_status.clone(),
        priority: payload.priority.clone().or(Some("P1".to_string())),
        active_actor: Some(actor.clone()),
        active_role: Some(payload.role.clone()),
        next_action: Some(next_action),
        decision_requests: Vec::new(),
        history: vec![StarbusHistoryEntry {
            ts: chrono::Utc::now().to_rfc3339(),
            from_status: Some("VOID".to_string()),
            to_status: Some(initial_status.clone()),
            actor: Some("ORCHESTRATOR".to_string()),
            note: Some("Created via /api/starbus/run-role-task".to_string()),
        }],
        step_count: 0,
        gate: Some("Gate1".to_string()),
        tags: payload.tags.clone(),
        domain_roles: payload.domain_roles.clone(),
        include_recommended_deps: payload.include_recommended_deps,
    };

    let task_dir = match write_task_skeleton_files(task_id, &preflight_payload, &initial_status) {
        Ok(dir) => dir,
        Err(err) => {
            let _ = Task::delete(&deployment.db().pool, task_id).await;
            return Err(err);
        }
    };

    if let Err(err) = Scratch::create(
        &deployment.db().pool,
        task_id,
        &CreateScratch {
            payload: ScratchPayload::StarbusTaskState(task_state.clone()),
        },
    )
    .await
    {
        let _ = fs::remove_dir_all(&task_dir);
        let _ = Task::delete(&deployment.db().pool, task_id).await;
        return Err(ApiError::BadRequest(format!(
            "Failed to persist starbus task state: {err}"
        )));
    }

    if payload.set_active {
        upsert_global_state(
            &deployment,
            StarbusGlobalStateData {
                active_task_id: Some(task_id),
            },
        )
        .await?;
    }

    let repos = db::models::project_repo::ProjectRepo::find_repos_for_project(
        &deployment.db().pool,
        project.id,
    )
    .await?;
    if repos.is_empty() {
        return Ok(ResponseJson(ApiResponse::success(StarbusRunRoleTaskResponse {
            project_id: project.id,
            task_id,
            workspace_id: None,
            status: initial_status,
            started: false,
            actor,
            role: payload.role,
            note: Some("Project has no repositories; task created but not started".to_string()),
        })));
    }

    let attempt_id = Uuid::new_v4();
    let git_branch_name = deployment
        .container()
        .git_branch_from_workspace(&attempt_id, &payload.title)
        .await;
    let agent_working_dir = if repos.len() == 1 {
        let repo = &repos[0];
        match &repo.default_working_dir {
            Some(subdir) => Some(PathBuf::from(&repo.name).join(subdir).to_string_lossy().to_string()),
            None => Some(repo.name.clone()),
        }
    } else {
        None
    };

    let workspace = Workspace::create(
        &deployment.db().pool,
        &CreateWorkspace {
            branch: git_branch_name,
            agent_working_dir,
        },
        attempt_id,
        task_id,
    )
    .await?;

    let workspace_repos: Vec<CreateWorkspaceRepo> = repos
        .iter()
        .map(|repo| CreateWorkspaceRepo {
            repo_id: repo.id,
            target_branch: payload
                .target_branches
                .get(&repo.name)
                .cloned()
                .or_else(|| repo.default_target_branch.clone())
                .unwrap_or_else(|| "main".to_string()),
        })
        .collect();
    WorkspaceRepo::create_many(&deployment.db().pool, workspace.id, &workspace_repos).await?;

    let executor_profile_id = payload
        .executor_profile_id
        .unwrap_or_else(|| ExecutorProfileId::new(BaseCodingAgent::ClaudeCode));
    let started = deployment
        .container()
        .start_workspace(&workspace, executor_profile_id)
        .await
        .inspect_err(|err| tracing::error!("Failed to start starbus workspace: {}", err))
        .is_ok();

    Ok(ResponseJson(ApiResponse::success(StarbusRunRoleTaskResponse {
        project_id: project.id,
        task_id,
        workspace_id: Some(workspace.id),
        status: initial_status,
        started,
        actor,
        role: payload.role,
        note: if started {
            Some("Workspace started with executor profile".to_string())
        } else {
            Some("Workspace created, but start failed; check container logs".to_string())
        },
    })))
}

async fn auto_start_task_workspace(
    deployment: &DeploymentImpl,
    task: &Task,
    actor: &str,
) -> Result<(bool, Option<Uuid>, Option<String>), ApiError> {
    let Some(profile) = actor_to_executor_profile(actor) else {
        return Ok((
            false,
            None,
            Some(format!(
                "Actor {actor} is manual-only in vibe-kanban; workspace was not auto-started"
            )),
        ));
    };

    let repos = db::models::project_repo::ProjectRepo::find_repos_for_project(
        &deployment.db().pool,
        task.project_id,
    )
    .await?;
    if repos.is_empty() {
        return Ok((
            false,
            None,
            Some("Project has no repositories; dispatch stored but not started".to_string()),
        ));
    }

    let existing_workspaces = Workspace::fetch_all(&deployment.db().pool, Some(task.id)).await?;
    let mut workspace = if let Some(ws) = existing_workspaces.first() {
        ws.clone()
    } else {
        let attempt_id = Uuid::new_v4();
        let git_branch_name = deployment
            .container()
            .git_branch_from_workspace(&attempt_id, &task.title)
            .await;
        let agent_working_dir = if repos.len() == 1 {
            let repo = &repos[0];
            match &repo.default_working_dir {
                Some(subdir) => Some(
                    PathBuf::from(&repo.name)
                        .join(subdir)
                        .to_string_lossy()
                        .to_string(),
                ),
                None => Some(repo.name.clone()),
            }
        } else {
            None
        };

        let ws = Workspace::create(
            &deployment.db().pool,
            &CreateWorkspace {
                branch: git_branch_name,
                agent_working_dir,
            },
            attempt_id,
            task.id,
        )
        .await?;

        let workspace_repos: Vec<CreateWorkspaceRepo> = repos
            .iter()
            .map(|repo| CreateWorkspaceRepo {
                repo_id: repo.id,
                target_branch: repo
                    .default_target_branch
                    .clone()
                    .unwrap_or_else(|| "main".to_string()),
            })
            .collect();
        WorkspaceRepo::create_many(&deployment.db().pool, ws.id, &workspace_repos).await?;
        ws
    };

    let workspace_id = Some(workspace.id);
    match deployment.container().start_workspace(&workspace, profile.clone()).await {
        Ok(_) => Ok((true, workspace_id, None)),
        Err(err) => {
            let err_msg = err.to_string();
            if err_msg.contains("already exists") {
                // Branch/worktree collision: create a fresh workspace and retry once.
                let attempt_id = Uuid::new_v4();
                let git_branch_name = deployment
                    .container()
                    .git_branch_from_workspace(&attempt_id, &task.title)
                    .await;
                let agent_working_dir = if repos.len() == 1 {
                    let repo = &repos[0];
                    match &repo.default_working_dir {
                        Some(subdir) => Some(
                            PathBuf::from(&repo.name)
                                .join(subdir)
                                .to_string_lossy()
                                .to_string(),
                        ),
                        None => Some(repo.name.clone()),
                    }
                } else {
                    None
                };
                let retry_ws = Workspace::create(
                    &deployment.db().pool,
                    &CreateWorkspace {
                        branch: git_branch_name,
                        agent_working_dir,
                    },
                    attempt_id,
                    task.id,
                )
                .await?;
                let workspace_repos: Vec<CreateWorkspaceRepo> = repos
                    .iter()
                    .map(|repo| CreateWorkspaceRepo {
                        repo_id: repo.id,
                        target_branch: repo
                            .default_target_branch
                            .clone()
                            .unwrap_or_else(|| "main".to_string()),
                    })
                    .collect();
                WorkspaceRepo::create_many(&deployment.db().pool, retry_ws.id, &workspace_repos)
                    .await?;
                workspace = retry_ws;

                let retry_workspace_id = Some(workspace.id);
                match deployment
                    .container()
                    .start_workspace(&workspace, profile)
                    .await
                {
                    Ok(_) => Ok((true, retry_workspace_id, None)),
                    Err(retry_err) => Ok((
                        false,
                        retry_workspace_id,
                        Some(format!("Workspace start failed after retry: {}", retry_err)),
                    )),
                }
            } else {
                Ok((false, workspace_id, Some(format!("Workspace start failed: {err_msg}"))))
            }
        }
    }
}

pub async fn dispatch_task(
    State(deployment): State<DeploymentImpl>,
    Json(req): Json<StarbusDispatchRequest>,
) -> Result<ResponseJson<ApiResponse<StarbusDispatchResponse>>, ApiError> {
    let task = Task::find_by_id(&deployment.db().pool, req.task_id)
        .await?
        .ok_or_else(|| ApiError::BadRequest(format!("Task not found: {}", req.task_id)))?;

    let actor = req
        .actor
        .as_deref()
        .and_then(normalize_actor_choice)
        .unwrap_or_else(|| "ACTOR_CLAUDE".to_string());
    let hitl_select_actor = req.hitl_select_actor.unwrap_or(false) && req.actor.is_none();
    let (default_role, default_status, default_gate, default_action) =
        infer_dispatch_from_title(&task.title);
    let role = req.role_override.clone().unwrap_or(default_role);
    let planned_status = if req.role_override.is_some() {
        initial_status_for_role(&role)
    } else {
        default_status
    };
    let gate = default_gate;
    let action = req.action_override.clone().unwrap_or(default_action);

    let workspace_root = discover_workspace_root();
    let task_dir = starbus_runs_root(&workspace_root).join(task.id.to_string());
    if !task_dir.exists() {
        let intake_for_skeleton = StarbusIntakeRequest {
            project_id: task.project_id,
            title: task.title.clone(),
            description: task.description.clone(),
            acceptance: None,
            priority: Some("P1".to_string()),
            domain_roles: role_domain_defaults(&role),
            include_recommended_deps: Some(true),
            tags: vec!["dispatch".to_string()],
            set_active: req.set_active,
            default_actor: Some(actor.clone()),
            default_role: Some(role.clone()),
        };
        let _ = write_task_skeleton_files(task.id, &intake_for_skeleton, &planned_status)?;
    }

    let prompt_rel = if let Some(rel) = existing_prompt_relpath(&workspace_root, &task.title, &role) {
        rel
    } else {
        let prompt_abs = task_dir.join("dispatch-prompt.md");
        fs::write(
            &prompt_abs,
            render_dispatch_prompt(&task.title, &role, &action),
        )
        .map_err(|e| ApiError::BadRequest(format!("Failed to write dispatch prompt: {e}")))?;
        prompt_abs
            .strip_prefix(&workspace_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| prompt_abs.to_string_lossy().to_string())
    };

    let next_action = StarbusNextAction {
        actor: actor.clone(),
        role: role.clone(),
        action: action.clone(),
        inputs: vec![
            prompt_rel.clone(),
            format!("docs/starbus/runs/{}/task.md", task.id),
            format!("docs/starbus/runs/{}/context.md", task.id),
        ],
        outputs: vec![
            format!("docs/starbus/runs/{}/03-dev.md", task.id),
            format!("docs/starbus/runs/{}/04-test.md", task.id),
            format!("docs/starbus/runs/{}/06-audit.md", task.id),
        ],
    };

    let existing = Scratch::find_by_id(&deployment.db().pool, task.id, &ScratchType::StarbusTaskState)
        .await?;
    let mut state = if let Some(scratch) = existing {
        match scratch.payload {
            ScratchPayload::StarbusTaskState(data) => data,
            _ => {
                return Err(ApiError::BadRequest(
                    "Scratch payload type mismatch for StarBus task".to_string(),
                ))
            }
        }
    } else {
        StarbusTaskStateData {
            task_id: task.id,
            title: task.title.clone(),
            status: starbus_status_from_task_status(&task.status),
            priority: Some("P1".to_string()),
            active_actor: None,
            active_role: None,
            next_action: None,
            decision_requests: Vec::new(),
            history: Vec::new(),
            step_count: 0,
            gate: Some(gate.clone()),
            tags: Vec::new(),
            domain_roles: role_domain_defaults(&role),
            include_recommended_deps: Some(true),
        }
    };

    let previous_status = state.status.clone();
    state.status = if hitl_select_actor {
        "BLOCKED_HUMAN".to_string()
    } else {
        planned_status.clone()
    };
    state.active_actor = if hitl_select_actor {
        Some("ACTOR_HUMAN".to_string())
    } else {
        Some(actor.clone())
    };
    state.active_role = Some(role.clone());
    state.next_action = Some(next_action);
    state.gate = Some(gate.clone());
    if hitl_select_actor {
        let mut options: Vec<String> = req
            .actor_options
            .clone()
            .unwrap_or_else(default_actor_options)
            .into_iter()
            .filter_map(|raw| normalize_actor_choice(&raw))
            .collect();
        options.sort();
        options.dedup();
        if options.is_empty() {
            options = default_actor_options();
        }
        state.decision_requests.push(db::models::scratch::StarbusDecisionRequest {
            id: format!("DR-ACTOR-{}", chrono::Utc::now().timestamp_millis()),
            question: "Select execution actor before auto start".to_string(),
            options,
            recommended: Some("ACTOR_CLAUDE".to_string()),
            context_refs: vec![
                "HITL_ACTOR_SELECT".to_string(),
                format!("GATE:{}", gate),
                format!("RESUME_STATUS:{planned_status}"),
                format!("ROLE:{role}"),
                format!("ACTION:{action}"),
            ],
            resolved_at: None,
            resolution: None,
        });
    }
    state.history.push(StarbusHistoryEntry {
        ts: chrono::Utc::now().to_rfc3339(),
        from_status: Some(previous_status),
        to_status: Some(state.status.clone()),
        actor: Some("ORCHESTRATOR".to_string()),
        note: Some(if hitl_select_actor {
            "Dispatched by /api/starbus/dispatch (HITL actor selection required)".to_string()
        } else {
            "Dispatched by /api/starbus/dispatch".to_string()
        }),
    });

    let scratch_payload = UpdateScratch {
        payload: ScratchPayload::StarbusTaskState(state.clone()),
    };
    if Scratch::find_by_id(&deployment.db().pool, task.id, &ScratchType::StarbusTaskState)
        .await?
        .is_some()
    {
        Scratch::update(
            &deployment.db().pool,
            task.id,
            &ScratchType::StarbusTaskState,
            &scratch_payload,
        )
        .await?;
    } else {
        Scratch::create(
            &deployment.db().pool,
            task.id,
            &CreateScratch {
                payload: ScratchPayload::StarbusTaskState(state.clone()),
            },
        )
        .await?;
    }

    let mapped = map_starbus_to_task_status(&state.status);
    let _ = Task::update_status(&deployment.db().pool, task.id, mapped).await;

    if req.set_active {
        upsert_global_state(
            &deployment,
            StarbusGlobalStateData {
                active_task_id: Some(task.id),
            },
        )
        .await?;
    }

    let mut started = false;
    let mut workspace_id = None;
    let mut note = None;

    if req.auto_start && !hitl_select_actor {
        let (did_start, ws_id, start_note) = auto_start_task_workspace(&deployment, &task, &actor).await?;
        started = did_start;
        workspace_id = ws_id;
        note = start_note;
    } else if hitl_select_actor {
        note = Some("HITL actor selection created; resolve decision to continue".to_string());
    }

    Ok(ResponseJson(ApiResponse::success(StarbusDispatchResponse {
        task_id: task.id,
        actor,
        role,
        status: state.status.clone(),
        gate,
        prompt_path: prompt_rel,
        workspace_id,
        started,
        note,
    })))
}

pub async fn get_runs(
    State(deployment): State<DeploymentImpl>,
    axum::extract::Path(task_id): axum::extract::Path<Uuid>,
) -> Result<ResponseJson<ApiResponse<StarbusRunsResponse>>, ApiError> {
    let workspaces = Workspace::fetch_all(&deployment.db().pool, Some(task_id))
        .await
        .map_err(ApiError::Workspace)?;
    let latest = ExecutionProcess::find_latest_for_workspaces(&deployment.db().pool, false).await?;

    let infos = workspaces
        .into_iter()
        .map(|ws| {
            if let Some(proc_info) = latest.get(&ws.id) {
                StarbusWorkspaceRunInfo {
                    workspace_id: ws.id,
                    branch: ws.branch,
                    container_ref: ws.container_ref,
                    latest_execution_process_id: Some(proc_info.execution_process_id),
                    latest_session_id: Some(proc_info.session_id),
                    latest_status: Some(format!("{:?}", proc_info.status)),
                    latest_completed_at: proc_info.completed_at.map(|dt| dt.to_rfc3339()),
                    is_running: proc_info.status == ExecutionProcessStatus::Running,
                }
            } else {
                StarbusWorkspaceRunInfo {
                    workspace_id: ws.id,
                    branch: ws.branch,
                    container_ref: ws.container_ref,
                    latest_execution_process_id: None,
                    latest_session_id: None,
                    latest_status: None,
                    latest_completed_at: None,
                    is_running: false,
                }
            }
        })
        .collect::<Vec<_>>();

    Ok(ResponseJson(ApiResponse::success(StarbusRunsResponse {
        task_id,
        workspace_count: infos.len(),
        workspaces: infos,
    })))
}

pub async fn handoff(
    State(deployment): State<DeploymentImpl>,
    Json(req): Json<StarbusHandoffRequest>,
) -> Result<ResponseJson<ApiResponse<StarbusHandoffResponse>>, ApiError> {
    let mut state = get_starbus_task(&deployment, req.task_id).await?;
    let workspace_root = discover_workspace_root();
    let task_dir = starbus_runs_root(&workspace_root).join(req.task_id.to_string());
    fs::create_dir_all(&task_dir)
        .map_err(|e| ApiError::BadRequest(format!("Failed to create task dir: {e}")))?;
    let handoff_path = task_dir.join("handoff.md");
    let mut body = String::new();
    body.push_str("# Handoff\n\n");
    body.push_str(&format!("Task: `{}`\n\n", req.task_id));
    body.push_str("## Summary\n");
    body.push_str(&req.summary);
    body.push('\n');
    if !req.results.is_empty() {
        body.push_str("\n## Results\n");
        for item in &req.results {
            body.push_str(&format!("- {item}\n"));
        }
    }
    if !req.next_steps.is_empty() {
        body.push_str("\n## Next Steps\n");
        for item in &req.next_steps {
            body.push_str(&format!("- {item}\n"));
        }
    }
    fs::write(&handoff_path, body)
        .map_err(|e| ApiError::BadRequest(format!("Failed to write handoff.md: {e}")))?;

    if let Some(status) = req.status.as_ref() {
        let next = normalize_status(status);
        if is_valid_transition(&state.status, &next) {
            state.history.push(StarbusHistoryEntry {
                ts: chrono::Utc::now().to_rfc3339(),
                from_status: Some(state.status.clone()),
                to_status: Some(next.clone()),
                actor: Some("ORCHESTRATOR".to_string()),
                note: Some("Handoff status update".to_string()),
            });
            state.status = next;
        }
    }

    state.history.push(StarbusHistoryEntry {
        ts: chrono::Utc::now().to_rfc3339(),
        from_status: Some(state.status.clone()),
        to_status: Some(state.status.clone()),
        actor: Some("ORCHESTRATOR".to_string()),
        note: Some("Handoff markdown persisted".to_string()),
    });

    Scratch::update(
        &deployment.db().pool,
        state.task_id,
        &ScratchType::StarbusTaskState,
        &UpdateScratch {
            payload: ScratchPayload::StarbusTaskState(state.clone()),
        },
    )
    .await?;

    Ok(ResponseJson(ApiResponse::success(StarbusHandoffResponse {
        task_id: req.task_id,
        handoff_path: format!("docs/starbus/runs/{}/handoff.md", req.task_id),
        status: state.status,
    })))
}

pub fn router(_deployment: &DeploymentImpl) -> Router<DeploymentImpl> {
    Router::new()
        .route("/starbus/state", get(get_starbus_state))
        .route("/starbus/status-mapping", get(get_status_mapping))
        .route("/starbus/intake/preflight", post(intake_preflight))
        .route("/starbus/intake/create", post(intake_create))
        .route("/starbus/dispatch", post(dispatch_task))
        .route("/starbus/run-role-task", post(run_role_task))
        .route("/starbus/runs/{task_id}", get(get_runs))
        .route("/starbus/handoff", post(handoff))
        .route(
            "/starbus/state/sync/project-statuses",
            post(sync_project_statuses),
        )
        .route("/starbus/state/next_action", post(update_next_action))
        .route("/starbus/state/transition", post(transition_state))
        .route("/starbus/state/decision/resolve", post(resolve_decision))
}
