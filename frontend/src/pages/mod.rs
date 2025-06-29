pub mod home;
pub mod auth;
pub mod dashboard;
pub mod recordings;
pub mod settings;

pub use home::Home;
pub use auth::{Login, Register};
pub use dashboard::Dashboard;
pub use recordings::Recordings;
pub use settings::Settings;
