use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(unused)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub exp: usize, // expiration timestamp
}

pub trait JwtConfig {
    fn jwt_secret(&self) -> &str;
}

impl<T: JwtConfig> JwtConfig for Arc<T> {
    fn jwt_secret(&self) -> &str {
        self.as_ref().jwt_secret()
    }
}
