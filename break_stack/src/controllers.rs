use crate::auth::UserId;
use crate::components::Component;
use crate::errors::*;
use crate::models::DBConn;
use crate::models::*;
use axum::{extract::Path, http::header::HeaderValue, response::Response, Form};

pub trait ModelController: Send + Sync + Sized {
    type Model: Send + Sync + Sized;
    fn build_response(
        conn: &mut DBConn,
        user_id: Option<UserId>,
        m: Self::Model,
    ) -> impl std::future::Future<Output = AppResult<Response>> + Send;
}

pub trait InitController {
    type Init: Send + Sync + Sized;
    fn build_response(
        conn: &mut DBConn,
        data: Self::Init,
        user_id: Option<UserId>,
    ) -> impl std::future::Future<Output = AppResult<Response>> + Send;
}

pub async fn model_controller_read<H: ModelController<Model: AuthModelRead>>(
    mut conn: DBConn,
    id: Path<<H::Model as Model>::ID>,
    user_id: Option<UserId>,
) -> AppResult<Response> {
    <H::Model as AuthModelRead>::can_read(&mut conn, *id, user_id).await?;

    let item = <H::Model as ModelRead>::read(&mut conn, *id)
        .await?
        .ok_or_else(|| AppError::NotFound)?;
    H::build_response(&mut conn, user_id, item).await
}

pub async fn model_controller_write<H: ModelController<Model: AuthModelWrite>>(
    mut conn: DBConn,
    id: Path<<H::Model as Model>::ID>,
    user_id: Option<UserId>,
    Form(data): Form<<H::Model as ModelWrite>::Write>,
) -> AppResult<Response> {
    <H::Model as AuthModelWrite>::can_write(&mut conn, *id, user_id, &data).await?;

    let item = <H::Model as ModelWrite>::write(&mut conn, *id, data)
        .await?
        .ok_or_else(|| AppError::NotFound)?;
    let mut response = H::build_response(&mut conn, user_id, item).await?;
    response.headers_mut().insert(
        "HX-Trigger",
        <H::Model as Model>::event_updated()
            .parse::<HeaderValue>()
            .map_err(|e| {
                AppError::Internal(format!(
                    "failed to build HX-Trigger header: {}",
                    e.to_string()
                ))
            })?,
    );
    Ok(response)
}

pub async fn model_controller_create<H: ModelController<Model: AuthModelCreate>>(
    mut conn: DBConn,
    user_id: Option<UserId>,
    Form(data): Form<<H::Model as ModelCreate>::Create>,
) -> AppResult<Response> {
    <H::Model as AuthModelCreate>::can_create(&mut conn, user_id, &data).await?;

    let item = <H::Model as ModelCreate>::create(&mut conn, data).await?;

    let mut response = H::build_response(&mut conn, user_id, item).await?;
    response.headers_mut().insert(
        "HX-Trigger",
        <H::Model as Model>::event_created()
            .parse::<HeaderValue>()
            .map_err(|e| {
                AppError::Internal(format!(
                    "failed to build HX-Trigger header: {}",
                    e.to_string()
                ))
            })?,
    );
    Ok(response)
}

pub async fn model_controller_delete<H: ModelController<Model: AuthModelDelete>>(
    mut conn: DBConn,
    id: Path<<H::Model as Model>::ID>,
    user_id: Option<UserId>,
) -> AppResult<Response> {
    <H::Model as AuthModelDelete>::can_delete(&mut conn, *id, user_id).await?;

    let item = <H::Model as ModelDelete>::delete(&mut conn, *id).await?;

    let mut response = H::build_response(&mut conn, user_id, item).await?;
    response.headers_mut().insert(
        "HX-Trigger",
        <H::Model as Model>::event_deleted()
            .parse::<HeaderValue>()
            .map_err(|e| {
                AppError::Internal(format!(
                    "failed to build HX-Trigger header: {}",
                    e.to_string()
                ))
            })?,
    );
    Ok(response)
}

pub async fn init_controller_from_query<C: InitController>(
    mut conn: DBConn,
    user_id: Option<UserId>,
    Form(data): Form<<C as InitController>::Init>,
) -> AppResult<Response> {
    <C as InitController>::build_response(&mut conn, data, user_id).await
}

pub struct ComponentFromModelController<
    Model: Send + Sync + Sized,
    Comp: Component + From<Model> + Send + Sync + Sized,
>(Model, Comp);
impl<Model: Send + Sync + Sized, Comp: Component + From<Model> + Send + Sync + Sized>
    ModelController for ComponentFromModelController<Model, Comp>
{
    type Model = Model;

    async fn build_response(
        _conn: &mut DBConn,
        _user_id: Option<UserId>,
        m: Self::Model,
    ) -> AppResult<Response> {
        Ok(<Comp as From<Model>>::from(m).into_response())
    }
}
