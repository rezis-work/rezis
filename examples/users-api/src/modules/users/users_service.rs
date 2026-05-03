use serde::Serialize;

use super::users_dto::CreateUserDto;

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

#[derive(Clone, Default)]
pub struct UsersService;

impl UsersService {
    pub fn new() -> Self {
        Self
    }

    pub async fn find_all(&self) -> Vec<User> {
        vec![
            User {
                id: 1,
                name: "Ada".into(),
                email: "ada@example.com".into(),
            },
            User {
                id: 2,
                name: "Grace".into(),
                email: "grace@example.com".into(),
            },
        ]
    }

    pub async fn create(&self, dto: CreateUserDto) -> User {
        User {
            id: 3,
            name: dto.name,
            email: dto.email,
        }
    }
}
