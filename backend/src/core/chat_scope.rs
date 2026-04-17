use std::collections::HashSet;

use uuid::Uuid;

use crate::core::error::AppError;

pub async fn get_chat_targets(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    role: &str,
) -> Result<Vec<Uuid>, AppError> {
    match role {
        "student" => get_student_chat_targets(pool, user_id).await,
        "teacher" => get_teacher_chat_targets(pool, user_id).await,
        "parent" | "admin" => Ok(Vec::new()),
        _ => Ok(Vec::new()),
    }
}

pub async fn ensure_conversation_member(
    pool: &sqlx::PgPool,
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1
            FROM chat_conversation_members
            WHERE conversation_id = $1 AND user_id = $2
        )",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if exists {
        Ok(())
    } else {
        Err(AppError::Auth("无权访问该会话".to_string()))
    }
}

async fn get_student_chat_targets(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let class_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT class_id FROM students WHERE person_id = $1 AND class_id IS NOT NULL",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    let Some(class_id) = class_id else {
        return Ok(Vec::new());
    };

    let teacher_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT teacher_id
         FROM (
             SELECT tc.teacher_id AS teacher_id
             FROM teacher_class tc
             WHERE tc.class_id = $1
             UNION
             SELECT c.teacher_id AS teacher_id
             FROM classes c
             WHERE c.id = $1 AND c.teacher_id IS NOT NULL
         ) t",
    )
    .bind(class_id)
    .fetch_all(pool)
    .await?;

    let student_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT person_id FROM students WHERE class_id = $1 AND person_id <> $2",
    )
    .bind(class_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut ids = HashSet::new();
    for id in teacher_ids.into_iter().chain(student_ids.into_iter()) {
        if id != user_id {
            ids.insert(id);
        }
    }

    Ok(ids.into_iter().collect())
}

async fn get_teacher_chat_targets(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let class_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT class_id
         FROM (
             SELECT tc.class_id AS class_id FROM teacher_class tc WHERE tc.teacher_id = $1
             UNION
             SELECT c.id AS class_id FROM classes c WHERE c.teacher_id = $1
         ) cls",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut ids = HashSet::new();

    for class_id in class_ids {
        let student_ids =
            sqlx::query_scalar::<_, Uuid>("SELECT person_id FROM students WHERE class_id = $1")
                .bind(class_id)
                .fetch_all(pool)
                .await?;

        for id in student_ids {
            if id != user_id {
                ids.insert(id);
            }
        }
    }

    Ok(ids.into_iter().collect())
}
