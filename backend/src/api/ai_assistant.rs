use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::error::AppError;

#[derive(Debug, Deserialize)]
pub struct AssistantSuggestionRequest {
    pub page_context: serde_json::Value,
    pub path: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AssistantSuggestionResponse {
    pub suggestion: String,
}

pub async fn get_assistant_suggestion(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(req): Json<AssistantSuggestionRequest>,
) -> Result<Json<AssistantSuggestionResponse>, AppError> {
    let _pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let _route_hint = req.path.as_deref();
    let _widget_name = req.name.as_deref();

    let page = req
        .page_context
        .get("page")
        .and_then(|v| v.as_str())
        .unwrap_or("dashboard");

    // 前端当前传参结构为：{ page, path, data, timestamp }
    // 这里优先使用 data 作为真实页面上下文，同时兼容旧结构（直接传 stats/summary）
    let context = req.page_context.get("data").unwrap_or(&req.page_context);

    let suggestion = build_suggestion(page, context);

    Ok(Json(AssistantSuggestionResponse { suggestion }))
}

fn build_suggestion(page: &str, context: &serde_json::Value) -> String {
    match page {
        "person" => {
            let total = context
                .get("stats")
                .and_then(|s| s.get("total"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            format!(
                "当前人员总数为 {}，你可以让我帮你快速筛选信息不完整的人员，或生成班级人员分布摘要。",
                total
            )
        }
        "attendance" => {
            let present = context
                .get("today_stats")
                .and_then(|s| s.get("present"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let absent = context
                .get("today_stats")
                .and_then(|s| s.get("absent"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            format!(
                "今日出勤 {} 人、缺勤 {} 人。你可以让我直接生成异常考勤分析和处理建议。",
                present, absent
            )
        }
        "notice" => {
            let total = context
                .get("stats")
                .and_then(|s| s.get("total"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            format!(
                "当前公告总数为 {}。你可以让我帮你起草公告标题、优化内容语气，或生成不同对象版本。",
                total
            )
        }
        "class" => {
            let total = context
                .get("stats")
                .and_then(|s| s.get("total_classes"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            format!(
                "当前班级总数为 {}。你可以让我汇总班级信息、识别重点班级，或生成管理建议。",
                total
            )
        }
        "group" => {
            let total = context
                .get("stats")
                .and_then(|s| s.get("total_groups"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            format!(
                "当前小组总数为 {}。你可以让我分析小组规模分布，并给出积分管理建议。",
                total
            )
        }
        _ => {
            let persons = context
                .get("summary")
                .and_then(|s| s.get("total_persons"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            format!(
                "当前系统共有 {} 名人员。你可以告诉我你的目标，我会给出下一步可执行建议。",
                persons
            )
        }
    }
}
