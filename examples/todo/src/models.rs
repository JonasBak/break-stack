use break_stack::auth::UserId;
use break_stack::errors::*;
use break_stack::models::*;
use serde::Deserialize;

#[derive(Deserialize, Model, ModelRead, ModelWrite, ModelCreate)]
#[model(name = "TodoItem")]
#[model_read(query = "SELECT * FROM todo_items WHERE id = ?")]
#[model_write(
    data_type = "TodoItemWrite",
    query = "UPDATE todo_items SET description = ?, done = ? WHERE id = ? RETURNING *",
    fields = "data.description, data.done, id"
)]
#[model_create(
    data_type = "TodoItemCreate",
    query = "INSERT INTO todo_items (description, done) VALUES (?, FALSE) RETURNING *",
    fields = "data.description"
)]
pub struct TodoItemModel {
    pub id: i64,
    pub description: String,
    pub done: bool,
}

impl TodoItemModel {
    pub async fn all(conn: &mut DBConn) -> Result<Vec<Self>, ModelError> {
        let items = sqlx::query_as!(Self, "SELECT * FROM todo_items")
            .fetch_all(&mut **conn)
            .await?;

        Ok(items)
    }
}

#[derive(Deserialize)]
pub struct TodoItemWrite {
    pub description: String,
    #[serde(default)]
    pub done: bool,
}

#[derive(Deserialize)]
pub struct TodoItemCreate {
    pub description: String,
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
