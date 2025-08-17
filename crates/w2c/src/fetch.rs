use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures::{Stream, TryStreamExt};
use gloo::net::{
    Error as GlooNetError,
    http::{
        Headers as GlooHttpHeaders, Method, Request as GlooHttpRequest,
        RequestBuilder as GlooHttpRequestBuilder, Response as GlooHttpResponse,
    },
};
use http::{
    Error as HttpError, HeaderName, Request as HttpRequest, Response as HttpResponse,
    header::{
        InvalidHeaderName as HttpInvalidHeaderName, InvalidHeaderValue as HttpInvalidHeaderValue,
        ToStrError as HttpHeaderToStrError,
    },
};
use http_body::{Body as HttpBody, Frame as HttpBodyFrame};
use http_body_util::BodyExt;
use nill::{Nil, nil};
use tower::Service;
use wasm_bindgen::JsValue;
use wasm_streams::ReadableStream as WasmReadableStream;
use web_sys::{ReadableStream as HttpReadableStream, RequestMode, js_sys::Uint8Array};
use xscan_h2c::{
    tonic::{Status, body::Body as GrpcBody},
    web::GrpcWebCall,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    HttpError(#[from] HttpError),

    #[error(transparent)]
    HttpHeaderToStr(#[from] HttpHeaderToStrError),

    #[error(transparent)]
    HttpInvalidHeaderName(#[from] HttpInvalidHeaderName),

    #[error(transparent)]
    HttpInvalidHeaderValue(#[from] HttpInvalidHeaderValue),

    #[error(transparent)]
    TonicStatus(#[from] Status),

    #[error(transparent)]
    GlooNet(#[from] GlooNetError),

    #[error("{0}")]
    Generic(String),
}

macro_rules! err {
    ($($arg:tt)*) => {
        Err(Error::Generic(format!($($arg)*)))
    }
}

impl From<JsValue> for Error {
    fn from(err: JsValue) -> Self {
        Error::Generic(format!("{err:?}"))
    }
}

trait HttpRequestExt {
    async fn try_into_gloo(self) -> Result<GlooHttpRequest, Error>;
}

impl HttpRequestExt for HttpRequest<GrpcWebCall<GrpcBody>> {
    async fn try_into_gloo(self) -> Result<GlooHttpRequest, Error> {
        let uri = self.uri().to_string();
        let headers = GlooHttpHeaders::new();
        for (key, val) in self.headers() {
            headers.set(key.as_str(), val.to_str()?);
        }
        let bytes = self.into_body().collect().await?.to_bytes();
        let fetch = GlooHttpRequestBuilder::new(&uri)
            .mode(RequestMode::Cors)
            .headers(headers)
            .method(Method::POST)
            .body(Uint8Array::from(&*bytes))?;
        Ok(fetch)
    }
}

trait HttpResponseExt {
    async fn try_into_grpc(self) -> Result<HttpResponse<GrpcBody>, Error>;
}

impl HttpResponseExt for GlooHttpResponse {
    async fn try_into_grpc(self) -> Result<HttpResponse<GrpcBody>, Error> {
        match self.body() {
            Some(stream) => {
                let data = GrpcBody::new(GrpcWebCallStream::new(stream));
                let mut grpc = HttpResponse::builder().status(self.status()).body(data)?;
                let headers = grpc.headers_mut();
                for (key, val) in self.headers().entries() {
                    headers.insert(HeaderName::try_from(key)?, val.parse()?);
                }
                Ok(grpc)
            },
            None => err!("HTTP content return None: {self:?}"),
        }
    }
}

pub struct GrpcWebCallStream {
    inner: Pin<Box<dyn Stream<Item = Result<HttpBodyFrame<Bytes>, Error>>>>,
}

impl GrpcWebCallStream {
    pub fn new(http_stream: HttpReadableStream) -> Self {
        let stream = WasmReadableStream::from_raw(http_stream)
            .into_stream()
            .map_ok(|data| {
                // TODO: stream
                let bytes = Bytes::from(Uint8Array::new(&data).to_vec());
                HttpBodyFrame::data(bytes)
            })
            .map_err(Error::from);
        Self { inner: Box::pin(stream) }
    }
}

unsafe impl Send for GrpcWebCallStream {}

impl HttpBody for GrpcWebCallStream {
    type Data = Bytes;
    type Error = Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<HttpBodyFrame<Self::Data>, Self::Error>>> {
        // TODO: void dyn
        self.inner.as_mut().poll_next(cx)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Fetch {}

impl Fetch {
    pub const fn new() -> Self {
        Fetch {}
    }

    async fn grpc_web_call(
        self,
        grpc: HttpRequest<GrpcWebCall<GrpcBody>>,
    ) -> Result<HttpResponse<GrpcBody>, Error> {
        let fetch = grpc.try_into_gloo().await?;
        fetch.send().await?.try_into_grpc().await
    }
}

impl Service<HttpRequest<GrpcWebCall<GrpcBody>>> for Fetch {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Response = HttpResponse<GrpcBody>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<Nil, Self::Error>> {
        Poll::Ready(Ok(nil))
    }

    fn call(&mut self, grpc: HttpRequest<GrpcWebCall<GrpcBody>>) -> Self::Future {
        // TODO: void clone
        Box::pin(self.clone().grpc_web_call(grpc))
    }
}
