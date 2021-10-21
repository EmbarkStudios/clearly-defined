use crate::Error;
use reqwest::Client as AClient;

#[derive(Default)]
pub struct Client {
    inner: AClient,
}

impl From<AClient> for Client {
    fn from(o: AClient) -> Self {
        Self { inner: o }
    }
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn execute<Res>(&self, req: http::Request<bytes::Bytes>) -> Result<Res, Error>
    where
        Res: crate::ApiResponse<bytes::Bytes>,
    {
        let request = convert_request(req, &self.inner).await?;
        let response = self.inner.execute(request).await?;
        let response = convert_response(response).await?;

        Ok(Res::try_from_parts(response)?)
    }
}

/// Converts a vanilla [`http::Request`] into a [`reqwest::Request`]
async fn convert_request(
    req: http::Request<bytes::Bytes>,
    client: &AClient,
) -> Result<reqwest::Request, Error> {
    let (parts, body) = req.into_parts();

    let uri = parts.uri.to_string();

    let builder = match parts.method {
        http::Method::GET => client.get(&uri),
        http::Method::POST => client.post(&uri),
        http::Method::DELETE => client.delete(&uri),
        http::Method::PUT => client.put(&uri),
        method => unreachable!("{} not implemented", method),
    };

    Ok(builder.headers(parts.headers).body(body).build()?)
}

/// Converts a [`reqwest::Response`] into a vanilla [`http::Response`]. This
/// currently copies the entire response body into a single buffer with no streaming
async fn convert_response(res: reqwest::Response) -> Result<http::Response<bytes::Bytes>, Error> {
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

    let body = res.bytes().await?;

    Ok(builder.body(body)?)
}
