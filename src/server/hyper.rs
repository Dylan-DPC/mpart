//! Server-side integration with [Hyper](https://github.com/hyperium/hyper).
//! Enabled with the `hyper` feature (on by default).
//!
//! Also contains an implementation of [`HttpRequest`](../trait.HttpRequest.html)
//! for `hyper::server::Request` and `&mut hyper::server::Request`.
use hyper::header::ContentType;
use hyper::method::Method;
use hyper::net::Fresh;
use hyper::server::{Handler, Request, Response};

#[allow(clippy::module_name_repetitions)]
pub use hyper::server::Request as HyperRequest;

use hyper::mime::{Attr, Mime, SubLevel, TopLevel, Value};

use super::{HttpRequest, Multipart};

/// A container that implements `hyper::server::Handler` which will switch
/// the handler implementation depending on if the incoming request is multipart or not.
///
/// Create an instance with `new()` and pass it to `hyper::server::Server::listen()` where
/// you would normally pass a `Handler` instance.
///
/// A convenient wrapper for `Multipart::from_request()`.
pub struct Switch<H, M> {
    normal: H,
    multipart: M,
}

impl<H, M> Switch<H, M>
where
    H: Handler,
    M: MultipartHandler,
{
    /// Create a new `Switch` instance where
    /// `normal` handles normal Hyper requests and `multipart` handles Multipart requests
    pub fn new(normal: H, multipart: M) -> Switch<H, M> {
        Switch { normal, multipart }
    }
}

impl<H, M> Handler for Switch<H, M>
where
    H: Handler,
    M: MultipartHandler,
{
    #[allow(clippy::similar_names)]
    fn handle<'a>(&'a self, req: Request<'a, '_>, res: Response<'a, Fresh>) {
        match Multipart::from_request(req) {
            Ok(multi) => self.multipart.handle_multipart(multi, res),
            Err(req) => self.normal.handle(req, res),
        }
    }
}

/// A trait defining a type that can handle an incoming multipart request.
///
/// Extends to closures of the type `Fn(Multipart<Request>, Response<Fresh>)`,
/// and subsequently static functions.
pub trait MultipartHandler: Send + Sync {
    /// Generate a response from this multipart request.
    fn handle_multipart<'a>(
        &self,
        multipart: Multipart<Request<'a, '_>>,
        response: Response<'a, Fresh>,
    );
}

impl<F> MultipartHandler for F
where
    F: Fn(Multipart<Request<'_, '_>>, Response<'_, Fresh>),
    F: Send + Sync,
{
    fn handle_multipart<'a>(
        &self,
        multipart: Multipart<Request<'a, '_>>,
        response: Response<'a, Fresh>,
    ) {
        (*self)(multipart, response);
    }
}

impl<'a, 'b> HttpRequest for HyperRequest<'a, 'b> {
    type Body = Self;

    fn multipart_boundary(&self) -> Option<&str> {
        if self.method != Method::Post {
            return None;
        }

        self.headers.get::<ContentType>().and_then(|ct| {
            let ContentType(ref mime) = *ct;
            let Mime(TopLevel::Multipart, SubLevel::FormData, ref params) = *mime else {
                return None;
            };

            params
                .iter()
                .find(|&(name, _)| matches!(*name, Attr::Boundary))
                .and_then(|(_, val)| match *val {
                    Value::Ext(ref val) => Some(&**val),
                    Value::Utf8 => None,
                })
        })
    }

    fn body(self) -> Self {
        self
    }
}

impl<'r, 'a, 'b> HttpRequest for &'r mut HyperRequest<'a, 'b> {
    type Body = Self;

    fn multipart_boundary(&self) -> Option<&str> {
        if self.method != Method::Post {
            return None;
        }

        self.headers.get::<ContentType>().and_then(|ct| {
            let ContentType(ref mime) = *ct;
            let Mime(TopLevel::Multipart, SubLevel::FormData, ref params) = *mime else {
                return None;
            };

            params
                .iter()
                .find(|&(name, _)| matches!(*name, Attr::Boundary))
                .and_then(|(_, val)| match *val {
                    Value::Ext(ref val) => Some(&**val),
                    Value::Utf8 => None,
                })
        })
    }

    fn body(self) -> Self::Body {
        self
    }
}
