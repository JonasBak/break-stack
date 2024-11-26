#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct UserId(pub i64);

impl std::ops::Deref for UserId {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
