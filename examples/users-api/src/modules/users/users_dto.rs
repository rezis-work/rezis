use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(min = 2))]
    pub name: String,
    #[validate(email)]
    pub email: String,
}
