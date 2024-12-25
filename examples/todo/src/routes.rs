use crate::controllers::*;
use crate::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use break_stack::controllers::*;
use break_stack::routes::build_router;

build_router! {
    AppState,
    (index, "/", (), get(get_index_page)),
    (htmx_items, "/htmx/items", (), post(model_controller_create::<HtmxTodoItemViewController>)),
    (htmx_items_new, "/htmx/items/new", (), get(get_htmx_items_new)),
    (htmx_items_button_new, "/htmx/items/button-new", (), get(get_htmx_items_button_new)),
    (htmx_items_id, "/htmx/items/{}", (path -> id: &i64 => ":id"), get(model_controller_read::<HtmxTodoItemViewController>).put(model_controller_write::<HtmxTodoItemViewController>)),
    (htmx_items_id_edit, "/htmx/items/{}/edit", (path -> id: &i64 => ":id"), get(model_controller_read::<HtmxTodoItemEditController>)),
}
