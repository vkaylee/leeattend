pub mod crud;
pub mod data_management;
pub mod departments;
pub mod members;
pub mod permissions;
pub mod roles;
pub mod audit;

// Re-export all public handler functions
pub use crud::*;
pub use data_management::*;
pub use departments::*;
pub use members::*;
pub use permissions::*;
pub use roles::*;
pub use audit::*;
