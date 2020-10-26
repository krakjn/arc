// import the modules into the folder module
mod all;
mod checkout;
mod init;
mod pull;
mod reset;
mod status;
mod sync;

// publicy declare the functions
pub use all::all;
pub use checkout::checkout;
pub use init::init;
pub use pull::pull;
pub use reset::reset;
pub use status::status;
pub use sync::sync;
