use chrono::Utc;

use crate::{
    dto::{LoginInput, RegisterInput},
    error::{Error, Result},
    model::{CreateUserData, User},
    utils::encryption,
};

pub struct AuthService;

impl AuthService {
    pub async fn sign_in(input: LoginInput) -> Result<User> {
        let user = User::find_by_email(&input.email).await?;
        if encryption::verify_password(input.password, user.password.to_owned()).await? {
            Ok(user)
        } else {
            Err(Error::WrongPassword)
        }
    }

    pub async fn sign_up(input: RegisterInput) -> Result<User> {
        if User::find_by_email(&input.email).await.is_ok() {
            return Err(Error::DuplicateUserEmail);
        }
        if User::find_by_name(&input.name).await.is_ok() {
            return Err(Error::DuplicateUserName);
        }

        let data = CreateUserData {
            name: input.name,
            email: input.email,
            password: encryption::hash_password(input.password).await?,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        Ok(User::create(data).await?)
    }
}
