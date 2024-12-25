use crate::components::*;
use crate::models::*;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use break_stack::controllers::*;
use break_stack::errors::*;

pub type HtmxTodoItemViewController =
    ComponentFromModelController<TodoItemModel, TodoItemViewComponent>;

pub type HtmxTodoItemEditController =
    ComponentFromModelController<TodoItemModel, TodoItemEditComponent>;

pub async fn get_htmx_items_new() -> AppResult<Response> {
    Ok(TodoItemNewComponentRef::new().into_response())
}

pub async fn get_htmx_items_button_new() -> AppResult<Response> {
    Ok(TodoItemButtonNewComponentRef::new().into_response())
}

pub async fn get_index_page(state: State<crate::AppState>) -> AppResult<Response> {
    let mut conn = state.conn().await?;

    let todo_items = TodoItemModel::all(&mut conn).await?;

    Ok(IndexPageComponent { todo_items }.into_response())
}
