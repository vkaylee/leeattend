pub mod audit;
pub mod crud;
pub mod data_management;
pub mod departments;
pub mod locations;
pub mod members;
pub mod permissions;
pub mod roles;

// Re-export all public handler functions
pub use audit::*;
pub use crud::*;
pub use data_management::*;
pub use departments::*;
pub use locations::*;
pub use members::*;
pub use permissions::*;
pub use roles::*;
