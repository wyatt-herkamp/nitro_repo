use nr_core::{
    database::entities::user::{UserSafeData, UserType, auth_token::AuthToken},
    user::permissions::{HasPermissions, HasUserType, UserPermissions},
};
use serde::{Deserialize, Serialize};
use tracing::{Span, debug, instrument};

use crate::app::NitroRepo;

use super::{AuthenticationError, get_user_and_auth_token, session::Session};
/// Authentication Message for Websockets.
///
/// This type should be added to your WebSocket Message Enum to handle Authentication.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum WebSocketAuthenticationMessage {
    /// An Auth Token was passed under the Authorization Header
    AuthToken(String),
    /// Session Value from Cookie
    Session(String),
}
impl WebSocketAuthenticationMessage {
    #[instrument(skip(self, site), fields(login.type, project_module = "Authentication"))]
    pub async fn attempt_login(
        &self,
        site: &NitroRepo,
    ) -> Result<WebSocketAuthentication, AuthenticationError> {
        match self {
            WebSocketAuthenticationMessage::AuthToken(token) => {
                Span::current().record("login.type", "auth_token");
                let (user, auth_token) = get_user_and_auth_token(token, &site.database).await?;
                debug!(?user, "User Login Via Auth Token");
                let result = WebSocketAuthentication::AuthToken {
                    token: auth_token,
                    user,
                };
                Ok(result)
            }
            WebSocketAuthenticationMessage::Session(session) => {
                Span::current().record("login.type", "session");
                let Some(session) = site.session_manager.get_session(session)? else {
                    return Err(AuthenticationError::Unauthorized);
                };

                let user = UserSafeData::get_by_id(session.user_id, &site.database)
                    .await?
                    .ok_or(AuthenticationError::Unauthorized)?;
                debug!(?user, "User Login Via Session");

                Ok(WebSocketAuthentication::Session { session, user })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebSocketAuthentication {
    /// An Auth Token was passed under the Authorization Header
    AuthToken {
        token: AuthToken,
        user: UserSafeData,
    },
    /// Session Value from Cookie
    Session {
        session: Session,
        user: UserSafeData,
    },
}
impl HasPermissions for WebSocketAuthentication {
    fn user_id(&self) -> Option<i32> {
        match self {
            WebSocketAuthentication::AuthToken { user, .. }
            | WebSocketAuthentication::Session { user, .. } => Some(user.id),
        }
    }

    fn get_permissions(&self) -> Option<UserPermissions> {
        match self {
            WebSocketAuthentication::AuthToken { user, .. }
            | WebSocketAuthentication::Session { user, .. } => user.get_permissions(),
        }
    }
}
impl HasUserType for WebSocketAuthentication {
    type UserType = UserSafeData;

    fn user(&self) -> Option<&Self::UserType> {
        match self {
            WebSocketAuthentication::AuthToken { user, .. }
            | WebSocketAuthentication::Session { user, .. } => Some(user),
        }
    }
}
