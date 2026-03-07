use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::core::error::AppError;
use crate::models::ai_action::{
    MultiStepSession, MultiStepRequest, MultiStepResponse,
    StepRequest, StepInfo, StepStatus, SessionStatus,
};
use crate::api::ai_actions::{AIActionExecutor, AIActionResponse};

/// AI编排器 - 管理多步骤操作
#[allow(dead_code)]
pub struct AIOrchestrator;

// 使用OnceLock初始化Mutex<HashMap>
fn get_sessions() -> &'static Mutex<HashMap<Uuid, MultiStepSession>> {
    static SESSIONS: OnceLock<Mutex<HashMap<Uuid, MultiStepSession>>> = OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[allow(dead_code)]
impl AIOrchestrator {
    /// 创建新会话
    fn create_session(user_id: Uuid) -> MultiStepSession {
        MultiStepSession {
            session_id: Uuid::new_v4(),
            user_id,
            created_at: Utc::now(),
            status: SessionStatus::InProgress,
            steps: Vec::new(),
        }
    }
    
    /// 保存会话
    fn save_session(session: MultiStepSession) {
        if let Ok(mut sessions) = get_sessions().lock() {
            sessions.insert(session.session_id, session);
        }
    }
    
    /// 获取会话
    fn get_session(session_id: Uuid) -> Option<MultiStepSession> {
        get_sessions().lock().ok()?.get(&session_id).cloned()
    }
    
    /// 获取会话（可变）
    fn get_session_mut(session_id: Uuid) -> Option<MultiStepSession> {
        get_sessions().lock().ok()?.get(&session_id).cloned()
    }
    
    /// 执行单步骤
    async fn execute_step(
        pool: &PgPool,
        step: &StepRequest,
        user_id: Uuid,
        user_name: &str,
    ) -> Result<AIActionResponse, AppError> {
        // 转换action类型
        let api_action = crate::api::ai_actions::AIActionRequest {
            action_type: format!("{:?}", step.action.action_type).to_lowercase(),
            params: step.action.params.clone(),
            reason: step.action.reason.clone(),
        };
        AIActionExecutor::execute(pool, &api_action, user_id, user_name).await
    }
    
    /// 处理多步骤请求
    pub async fn process_multi_step(
        pool: &PgPool,
        request: MultiStepRequest,
        user_id: Uuid,
        user_name: &str,
    ) -> Result<MultiStepResponse, AppError> {
        // 获取或创建会话
        let session_id = request.session_id.unwrap_or_else(Uuid::new_v4);
        
        let mut session = if let Some(existing_session) = Self::get_session(session_id) {
            existing_session
        } else {
            Self::create_session(user_id)
        };
        
        // 更新会话ID
        session.session_id = session_id;
        
        // 检查会话状态
        if session.status == SessionStatus::Completed || 
           session.status == SessionStatus::Failed ||
           session.status == SessionStatus::Cancelled {
            return Err(AppError::InvalidInput(
                format!("会话 {} 已结束，无法继续执行", session_id)
            ));
        }
        
        // 检查依赖步骤是否完成
        if let Some(depends_on) = &request.current_step.depends_on {
            let dependency_completed = session.steps.iter().any(|s| {
                s.step_id == *depends_on && s.status == StepStatus::Completed
            });
            
            if !dependency_completed {
                return Err(AppError::InvalidInput(
                    format!("依赖的步骤 {} 尚未完成", depends_on)
                ));
            }
        }
        
        // 执行当前步骤
        let step_result = Self::execute_step(
            pool,
            &request.current_step,
            user_id,
            user_name
        ).await;
        
        // 创建步骤信息
        let step_info = StepInfo {
            step_id: request.current_step.step_id.clone(),
            step_number: request.current_step.step_number,
            action_type: request.current_step.action.action_type.clone(),
            status: if step_result.is_ok() && step_result.as_ref().unwrap().success {
                StepStatus::Completed
            } else {
                StepStatus::Failed
            },
            result: step_result.as_ref().ok().map(|r| {
                serde_json::json!({
                    "success": r.success,
                    "message": &r.message,
                    "data": &r.data
                })
            }),
            depends_on: request.current_step.depends_on.clone(),
            executed_at: Some(Utc::now()),
        };
        
        // 添加到会话步骤列表
        session.steps.push(step_info.clone());
        
        // 更新会话状态
        let completed_steps = session.steps.iter().filter(|s| s.status == StepStatus::Completed).count();
        let remaining_steps = request.total_steps.saturating_sub(completed_steps);
        
        if remaining_steps == 0 {
            session.status = SessionStatus::Completed;
        } else if step_info.status == StepStatus::Failed {
            // 如果步骤失败，检查是否是关键步骤
            session.status = SessionStatus::Failed;
        }
        
        // 保存会话
        Self::save_session(session.clone());
        
        // 构建响应
        let current_step_result = step_result?;
        
        // 生成下一步建议
        let next_step_suggestions = Self::generate_next_suggestions(
            &session,
            &request.current_step,
            &current_step_result
        );
        
        // 将结果转换为JSON
        let current_step_result_json = serde_json::json!({
            "success": current_step_result.success,
            "message": current_step_result.message,
            "data": current_step_result.data,
            "user_permissions": current_step_result.user_permissions,
            "need_confirmation": current_step_result.need_confirmation,
            "candidates": current_step_result.candidates,
        });
        
        Ok(MultiStepResponse {
            success: current_step_result.success,
            session_id,
            current_step_result: current_step_result_json,
            session_status: session.status.clone(),
            completed_steps,
            remaining_steps,
            next_step_suggestions,
        })
    }
    
    /// 生成下一步建议
    fn generate_next_suggestions(
        session: &MultiStepSession,
        current_step: &StepRequest,
        result: &AIActionResponse,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if !result.success {
            suggestions.push("检查操作参数是否正确".to_string());
            suggestions.push("确认您有执行此操作的权限".to_string());
            return suggestions;
        }
        
        // 根据当前操作类型建议下一步
        let action_type = format!("{:?}", current_step.action.action_type).to_lowercase();
        
        match action_type.as_str() {
            "createperson" => {
                if let Some(data) = &result.data {
                    if let Some(person_type) = data.get("type").and_then(|v| v.as_str()) {
                        match person_type {
                            "student" => {
                                suggestions.push("为该学生创建考勤记录".to_string());
                                suggestions.push("将该学生添加到班级".to_string());
                            }
                            "teacher" => {
                                suggestions.push("为该教师分配部门".to_string());
                                suggestions.push("创建教师的考勤记录".to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }
            "creategroup" => {
                suggestions.push("为小组添加成员".to_string());
                suggestions.push("设置小组积分".to_string());
            }
            "createattendance" => {
                suggestions.push("继续添加其他考勤记录".to_string());
                suggestions.push("查看考勤统计".to_string());
            }
            "createscore" => {
                suggestions.push("继续添加其他积分记录".to_string());
                suggestions.push("查看积分排名".to_string());
            }
            _ => {
                suggestions.push("继续执行其他操作".to_string());
            }
        }
        
        // 如果还有剩余步骤，提示继续
        let completed = session.steps.iter().filter(|s| s.status == StepStatus::Completed).count();
        if completed < session.steps.len() + 1 {
            suggestions.push(format!("继续执行下一步骤（已完成 {}/{}）", completed, session.steps.len() + 1));
        }
        
        suggestions
    }
    
    /// 取消会话
    pub fn cancel_session(session_id: Uuid) -> Result<(), AppError> {
        if let Ok(mut sessions) = get_sessions().lock() {
            if let Some(session) = sessions.get_mut(&session_id) {
                session.status = SessionStatus::Cancelled;
                Ok(())
            } else {
                Err(AppError::NotFound)
            }
        } else {
            Err(AppError::Internal)
        }
    }
    
    /// 获取会话状态
    pub fn get_session_status(session_id: Uuid) -> Result<MultiStepSession, AppError> {
        Self::get_session(session_id).ok_or(AppError::NotFound)
    }
    
    /// 清理过期会话（应该在定时任务中调用）
    pub fn cleanup_expired_sessions(max_age_hours: i64) {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours);
        if let Ok(mut sessions) = get_sessions().lock() {
            sessions.retain(|_, session| session.created_at > cutoff);
        }
    }
}
