use crate::auth::UserId;
use crate::errors::{AuthError, ModelError};
pub use break_stack_macros::{Model, ModelCreate, ModelRead, ModelWrite};

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
    ) -> impl std::future::Future<Output = Result<Option<i64>, ModelError>> + Send;
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

pub trait OwnerAuthModelRead: WithOwnerModel + ModelRead {}

impl<Model: OwnerAuthModelRead> AuthModelRead for Model {
    async fn can_read(
        conn: &mut DBConn,
        id: i64,
        user_id: Option<UserId>,
    ) -> Result<(), AuthError> {
        let Some(user_id) = user_id else {
            return Err(AuthError::Unauthenticated);
        };
        let Some(owner) = Model::owner(conn, id).await? else {
            return Err(AuthError::Unauthorized);
        };

        if owner != *user_id {
            return Err(AuthError::Unauthorized);
        }

        Ok(())
    }
}

pub trait OwnerAuthModelWrite: WithOwnerModel + ModelWrite {}

impl<Model: OwnerAuthModelWrite> AuthModelWrite for Model {
    async fn can_write(
        conn: &mut DBConn,
        id: i64,
        user_id: Option<UserId>,
        _data: &<Self as ModelWrite>::Write,
    ) -> Result<(), AuthError> {
        let Some(user_id) = user_id else {
            return Err(AuthError::Unauthenticated);
        };
        let Some(owner) = Model::owner(conn, id).await? else {
            return Err(AuthError::Unauthorized);
        };

        if owner != *user_id {
            return Err(AuthError::Unauthorized);
        }

        Ok(())
    }
}

pub mod testutils {
    #[macro_export]
    macro_rules! model_read_test_cases {
        ( $model:ident, $conn:expr, [ $( $case_name:literal : $id:literal => $expect_pat:pat $(if $cond:expr)?, )* ] ) => {
            $(
                {
                    println!("Running case '{}'", $case_name);
                    let id = $id;
                    let entry = <$model as break_stack::models::ModelRead>::read($conn, id)
                        .await
                        .unwrap();
                    if !matches!(entry, $expect_pat $(if $cond)*) {
                        panic!("Case '{}'\nEntry returned by ModelRead::read didn't match expected output:\nExpected output to match: {}\nGot entry: {:?}", $case_name, stringify!($expect_pat $(if $cond)*), entry);
                    }
                }
            )*
        }
    }
    pub use model_read_test_cases;

    #[macro_export]
    macro_rules! model_write_test_cases {
        ( $model:ident, $conn:expr, [ $( $case_name:literal : ($id:literal, $write_data:expr) => $expect_pat:pat $(if $cond:expr)?, )* ] ) => {
            $(
                {
                    println!("Running case '{}'", $case_name);
                    let id = $id;
                    let entry = <$model as break_stack::models::ModelWrite>::write($conn, id, $write_data)
                        .await
                        .unwrap();
                    if !matches!(entry, $expect_pat $(if $cond)*) {
                        panic!("Case '{}'\nEntry returned by ModelWrite::write didn't match expected output:\nExpected output to match: {}\nGot entry: {:?}", $case_name, stringify!($expect_pat $(if $cond)*), entry);
                    }
                }
            )*
        }
    }
    pub use model_write_test_cases;

    #[macro_export]
    macro_rules! model_create_test_cases {
        ( $model:ident, $conn:expr, [ $( $case_name:literal : $create_data:expr => $expect_pat:pat $(if $cond:expr)?, )* ] ) => {
            $(
                {
                    println!("Running case '{}'", $case_name);
                    let entry = <$model as break_stack::models::ModelCreate>::create($conn, $create_data)
                        .await
                        .unwrap();
                    if !matches!(entry, $expect_pat $(if $cond)*) {
                        panic!("Case '{}'\nEntry returned by ModelCreate::create didn't match expected output:\nExpected output to match: {}\nGot entry: {:?}", $case_name, stringify!($expect_pat $(if $cond)*), entry);
                    }
                }
            )*
        }
    }
    pub use model_create_test_cases;

    #[macro_export]
    macro_rules! auth_model_read_test_cases {
        ( $model:ident, $conn:expr, [ $( $case_name:literal : ($id:literal, $user:expr) => $expect_pat:pat $(if $cond:expr)?, )* ] ) => {
            $(
                {
                    println!("Running case '{}'", $case_name);
                    let id = $id;
                    let user = $user;
                    let res = <$model as break_stack::models::AuthModelRead>::can_read($conn, id, user)
                        .await;
                    if !matches!(res, $expect_pat $(if $cond)*) {
                        panic!("Case '{}'\nResult returned by AuthModelRead::can_read didn't match expected output:\nExpected output to match: {}\nGot: {:?}", $case_name, stringify!($expect_pat $(if $cond)*), res);
                    }
                }
            )*
        }
    }
    pub use auth_model_read_test_cases;

    #[macro_export]
    macro_rules! auth_model_write_test_cases {
        ( $model:ident, $conn:expr, [ $( $case_name:literal : ($id:literal, $user:expr, $data:expr) => $expect_pat:pat $(if $cond:expr)?, )* ] ) => {
            $(
                {
                    println!("Running case '{}'", $case_name);
                    let id = $id;
                    let user = $user;
                    let res = <$model as break_stack::models::AuthModelWrite>::can_write($conn, id, user, $data)
                        .await;
                    if !matches!(res, $expect_pat $(if $cond)*) {
                        panic!("Case '{}'\nResult returned by AuthModelWrite::can_write didn't match expected output:\nExpected output to match: {}\nGot: {:?}", $case_name, stringify!($expect_pat $(if $cond)*), res);
                    }
                }
            )*
        }
    }
    pub use auth_model_write_test_cases;

    #[macro_export]
    macro_rules! auth_model_create_test_cases {
        ( $model:ident, $conn:expr, [ $( $case_name:literal : ($user:expr, $data:expr) => $expect_pat:pat $(if $cond:expr)?, )* ] ) => {
            $(
                {
                    println!("Running case '{}'", $case_name);
                    let user = $user;
                    let res = <$model as break_stack::models::AuthModelCreate>::can_create($conn, user, $data)
                        .await;
                    if !matches!(res, $expect_pat $(if $cond)*) {
                        panic!("Case '{}'\nResult returned by AuthModelCreate::can_create didn't match expected output:\nExpected output to match: {}\nGot: {:?}", $case_name, stringify!($expect_pat $(if $cond)*), res);
                    }
                }
            )*
        }
    }
    pub use auth_model_create_test_cases;
}
