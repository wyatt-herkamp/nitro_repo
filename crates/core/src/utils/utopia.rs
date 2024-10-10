use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, ToSchema)]
/// Used to Tell Utopia that the type is not standard. It is a generic JSON object.

pub struct AnyType;
