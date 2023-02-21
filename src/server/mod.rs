//! The server-side abstraction for multipart requests. Enabled with the `server` feature.
//!
//! Use this when you are implementing an HTTP server and want to
//! to accept, parse, and serve HTTP `multipart/form-data` requests (file uploads).
//!
//! See the `Multipart` struct for more info.

use std::io;
use std::io::prelude::*;

use crate::server::boundary::BoundaryReader;
use crate::server::field::PrivReadEntry;
use crate::server::field::{MultipartField, ReadEntry, ReadEntryResult};
use crate::server::save::SaveBuilder;

pub use crate::server::save::{Entries, SaveResult, SavedField};

pub mod boundary;
pub mod field;

#[cfg(feature = "hyper")]
pub mod hyper;

#[cfg(feature = "tiny_http")]
pub mod tiny_http;

pub mod save;

/// The server-side implementation of `multipart/form-data` requests.
///
/// Implements `Borrow<R>` to allow access to the request body, if desired.
pub struct Multipart<R> {
    reader: BoundaryReader<R>,
}

impl Multipart<()> {
    /// If the given `HttpRequest` is a multipart/form-data POST request,
    /// return the request body wrapped in the multipart reader. Otherwise,
    /// returns the original request.
    pub fn from_request<R: HttpRequest>(req: R) -> Result<Multipart<R::Body>, R> {
        //FIXME: move `map` expr to `Some` arm when nonlexical borrow scopes land.
        let boundary = match req.multipart_boundary().map(String::from) {
            Some(boundary) => boundary,
            None => return Err(req),
        };

        Ok(Multipart::with_body(req.body(), boundary))
    }
}

impl<R: Read> Multipart<R> {
    /// Construct a new `Multipart` with the given body reader and boundary.
    ///
    /// ## Note: `boundary`
    /// This will prepend the requisite `--` to the boundary string as documented in
    /// [IETF RFC 1341, Section 7.2.1: "Multipart: the common syntax"][rfc1341-7.2.1].
    /// Simply pass the value of the `boundary` key from the `Content-Type` header in the
    /// request (or use `Multipart::from_request()`, if supported).
    ///
    /// [rfc1341-7.2.1]: https://tools.ietf.org/html/rfc1341#page-30
    pub fn with_body<Bnd: Into<String>>(body: R, boundary: Bnd) -> Self {
        let boundary = boundary.into();

        log::info!("Multipart::with_boundary(_, {:?})", boundary);

        Multipart {
            reader: BoundaryReader::from_reader(body, boundary),
        }
    }

    /// Read the next entry from this multipart request, returning a struct with the field's name and
    /// data. See `MultipartField` for more info.
    ///
    /// ## Warning: Risk of Data Loss
    /// If the previously returned entry had contents of type `MultipartField::File`,
    /// calling this again will discard any unread contents of that entry.
    pub fn read_entry(&mut self) -> io::Result<Option<MultipartField<&mut Self>>> {
        self.read_entry_mut().into_result()
    }

    /// Read the next entry from this multipart request, returning a struct with the field's name and
    /// data. See `MultipartField` for more info.
    pub fn into_entry(self) -> ReadEntryResult<Self> {
        self.read_entry()
    }

    /// Call `f` for each entry in the multipart request.
    ///
    /// This is a substitute for Rust not supporting streaming iterators (where the return value
    /// from `next()` borrows the iterator for a bound lifetime).
    ///
    /// Returns `Ok(())` when all fields have been read, or the first error.
    pub fn foreach_entry<F>(&mut self, mut foreach: F) -> io::Result<()>
    where
        F: FnMut(MultipartField<&mut Self>),
    {
        loop {
            match self.read_entry() {
                Ok(Some(field)) => foreach(field),
                Ok(None) => return Ok(()),
                Err(err) => return Err(err),
            }
        }
    }

    /// Get a builder type for saving the files in this request to the filesystem.
    ///
    /// See [`SaveBuilder`](save/struct.SaveBuilder.html) for more information.
    pub fn save(&mut self) -> SaveBuilder<&mut Self> {
        SaveBuilder::new(self)
    }
}

impl<R: Read> PrivReadEntry for Multipart<R> {
    type Source = BoundaryReader<R>;

    fn source_mut(&mut self) -> &mut BoundaryReader<R> {
        &mut self.reader
    }

    fn set_min_buf_size(&mut self, min_buf_size: usize) {
        self.reader.set_min_buf_size(min_buf_size)
    }

    /// Consume the next boundary.
    /// Returns `true` if a field should follow this boundary, `false` otherwise.
    fn consume_boundary(&mut self) -> io::Result<bool> {
        log::debug!("Consume boundary!");
        self.reader.consume_boundary()
    }
}

/// A server-side HTTP request that may or may not be multipart.
///
/// May be implemented by mutable references if providing the request or body by-value is
/// undesirable.
pub trait HttpRequest {
    /// The body of this request.
    type Body: Read;
    /// Get the boundary string of this request if it is a POST request
    /// with the `Content-Type` header set to `multipart/form-data`.
    ///
    /// The boundary string should be supplied as an extra value of the `Content-Type` header, e.g.
    /// `Content-Type: multipart/form-data; boundary={boundary}`.
    fn multipart_boundary(&self) -> Option<&str>;

    /// Return the request body for reading.
    fn body(self) -> Self::Body;
}
