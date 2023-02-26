/// Extract the current user_id from the session.
///
/// Returns:
/// - PublicError::InternalError on failure, or
/// - PublicError::Unauthorized if the user id wasn't found in the session.
macro_rules! user_id {
    ($session: expr) => {
        $session
            .get::<i64>("user_id")
            .map_err(|err| PublicError::from(err))? // Internal Error - failed to get session value
            .ok_or(PublicError::Unauthorized)? // Session key empty, user is unauthenticated
    };
}

pub(crate) use user_id;
