use crate::models::*;
use break_stack::components::*;
use break_stack::models::*;
use break_stack::utils::askama::filters;

#[derive(Component)]
#[template(
    source = r#"
        <div hx-target="this" hx-swap="outerHTML">
            {% if item.done %}
                <s>{{ item.description }}</s>
            {% else %}
                <span>{{ item.description }}</span>
            {% endif %}
            <button type="button" hx-get="/htmx/items/{{ item.id }}/edit" class="btn primary">
                Click To Edit
            </button>
        </div>
        "#,
    ext = "html"
)]
pub struct TodoItemViewComponent {
    pub item: TodoItemModel,
}
impl From<TodoItemModel> for TodoItemViewComponent {
    fn from(item: TodoItemModel) -> Self {
        Self { item }
    }
}

#[derive(Component)]
#[template(
    source = r#"
        <form hx-put="/htmx/items/{{ item.id }}" hx-target="this" hx-swap="outerHTML">
            <div>
                <label for="description">Description</label>
                <input type="text" id="description" name="description" value="{{ item.description }}" />
            </div>
            <div>
                <label for="done">Done</label>
                <input type="checkbox" id="done" name="done" value="true" {{ item.done|string_if_true("checked") }} />
            </div>
            <button type="button" hx-get="/htmx/items/{{ item.id }}">Cancel</button>
            <button type="submit">Update</button>
        </form>
        "#,
    ext = "html"
)]
pub struct TodoItemEditComponent {
    pub item: TodoItemModel,
}
impl From<TodoItemModel> for TodoItemEditComponent {
    fn from(item: TodoItemModel) -> Self {
        Self { item }
    }
}

#[derive(Component)]
#[template(
    source = r#"
        <form hx-post="/htmx/items" hx-target="this" hx-swap="outerHTML">
            <div>
                <label for="description">Description</label>
                <input type="text" id="description" name="description" />
            </div>
            <button type="button" hx-get="/htmx/items/button-new">Cancel</button>
            <button type="submit">Create</button>
        </form>
        "#,
    ext = "html"
)]
pub struct TodoItemNewComponent {}

#[derive(Component)]
#[template(
    source = r#"
        <button type="button" hx-get="/htmx/items/new" hx-target="this" hx-swap="outerHTML">New Todo Item</button>
        "#,
    ext = "html"
)]
pub struct TodoItemButtonNewComponent {}

#[derive(Component)]
#[template(
    source = r#"
        {% extends "layout.html" %}

        {% block body %}

            <div hx-get="{{ crate::routes::route_paths::htmx_items_button_new() }}" hx-swap="beforeend" hx-trigger="{{ TodoItemModel::event_created() }}">
                {% for item in todo_items.clone() %}
                    {{ TodoItemViewComponentRef::new(item)|safe }}
                {% endfor %}
                {{ TodoItemButtonNewComponentRef::new()|safe }}
            </div>
        {% endblock %}
        "#,
    ext = "html"
)]
pub struct IndexPageComponent {
    pub todo_items: Vec<TodoItemModel>,
}
