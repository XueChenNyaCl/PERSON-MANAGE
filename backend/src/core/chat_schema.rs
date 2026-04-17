use crate::core::error::AppError;

pub async fn ensure_chat_schema(pool: &sqlx::PgPool) -> Result<(), AppError> {
    let statements = [
        r#"CREATE TABLE IF NOT EXISTS chat_conversations (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            conversation_type VARCHAR(20) NOT NULL DEFAULT 'direct',
            pair_key VARCHAR(255) NOT NULL UNIQUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
        r#"CREATE TABLE IF NOT EXISTS chat_conversation_members (
            conversation_id UUID NOT NULL,
            user_id UUID NOT NULL,
            joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            last_read_at TIMESTAMPTZ,
            PRIMARY KEY (conversation_id, user_id)
        )"#,
        r#"CREATE TABLE IF NOT EXISTS chat_messages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            conversation_id UUID NOT NULL,
            sender_id UUID NOT NULL,
            content TEXT NOT NULL,
            message_type VARCHAR(20) NOT NULL DEFAULT 'text',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
        r#"DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM pg_constraint WHERE conname = 'fk_chat_conversation_members_conversation_id'
            ) THEN
                ALTER TABLE chat_conversation_members
                ADD CONSTRAINT fk_chat_conversation_members_conversation_id
                FOREIGN KEY (conversation_id) REFERENCES chat_conversations(id) ON DELETE CASCADE;
            END IF;
        END $$;"#,
        r#"DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM pg_constraint WHERE conname = 'fk_chat_conversation_members_user_id'
            ) THEN
                ALTER TABLE chat_conversation_members
                ADD CONSTRAINT fk_chat_conversation_members_user_id
                FOREIGN KEY (user_id) REFERENCES persons(id) ON DELETE CASCADE;
            END IF;
        END $$;"#,
        r#"DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM pg_constraint WHERE conname = 'fk_chat_messages_conversation_id'
            ) THEN
                ALTER TABLE chat_messages
                ADD CONSTRAINT fk_chat_messages_conversation_id
                FOREIGN KEY (conversation_id) REFERENCES chat_conversations(id) ON DELETE CASCADE;
            END IF;
        END $$;"#,
        r#"DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM pg_constraint WHERE conname = 'fk_chat_messages_sender_id'
            ) THEN
                ALTER TABLE chat_messages
                ADD CONSTRAINT fk_chat_messages_sender_id
                FOREIGN KEY (sender_id) REFERENCES persons(id) ON DELETE CASCADE;
            END IF;
        END $$;"#,
        r#"CREATE INDEX IF NOT EXISTS idx_chat_conversation_members_user_id ON chat_conversation_members(user_id)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_chat_messages_conversation_id ON chat_messages(conversation_id)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_chat_messages_sender_id ON chat_messages(sender_id)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_chat_conversations_pair_key ON chat_conversations(pair_key)"#,
        r#"DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM pg_trigger WHERE tgname = 'update_chat_conversations_updated_at'
            ) THEN
                CREATE TRIGGER update_chat_conversations_updated_at
                    BEFORE UPDATE ON chat_conversations
                    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
            END IF;
        END $$;"#,
    ];

    for statement in statements {
        sqlx::query(statement).execute(pool).await?;
    }

    Ok(())
}
