# break-stack

Library/collection of utils for fullstack web development in rust.

It's only intended to be used for small hobby-projects. It currently only supports sqlite, because sqlite + [litestream](https://github.com/benbjohnson/litestream) = :chef-kiss: for smaller projects.

It's called break-stack because as much of the stack as possible, from database queries to html templating, should be validated at compile-time, and therefore the build should break if you introduce certain types of bugs.

This is done by using [sqlx compile-time verified queries](https://github.com/launchbadge/sqlx?tab=readme-ov-file#compile-time-verification) and [askama for compile-time verified templates](https://github.com/rinja-rs/askama) on top of [axum](https://github.com/tokio-rs/axum).

It is intended to be used with [htmx](https://htmx.org/) to create interactivity in the frontend.

This library mostly provides traits, functions, and macros to develop MVC-style applications.

## Models

Models can be defined like this:

```rust
#[derive(Model, ModelRead, ModelWrite, ModelCreate)]
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
```

This implements functionality for create (using the `TodoItemCreate`), read (returning `TodoItemModel`), and update (using the `TodoItemWrite`), for `TodoItemModel`.

## View/Components

A view/component can be defined like this:

```rust
#[derive(Component)]
#[template(
    source = r#"
        <div hx-target="this" hx-swap="outerHTML">
            {% if item.done %}
                <s>{{ item.description }}</s>
            {% else %}
                <span>{{ item.description }}</span>
            {% endif %}
            <button type="button" hx-get="{{ crate::routes::route_paths::htmx_items_id_edit(item.id) }}">
                Click To Edit
            </button>
        </div>
        "#,
    ext = "html"
)]
pub struct TodoItemViewComponent {
    pub item: TodoItemModel,
}
```

This can either be included from other components, or be exposed as a "htmx endpoint" with a controller.

Deriving `Component` creates a new struct with the `Ref` suffix, in this case `TodoItemViewComponentRef`. This struct has the same number of fields, with the same names, as the original, but where the types are referenced instead of owned. This is also the struct that actually implements the template. This is to make it easier to call the template without needing to implement `Clone` and clone a bunch of data each time you want to render a component.

## Controllers

There are a couple of ways to define controllers.

"Model-based" controllers do CRUD-operations on a model, then let's you decide how the output of that operation is turned into a response.

If you have a simple component you want to expose and implement `From<SomeModel>` for that component, you can create a controller like this:

```rust
pub type HtmxTodoItemViewController = ComponentFromModelController<TodoItemModel, TodoItemViewComponent>;
```

Or if you need to fetch some more related data, or do some other stuff, you can do:

```rust
struct HtmxTodoItemViewController;
impl ModelController for HtmxTodoItemViewController {
    type Model = TodoItemModel;

    async fn build_response(
        conn: &mut break_stack::models::DBConn,
        user_id: Option<break_stack::auth::UserId>,
        item: Self::Model,
    ) -> AppResult<Response> {

        // Do other stuff

        Ok(TodoItemViewComponent { item }.into_response())
    }
}
```

These can then be exposed like this:

```rust
build_router! {
    AppState,

    // ...

    (htmx_items, "/htmx/items", (),
        post(model_controller_create::<HtmxTodoItemViewController>)
    ),
    (htmx_items_id, "/htmx/items/{}", (path -> id: &i64 => ":id"),
        get(model_controller_read::<HtmxTodoItemViewController>)
            .put(model_controller_write::<HtmxTodoItemViewController>)
            .delete(model_controller_delete::<HtmxTodoItemViewController>)
    ),
}
```

This will create a router with endpoints CRUD operations on the `TodoItem` model at `/htmx/items` and `/htmx/items/:id`, which returns the `TodoItemViewComponent`. It also creates the functions `route_paths::htmx_items()` and `route_paths::htmx_items_id(i64)` that can be used to reference the path of the endpoints from components or other controllers.

### Auth

When using "model-based" controllers you'll need to implement `AuthModel{Create,Read,Write,Delete}` to handle authentication.

If you have an "an instance of this model has an owner, and only the owner is able to do CRUD-operations on it"-type of model, you can implement `WithOwnerModel` for the model, and then implement the "marker-ish" traits `OwnerAuthModel{Create,Read,Write,Delete}`, which automatically implements the corresponding `AuthModel{Create,Read,Write,Delete}`.

## TODOs

- Improve ergonomics of iterators and weird types in components
  - switching from askama to rinja might help a bit
- Allow concurrent db queries in controllers
- More tests in break-stack
- More test-utilities for apps using break-stack
- Some opinionated way of handling errors in the frontend
- Some tools around forms to ensure all fields are included, named correctly, has correct type...
