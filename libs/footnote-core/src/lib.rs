pub mod model;
pub mod platform;
pub mod service;
pub mod util;

// pub use crate::model::vault::Vault;
// pub use crate::model::user::User;
// pub use crate::model::note::Note;
// pub use crate::service::sync_service::SyncService;
// pub use crate::service::join_service::JoinService;

pub type Result<T> = anyhow::Result<T>;
