use nr_core::{repository::Visibility, user::permissions::HasPermissions};
use uuid::Uuid;

use crate::app::authentication::Authentication;

pub fn can_read_repository(
    auth: Option<Authentication>,
    visibility: Visibility,
    repository_id: Uuid,
) -> bool {
    match visibility {
        Visibility::Public => true,
        Visibility::Private | Visibility::Hidden => auth.can_read_repository(repository_id),
    }
}
