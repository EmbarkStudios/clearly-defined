use crate::Error;
use reqwest::blocking::Client as BClient;

#[derive(Default)]
pub struct Client {
    inner: BClient,
}

impl From<BClient> for Client {
    fn from(o: BClient) -> Self {
        Self { inner: o }
    }
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute<Res>(&self, req: http::Request<bytes::Bytes>) -> Result<Res, Error>
    where
        Res: crate::ApiResponse<bytes::Bytes>,
    {
        let request = convert_request(req, &self.inner)?;
        let response = self.inner.execute(request)?;
        let response = convert_response(response)?;

        Res::try_from_parts(response)
    }
}

/// Converts a vanilla http::Request into a reqwest::Request
fn convert_request(
    req: http::Request<bytes::Bytes>,
    client: &BClient,
) -> Result<reqwest::blocking::Request, Error> {
    let (parts, body) = req.into_parts();

    let uri = parts.uri.to_string();

    let builder = match parts.method {
        http::Method::GET => client.get(&uri),
        http::Method::POST => client.post(&uri),
        http::Method::DELETE => client.delete(&uri),
        http::Method::PUT => client.put(&uri),
        method => unimplemented!("{} not implemented", method),
    };

    Ok(builder.headers(parts.headers).body(body.to_vec()).build()?)
}

/// Converts a reqwest::Response into a vanilla http::Response. This currently copies
/// the entire response body into a single buffer with no streaming
fn convert_response(
    mut res: reqwest::blocking::Response,
) -> Result<http::Response<bytes::Bytes>, Error> {
    let mut builder = http::Response::builder()
        .status(res.status())
        .version(res.version());

    use anyhow::Context;
    let headers = builder
        .headers_mut()
        .context("failed to convert response headers")?;

    headers.extend(
        res.headers()
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );

    use bytes::BufMut;
    let body = bytes::BytesMut::with_capacity(res.content_length().unwrap_or(1024) as usize);
    let mut w = body.writer();
    res.copy_to(&mut w)?;

    Ok(builder.body(w.into_inner().freeze())?)
}
