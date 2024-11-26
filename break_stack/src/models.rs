use crate::auth::UserId;
use crate::errors::{AuthError, ModelError};

pub type DBConn = sqlx::pool::PoolConnection<sqlx::Sqlite>;

pub trait Model {
    const MODEL_NAME: &'static str;
    fn event_created() -> String {
        format!("{}Created", Self::MODEL_NAME)
    }
    fn event_updated() -> String {
        format!("{}Updated", Self::MODEL_NAME)
    }
}

pub trait WithOwnerModel: Sized + Model {
    fn owner(
        conn: &mut DBConn,
        id: i64,
    ) -> impl std::future::Future<Output = Result<i64, ModelError>> + Send;
    fn all_for_owner(
        conn: &mut DBConn,
        user_id: i64,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, ModelError>> + Send;
}

/*
pub trait WithRelatedModel: Sized + Model {
    type Related: Sized;
    fn fetch_related(
        &self,
        conn: &mut DBConn,
    ) -> impl std::future::Future<Output = Result<Self::Related, ModelError>> + Send;
}
*/

/// Trait that represents some entity in the database that can be read.
/// This assumes lookup using a numeric id, other cases will need to be
/// implemented outside of this trait.
pub trait ModelRead: Sized + Model {
    fn read(
        conn: &mut DBConn,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Option<Self>, ModelError>> + Send;
    fn read_one(
        conn: &mut DBConn,
        id: i64,
    ) -> impl std::future::Future<Output = Result<Self, ModelError>> + Send {
        async move {
            Self::read(conn, id)
                .await?
                .ok_or_else(|| ModelError::NotFound)
        }
    }
}

/// Trait that represents some entity in the database that can be written to/updated.
/// The associated type `ModelWrite::Write` can be used to allow for updating only a
/// subset of the fields of the "main model". The write function should return the
/// updated object if an object with the given id exists and is updated.
pub trait ModelWrite: Sized + Model {
    type Write: Sized + Send + Sync;
    fn write(
        conn: &mut DBConn,
        id: i64,
        data: Self::Write,
    ) -> impl std::future::Future<Output = Result<Option<Self>, ModelError>> + Send;
    fn write_one(
        conn: &mut DBConn,
        id: i64,
        data: Self::Write,
    ) -> impl std::future::Future<Output = Result<Self, ModelError>> + Send {
        async move {
            Self::write(conn, id, data)
                .await?
                .ok_or_else(|| ModelError::NotFound)
        }
    }
}

/// Trait that represents some entity in the database that can be created.
/// The associated type `ModelCreate::Create` can be used to allow for only submitting a
/// subset of the fields of the "main model", useful when a table uses default values.
pub trait ModelCreate: Sized + Model {
    type Create: Sized + Send + Sync;
    fn create(
        conn: &mut DBConn,
        data: Self::Create,
    ) -> impl std::future::Future<Output = Result<Self, ModelError>> + Send;
}

/// Trait for authentication and authorization checks for reading an object.
pub trait AuthModelRead: ModelRead {
    fn can_read(
        conn: &mut DBConn,
        id: i64,
        user_id: Option<UserId>,
    ) -> impl std::future::Future<Output = Result<(), AuthError>> + Send;
}

/// Trait for authentication and authorization checks for writing/updating an object.
/// The `data` argument contains the data that will be used to call `ModelWrite::write`,
/// this should be inspected to determine if the user has permission to perform the operation.
pub trait AuthModelWrite: ModelWrite {
    fn can_write(
        conn: &mut DBConn,
        id: i64,
        user_id: Option<UserId>,
        data: &<Self as ModelWrite>::Write,
    ) -> impl std::future::Future<Output = Result<(), AuthError>> + Send;
}

/// Trait for authentication and authorization checks for creating an object.
/// The `data` argument contains the data that will be used to call `ModelCreate::create`,
/// this should be inspected to determine if the user has permission to perform the operation.
pub trait AuthModelCreate: ModelCreate {
    fn can_create(
        conn: &mut DBConn,
        user_id: Option<UserId>,
        data: &<Self as ModelCreate>::Create,
    ) -> impl std::future::Future<Output = Result<(), AuthError>> + Send;
}
