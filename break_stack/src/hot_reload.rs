use axum::{response::IntoResponse, routing::get, Router};

fn hot_reload_script() -> String {
    format!(
        r#"
let current_version = "{}".trim();
async function watch() {{
  const response = await fetch("/reload/build_id");
  const version = (await response.text()).trim();
  if (version.length > 0 && version != current_version) {{
    console.log("old version", current_version);
    console.log("new version", version);
    window.location.reload();
  }}
}}

setInterval(watch, 2000);

htmx.logAll();"#,
        hot_reload_build_id()
    )
}

pub fn reload_router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    if hot_reload_enabled() {
        Router::new()
            .route("/build_id", get(|| async { hot_reload_build_id() }))
            .route(
                "/script",
                get(|| async {
                    (
                        [("Content-Type", "application/javascript")],
                        hot_reload_script(),
                    )
                        .into_response()
                }),
            )
    } else {
        Router::new()
    }
}

pub fn hot_reload_build_id() -> &'static str {
    option_env!("HOT_RELOAD_BUILD_ID").unwrap_or("not_set")
}

pub fn hot_reload_enabled() -> bool {
    option_env!("HOT_RELOAD_BUILD_ID").is_some()
}

pub fn hot_reload_script_tag() -> &'static str {
    if hot_reload_enabled() {
        r#"<script src="/reload/script"></script>"#
    } else {
        r#""#
    }
}
