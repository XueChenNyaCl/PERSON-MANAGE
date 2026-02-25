use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Score {
    pub id: Uuid,
    pub person_id: Uuid,
    pub group_id: Option<Uuid>,
    pub score_type: String, // personal, group, class, dormitory
    pub value: i32,
    pub reason: String,
    pub event_id: Option<Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScoreCreate {
    pub person_id: Uuid,
    pub group_id: Option<Uuid>,
    pub score_type: String,
    pub value: i32,
    pub reason: String,
    pub event_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScoreResponse {
    pub id: Uuid,
    pub person_id: Uuid,
    pub group_id: Option<Uuid>,
    pub score_type: String,
    pub value: i32,
    pub reason: String,
    pub event_id: Option<Uuid>,
    pub created_at: chrono::NaiveDateTime,
}

impl From<Score> for ScoreResponse {
    fn from(score: Score) -> Self {
        Self {
            id: score.id,
            person_id: score.person_id,
            group_id: score.group_id,
            score_type: score.score_type,
            value: score.value,
            reason: score.reason,
            event_id: score.event_id,
            created_at: score.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScoreSummary {
    pub person_id: Uuid,
    pub total_score: i32,
    pub positive_count: i32,
    pub negative_count: i32,
}
