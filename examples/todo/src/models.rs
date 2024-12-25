use break_stack::auth::UserId;
use break_stack::errors::*;
use break_stack::models::DBConn;
use break_stack::models::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TodoItemModel {
    pub id: i64,
    pub description: String,
    pub done: bool,
}

impl Model for TodoItemModel {
    const MODEL_NAME: &'static str = "TodoItem";
}

impl TodoItemModel {
    pub async fn all(conn: &mut DBConn) -> Result<Vec<Self>, ModelError> {
        let items = sqlx::query_as!(Self, "SELECT * FROM todo_items")
            .fetch_all(&mut **conn)
            .await?;

        Ok(items)
    }
}

impl ModelRead for TodoItemModel {
    async fn read(conn: &mut DBConn, id: i64) -> Result<Option<Self>, ModelError> {
        let item = sqlx::query_as!(Self, "SELECT * FROM todo_items WHERE id = ?", id)
            .fetch_optional(&mut **conn)
            .await?;

        Ok(item)
    }
}

#[derive(Deserialize)]
pub struct TodoItemWrite {
    pub description: String,
    #[serde(default)]
    pub done: bool,
}

impl ModelWrite for TodoItemModel {
    type Write = TodoItemWrite;

    async fn write(
        conn: &mut DBConn,
        id: i64,
        data: Self::Write,
    ) -> Result<Option<Self>, ModelError> {
        let item = sqlx::query_as!(
            Self,
            "UPDATE todo_items SET description = ?, done = ? WHERE id = ? RETURNING *",
            data.description,
            data.done,
            id
        )
        .fetch_optional(&mut **conn)
        .await?;

        Ok(item)
    }
}

#[derive(Deserialize)]
pub struct TodoItemCreate {
    pub description: String,
}

impl ModelCreate for TodoItemModel {
    type Create = TodoItemCreate;

    async fn create(conn: &mut DBConn, data: Self::Create) -> Result<Self, ModelError> {
        let item = sqlx::query_as!(
            Self,
            "INSERT INTO todo_items (description, done) VALUES (?, FALSE) RETURNING *",
            data.description,
        )
        .fetch_one(&mut **conn)
        .await?;

        Ok(item)
    }
}

impl AuthModelRead for TodoItemModel {
    async fn can_read(
        _conn: &mut DBConn,
        _id: i64,
        _user_id: Option<UserId>,
    ) -> Result<(), AuthError> {
        Ok(())
    }
}

impl AuthModelWrite for TodoItemModel {
    async fn can_write(
        _conn: &mut DBConn,
        _id: i64,
        _user_id: Option<UserId>,
        _data: &<Self as ModelWrite>::Write,
    ) -> Result<(), AuthError> {
        Ok(())
    }
}

impl AuthModelCreate for TodoItemModel {
    async fn can_create(
        _conn: &mut DBConn,
        _user_id: Option<UserId>,
        _data: &<Self as ModelCreate>::Create,
    ) -> Result<(), AuthError> {
        Ok(())
    }
}
