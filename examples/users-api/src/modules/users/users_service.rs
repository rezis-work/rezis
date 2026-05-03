use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: u64,
    pub name: &'static str,
}

#[derive(Clone, Default)]
pub struct UsersService;

impl UsersService {
    pub fn new() -> Self {
        Self
    }

    pub async fn find_all(&self) -> Vec<User> {
        vec![
            User { id: 1, name: "Ada" },
            User {
                id: 2,
                name: "Grace",
            },
        ]
    }
}
