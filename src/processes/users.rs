use uzers::{Users, UsersCache};

pub struct UserResolver {
    cache: UsersCache,
}

impl UserResolver {
    pub fn new() -> Self {
        Self {
            cache: UsersCache::new(),
        }
    }

    pub fn resolve_name(&self, uid: u32) -> Option<String> {
        self.cache
            .get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().to_string())
    }
}
