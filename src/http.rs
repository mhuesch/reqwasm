use crate::{js_to_error, Error};
use serde::de::DeserializeOwned;
use std::fmt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;
pub use web_sys::{
    AbortSignal, FormData, Headers, ObserverCallback, ReadableStream, ReferrerPolicy, RequestCache,
    RequestCredentials, RequestMode, RequestRedirect,
};

/// Valid request methods.
#[derive(Clone, Copy, Debug)]
pub enum Method {
    GET,
    POST,
    PATCH,
    DELETE,
    PUT,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PATCH => "PATCH",
            Method::DELETE => "DELETE",
            Method::PUT => "PUT",
        };
        write!(f, "{}", s)
    }
}

/// A request.
pub struct Request {
    options: web_sys::RequestInit,
    headers: web_sys::Headers,
    url: String,
}

impl Request {
    /// Creates a new request with a url.
    pub fn new(url: &str) -> Self {
        Self {
            options: web_sys::RequestInit::new(),
            headers: web_sys::Headers::new().expect("headers"),
            url: url.into(),
        }
    }

    /// Sets the body.
    pub fn body(mut self, body: impl Into<JsValue>) -> Self {
        self.options.body(Some(&body.into()));
        self
    }

    /// Sets the request cache.
    pub fn cache(mut self, cache: RequestCache) -> Self {
        self.options.cache(cache);
        self
    }

    /// Sets the request credentials.
    pub fn credentials(mut self, credentials: RequestCredentials) -> Self {
        self.options.credentials(credentials);
        self
    }

    /// Sets a header.
    pub fn header(self, key: &str, value: &str) -> Self {
        self.headers.set(key, value).expect("set header");
        self
    }

    /// Sets the request integrity.
    pub fn integrity(mut self, integrity: &str) -> Self {
        self.options.integrity(integrity);
        self
    }

    /// Sets the request method.
    pub fn method(mut self, method: Method) -> Self {
        self.options.method(&method.to_string());
        self
    }

    /// Sets the request mode.
    pub fn mode(mut self, mode: RequestMode) -> Self {
        self.options.mode(mode);
        self
    }

    /// Sets the observer callback.
    pub fn observe(mut self, observe: &ObserverCallback) -> Self {
        self.options.observe(observe);
        self
    }

    /// Sets the request redirect.
    pub fn redirect(mut self, redirect: RequestRedirect) -> Self {
        self.options.redirect(redirect);
        self
    }

    /// Sets the request referrer.
    pub fn referrer(mut self, referrer: &str) -> Self {
        self.options.referrer(referrer);
        self
    }

    /// Sets the request referrer policy.
    pub fn referrer_policy(mut self, referrer_policy: ReferrerPolicy) -> Self {
        self.options.referrer_policy(referrer_policy);
        self
    }

    /// Sets the request abort signal.
    pub fn abort_signal(mut self, signal: Option<&AbortSignal>) -> Self {
        self.options.signal(signal);
        self
    }

    /// Executes the request.
    pub async fn send(mut self) -> Result<Response, Error> {
        self.options.headers(&self.headers);

        let request = web_sys::Request::new_with_str_and_init(&self.url, &self.options)
            .map_err(js_to_error)?;

        let promise = window().unwrap().fetch_with_request(&request);
        let response = JsFuture::from(promise).await.map_err(js_to_error)?;
        match response.dyn_into::<web_sys::Response>() {
            Ok(response) => Ok(Response {
                response: response.unchecked_into(),
            }),
            Err(_) => Err(Error::Other(anyhow::anyhow!("can't convert to Response"))),
        }
    }

    /// Creates a new [`GET`][RequestMethod::GET] `Request` with url.
    pub fn get(url: &str) -> Self {
        Self::new(url).method(Method::GET)
    }

    /// Creates a new [`POST`][RequestMethod::POST] `Request` with url.
    pub fn post(url: &str) -> Self {
        Self::new(url).method(Method::POST)
    }

    /// Creates a new [`PUT`][RequestMethod::PUT] `Request` with url.
    pub fn put(url: &str) -> Self {
        Self::new(url).method(Method::PUT)
    }

    /// Creates a new [`DELETE`][RequestMethod::DELETE] `Request` with url.
    pub fn delete(url: &str) -> Self {
        Self::new(url).method(Method::DELETE)
    }

    /// Creates a new [`PATCH`][RequestMethod::PATCH] `Request` with url.
    pub fn patch(url: &str) -> Self {
        Self::new(url).method(Method::PATCH)
    }
}

/// The [`Request`]'s response
pub struct Response {
    response: web_sys::Response,
}

impl Response {
    /// Gets the url.
    pub fn url(&self) -> String {
        self.response.url()
    }

    /// Whether the request was redirected.
    pub fn redirected(&self) -> bool {
        self.response.redirected()
    }

    /// Gets the status.
    pub fn status(&self) -> u16 {
        self.response.status()
    }

    /// Whether the response was `ok`.
    pub fn ok(&self) -> bool {
        self.response.ok()
    }

    /// Gets the status text.
    pub fn status_text(&self) -> String {
        self.response.status_text()
    }

    /// Gets the headers.
    pub fn headers(&self) -> Headers {
        self.response.headers()
    }

    /// Whether the body was used.
    pub fn body_used(&self) -> bool {
        self.response.body_used()
    }

    /// Gets the body.
    pub fn body(&self) -> Option<ReadableStream> {
        self.response.body()
    }

    /// Gets the raw [`Response`][web_sys::Response] object.
    pub fn as_raw(&self) -> &web_sys::Response {
        &self.response
    }

    /// Gets the form data.
    pub async fn form_data(&self) -> Result<FormData, Error> {
        let promise = self.response.form_data().map_err(js_to_error)?;
        let val = JsFuture::from(promise).await.map_err(js_to_error)?;
        Ok(FormData::from(val))
    }

    /// Gets and parses the json.
    pub async fn json<T: DeserializeOwned>(&self) -> Result<T, Error> {
        let promise = self.response.json().map_err(js_to_error)?;
        let json = JsFuture::from(promise).await.map_err(js_to_error)?;

        Ok(json.into_serde()?)
    }

    /// Gets the response text.
    pub async fn text(&self) -> Result<String, Error> {
        let promise = self.response.text().unwrap();
        let val = JsFuture::from(promise).await.map_err(js_to_error)?;
        let string = js_sys::JsString::from(val);
        Ok(String::from(&string))
    }
}
