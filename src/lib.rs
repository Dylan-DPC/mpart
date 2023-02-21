//! Client- and server-side abstractions for HTTP `multipart/form-data` requests.
//!
//! ### Features:
//! This documentation is built with all features enabled.
//!
//! * `client`: The client-side abstractions for generating multipart requests.
//!
//! * `server`: The server-side abstractions for parsing multipart requests.
//!
//! * `mock`: Provides mock implementations of core `client` and `server` traits for debugging
//! or non-standard use.
//!
//! * `hyper`: Integration with the [Hyper](https://crates.io/crates/hyper) HTTP library
//! for client and/or server depending on which other feature flags are set.
//!
//! * `iron`: Integration with the [Iron](http://crates.io/crates/iron) web application
//! framework. See the [`server::iron`](server/iron/index.html) module for more information.
//!
//! * `nickel` (returning in 0.14!): Integration with the [Nickel](https://crates.io/crates/nickel)
//! web application framework. See the [`server::nickel`](server/nickel/index.html) module for more
//! information.
//!
//! * `tiny_http`: Integration with the [`tiny_http`](https://crates.io/crates/tiny_http)
//! crate. See the [`server::tiny_http`](server/tiny_http/index.html) module for more information.
//!
//! ### Note: Work in Progress
//! I have left a number of Request-for-Comments (RFC) questions on various APIs and other places
//! in the code as there are some cases where I'm not sure what the desirable behavior is.
//!
//! I have opened an issue as a place to collect responses and discussions for these questions
//! [on Github](https://github.com/abonander/multipart/issues/96). Please quote the RFC-statement
//! (and/or link to its source line) and provide your feedback there.
#![deny(rust_2018_idioms)]

use rand::Rng;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

fn random_alphanumeric(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(len)
        .map(|c| c as char)
        .collect()
}

#[cfg(test)]
fn init_log() {
    let _ = env_logger::try_init();
}
