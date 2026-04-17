use sqlx::Row;
use uuid::Uuid;

use crate::core::error::AppError;

pub async fn get_accessible_class_ids(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    role: &str,
) -> Result<Vec<Uuid>, AppError> {
    if role == "admin" {
        let rows = sqlx::query("SELECT id FROM classes")
            .fetch_all(pool)
            .await?;

        return Ok(rows.into_iter().map(|row| row.get("id")).collect());
    }

    let rows = match role {
        "teacher" => {
            sqlx::query(
                "SELECT DISTINCT class_id
                 FROM teacher_class
                 WHERE teacher_id = $1",
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?
        }
        "student" => {
            sqlx::query(
                "SELECT class_id
                 FROM students
                 WHERE person_id = $1 AND class_id IS NOT NULL",
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?
        }
        "parent" => {
            sqlx::query(
                "SELECT DISTINCT s.class_id
                 FROM student_parent sp
                 JOIN students s ON sp.student_id = s.person_id
                 WHERE sp.parent_id = $1 AND s.class_id IS NOT NULL",
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?
        }
        _ => Vec::new(),
    };

    Ok(rows.into_iter().map(|row| row.get("class_id")).collect())
}

pub async fn ensure_class_access(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    role: &str,
    class_id: Uuid,
) -> Result<(), AppError> {
    if role == "admin" {
        return Ok(());
    }

    let class_ids = get_accessible_class_ids(pool, user_id, role).await?;
    if class_ids.contains(&class_id) {
        Ok(())
    } else {
        Err(AppError::Auth("无权访问该班级数据".to_string()))
    }
}

pub async fn get_group_class_id(pool: &sqlx::PgPool, group_id: Uuid) -> Result<Uuid, AppError> {
    let row = sqlx::query("SELECT class_id FROM class_groups WHERE id = $1")
        .bind(group_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(row.get("class_id"))
}

pub async fn ensure_group_access(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    role: &str,
    group_id: Uuid,
) -> Result<Uuid, AppError> {
    let class_id = get_group_class_id(pool, group_id).await?;
    ensure_class_access(pool, user_id, role, class_id).await?;
    Ok(class_id)
}
