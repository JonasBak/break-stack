use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    Form,
};
use break_stack::auth::*;
use break_stack::controllers::*;
use break_stack::errors::*;
use break_stack::models::*;
use http_body_util::BodyExt;

struct TestModel {
    id: i64,
    field: i64,
}
impl Model for TestModel {
    type ID = i64;

    const MODEL_NAME: &'static str = "Test";
}
impl ModelRead for TestModel {
    async fn read(_conn: &mut DBConn, id: i64) -> Result<Option<Self>, ModelError> {
        match id {
            10..20 => Ok(None),
            20..30 => Err(ModelError::DB(sqlx::Error::WorkerCrashed)),
            _ => Ok(Some(Self { id, field: 0 })),
        }
    }
}
impl AuthModelRead for TestModel {
    async fn can_read(
        _conn: &mut DBConn,
        id: i64,
        user_id: Option<UserId>,
    ) -> Result<(), AuthError> {
        match (id, user_id) {
            (_, None) => Err(AuthError::Unauthenticated),
            (30..40, Some(_)) => Err(AuthError::Model(ModelError::DB(sqlx::Error::WorkerCrashed))),
            (id, Some(user_id)) if id == *user_id => Ok(()),
            _ => Err(AuthError::Unauthorized),
        }
    }
}
impl ModelWrite for TestModel {
    type Write = i64;

    async fn write(
        _conn: &mut DBConn,
        id: i64,
        data: Self::Write,
    ) -> Result<Option<Self>, ModelError> {
        match id {
            10..20 => Ok(None),
            20..30 => Err(ModelError::DB(sqlx::Error::WorkerCrashed)),
            _ => Ok(Some(Self { id, field: data })),
        }
    }
}
impl AuthModelWrite for TestModel {
    async fn can_write(
        _conn: &mut DBConn,
        id: i64,
        user_id: Option<UserId>,
        data: &<Self as ModelWrite>::Write,
    ) -> Result<(), AuthError> {
        match (id, user_id) {
            (_, None) => Err(AuthError::Unauthenticated),
            (30..40, Some(_)) => Err(AuthError::Model(ModelError::DB(sqlx::Error::WorkerCrashed))),
            (id, Some(user_id)) if id == *user_id && *data >= 0 => Ok(()),
            _ => Err(AuthError::Unauthorized),
        }
    }
}
impl ModelCreate for TestModel {
    type Create = i64;

    async fn create(_conn: &mut DBConn, data: Self::Create) -> Result<Self, ModelError> {
        match data {
            20..30 => Err(ModelError::DB(sqlx::Error::WorkerCrashed)),
            _ => Ok(Self { id: data, field: 0 }),
        }
    }
}
impl AuthModelCreate for TestModel {
    async fn can_create(
        _conn: &mut DBConn,
        user_id: Option<UserId>,
        data: &<Self as ModelCreate>::Create,
    ) -> Result<(), AuthError> {
        match (data, user_id) {
            (_, None) => Err(AuthError::Unauthenticated),
            (30..40, Some(_)) => Err(AuthError::Model(ModelError::DB(sqlx::Error::WorkerCrashed))),
            (data, Some(_)) if *data >= 0 => Ok(()),
            _ => Err(AuthError::Unauthorized),
        }
    }
}

struct TestModelController;

impl ModelController for TestModelController {
    type Model = TestModel;

    async fn build_response(
        _conn: &mut DBConn,
        user_id: Option<UserId>,
        m: Self::Model,
    ) -> AppResult<Response> {
        let user_id = user_id.as_deref().copied().unwrap_or(999);
        Ok(format!("{}:{}:{}", user_id, m.id, m.field).into_response())
    }
}

#[sqlx::test]
async fn test_model_controller_read(pool: sqlx::pool::Pool<sqlx::Sqlite>) {
    for (case, id, user_id, expect) in [
        ("User 0 can read own id", 0, Some(0), Ok("0:0:0")),
        ("User 1 can read own id", 1, Some(1), Ok("1:1:0")),
        ("User 2 can read own id", 2, Some(2), Ok("2:2:0")),
        (
            "Unauthenticated user can't read id",
            0,
            None,
            Err(AppError::Auth(AuthError::Unauthenticated)),
        ),
        (
            "User 0 can't read id 1",
            1,
            Some(0),
            Err(AppError::Auth(AuthError::Unauthorized)),
        ),
        (
            "User 1 can't read id 2",
            2,
            Some(1),
            Err(AppError::Auth(AuthError::Unauthorized)),
        ),
        (
            "User 2 can't read id 3",
            3,
            Some(2),
            Err(AppError::Auth(AuthError::Unauthorized)),
        ),
        (
            "Unauthenticated user reads id that crash on read",
            20,
            None,
            Err(AppError::Auth(AuthError::Unauthenticated)),
        ),
        (
            "Authenticated user reads unauthorized id that crash on read",
            20,
            Some(0),
            Err(AppError::Auth(AuthError::Unauthorized)),
        ),
        (
            "Authorized user reads allowed id that crash on read",
            20,
            Some(20),
            Err(AppError::Model(ModelError::DB(sqlx::Error::WorkerCrashed))),
        ),
        (
            "Unauthenticated user reads id that crash on can_read",
            30,
            None,
            Err(AppError::Auth(AuthError::Unauthenticated)),
        ),
        (
            "Authenticated user reads unauthorized id that crash on can_read",
            30,
            Some(0),
            Err(AppError::Auth(AuthError::Model(ModelError::DB(
                sqlx::Error::WorkerCrashed,
            )))),
        ),
        (
            "Authorized user reads allowed id that crash on can_read",
            30,
            Some(30),
            Err(AppError::Auth(AuthError::Model(ModelError::DB(
                sqlx::Error::WorkerCrashed,
            )))),
        ),
    ] {
        println!("Running test case '{}'", case);
        let conn = pool.acquire().await.unwrap();
        let output =
            model_controller_read::<TestModelController>(conn, Path(id), user_id.map(UserId)).await;
        match (&output, &expect) {
            (Ok(response), Ok(expected)) => {
                assert_eq!(response.status(), 200);
                let body = output
                    .unwrap()
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes();
                let body = String::from_utf8(body.to_vec()).unwrap();
                assert_eq!(&body, expected);
            }
            (Err(AppError::Model(ref err)), Err(AppError::Model(ref expected))) => {
                assert_eq!(
                    std::mem::discriminant(err),
                    std::mem::discriminant(expected)
                );
            }
            (Err(AppError::Auth(ref err)), Err(AppError::Auth(ref expected))) => {
                assert_eq!(
                    std::mem::discriminant(err),
                    std::mem::discriminant(expected)
                );
            }
            _ => {
                panic!("Got response:\n{:?}\nExpected:\n{:?}", output, expect);
            }
        }
    }
}

#[sqlx::test]
async fn test_model_controller_write(pool: sqlx::pool::Pool<sqlx::Sqlite>) {
    for (case, id, user_id, data, expect) in [
        (
            "User 0 can write legal data to own id",
            0,
            Some(0),
            1,
            Ok("0:0:1"),
        ),
        (
            "User 1 can write legal data to own id",
            1,
            Some(1),
            1,
            Ok("1:1:1"),
        ),
        (
            "User 2 can write legal data to own id",
            2,
            Some(2),
            1,
            Ok("2:2:1"),
        ),
        (
            "Unauthenticated user can't write legal data to id",
            0,
            None,
            1,
            Err(AppError::Auth(AuthError::Unauthenticated)),
        ),
        (
            "User 0 can't write legal data to id 1",
            1,
            Some(0),
            1,
            Err(AppError::Auth(AuthError::Unauthorized)),
        ),
        (
            "User 1 can't write legal data to id 2",
            2,
            Some(1),
            1,
            Err(AppError::Auth(AuthError::Unauthorized)),
        ),
        (
            "User 2 can't write legal data to id 3",
            3,
            Some(2),
            1,
            Err(AppError::Auth(AuthError::Unauthorized)),
        ),
        (
            "Unauthenticated user writes id that crash on write",
            20,
            None,
            1,
            Err(AppError::Auth(AuthError::Unauthenticated)),
        ),
        (
            "Authenticated user writes unauthorized id that crash on write",
            20,
            Some(0),
            1,
            Err(AppError::Auth(AuthError::Unauthorized)),
        ),
        (
            "Authorized user writes allowed id that crash on write",
            20,
            Some(20),
            1,
            Err(AppError::Model(ModelError::DB(sqlx::Error::WorkerCrashed))),
        ),
        (
            "Unauthenticated user writes id that crash on can_write",
            30,
            None,
            1,
            Err(AppError::Auth(AuthError::Unauthenticated)),
        ),
        (
            "Authenticated user writes unauthorized id that crash on can_write",
            30,
            Some(0),
            1,
            Err(AppError::Auth(AuthError::Model(ModelError::DB(
                sqlx::Error::WorkerCrashed,
            )))),
        ),
        (
            "Authorized user writes allowed id that crash on can_write",
            30,
            Some(30),
            1,
            Err(AppError::Auth(AuthError::Model(ModelError::DB(
                sqlx::Error::WorkerCrashed,
            )))),
        ),
        (
            "Authorized user can't write illegal data to owned id",
            0,
            Some(0),
            -1,
            Err(AppError::Auth(AuthError::Unauthorized)),
        ),
        (
            "Unauthorized user can't write illegal data",
            0,
            Some(1),
            -1,
            Err(AppError::Auth(AuthError::Unauthorized)),
        ),
        (
            "Unauthenticated user can't write illegal data",
            0,
            None,
            -1,
            Err(AppError::Auth(AuthError::Unauthenticated)),
        ),
    ] {
        println!("Running test case '{}'", case);
        let conn = pool.acquire().await.unwrap();
        let output = model_controller_write::<TestModelController>(
            conn,
            Path(id),
            user_id.map(UserId),
            Form(data),
        )
        .await;
        match (&output, &expect) {
            (Ok(response), Ok(expected)) => {
                assert_eq!(response.status(), 200);
                let body = output
                    .unwrap()
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes();
                let body = String::from_utf8(body.to_vec()).unwrap();
                assert_eq!(&body, expected);
            }
            (Err(AppError::Model(ref err)), Err(AppError::Model(ref expected))) => {
                assert_eq!(
                    std::mem::discriminant(err),
                    std::mem::discriminant(expected)
                );
            }
            (Err(AppError::Auth(ref err)), Err(AppError::Auth(ref expected))) => {
                assert_eq!(
                    std::mem::discriminant(err),
                    std::mem::discriminant(expected)
                );
            }
            _ => {
                panic!("Got response:\n{:?}\nExpected:\n{:?}", output, expect);
            }
        }
    }
}

#[sqlx::test]
async fn test_model_controller_create(pool: sqlx::pool::Pool<sqlx::Sqlite>) {
    for (case, user_id, data, expect) in [
        ("User 0 can create legal data", Some(0), 1, Ok("0:1:0")),
        ("User 1 can create legal data", Some(1), 1, Ok("1:1:0")),
        ("User 2 can create legal data", Some(2), 1, Ok("2:1:0")),
        (
            "Authenticated user can't write illegal data",
            Some(0),
            -1,
            Err(AppError::Auth(AuthError::Unauthorized)),
        ),
        (
            "Unauthenticated user can't write illegal data",
            None,
            -1,
            Err(AppError::Auth(AuthError::Unauthenticated)),
        ),
        (
            "Unauthenticated user creates legal data that crash on create",
            None,
            20,
            Err(AppError::Auth(AuthError::Unauthenticated)),
        ),
        (
            "Authorized user creates legal data that crash on create",
            Some(20),
            20,
            Err(AppError::Model(ModelError::DB(sqlx::Error::WorkerCrashed))),
        ),
        (
            "Authorized user creates illegal data that crash on create",
            Some(21),
            20,
            Err(AppError::Model(ModelError::DB(sqlx::Error::WorkerCrashed))),
        ),
        (
            "Unauthenticated user creates legal data that crash on can_create",
            None,
            30,
            Err(AppError::Auth(AuthError::Unauthenticated)),
        ),
        (
            "Authenticated user creates legal data that crash on can_create",
            Some(30),
            30,
            Err(AppError::Auth(AuthError::Model(ModelError::DB(
                sqlx::Error::WorkerCrashed,
            )))),
        ),
        (
            "Authenticated user creates lllegal data that crash on can_create",
            Some(31),
            30,
            Err(AppError::Auth(AuthError::Model(ModelError::DB(
                sqlx::Error::WorkerCrashed,
            )))),
        ),
    ] {
        println!("Running test case '{}'", case);
        let conn = pool.acquire().await.unwrap();
        let output =
            model_controller_create::<TestModelController>(conn, user_id.map(UserId), Form(data))
                .await;
        match (&output, &expect) {
            (Ok(response), Ok(expected)) => {
                assert_eq!(response.status(), 200);
                let body = output
                    .unwrap()
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes();
                let body = String::from_utf8(body.to_vec()).unwrap();
                assert_eq!(&body, expected);
            }
            (Err(AppError::Model(ref err)), Err(AppError::Model(ref expected))) => {
                assert_eq!(
                    std::mem::discriminant(err),
                    std::mem::discriminant(expected)
                );
            }
            (Err(AppError::Auth(ref err)), Err(AppError::Auth(ref expected))) => {
                assert_eq!(
                    std::mem::discriminant(err),
                    std::mem::discriminant(expected)
                );
            }
            _ => {
                panic!("Got response:\n{:?}\nExpected:\n{:?}", output, expect);
            }
        }
    }
}
