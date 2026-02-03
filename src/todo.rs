use embedded_nal_async::{Dns, TcpConnect};
use reqwless::{
    client::HttpClient,
    request::{Method, RequestBuilder},
};

use crate::errors::Error;

pub async fn get_todos<'a, T, D>(
    client: &mut HttpClient<'_, T, D>,
    url: &str,
    auth: &str,
    read_buffer: &'a mut [u8],
) -> Result<impl Iterator<Item = &'a str>, Error>
where
    T: TcpConnect,
    D: Dns,
{
    log::debug!("getting todos from {url}");

    let headers = [("Authorization", auth)];

    let mut write_buffer = [0u8; 1024];

    let size = client
        .request(Method::GET, url)
        .await
        .map_err(|_| Error::Http("failed to connect to todo URL"))?
        .headers(&headers)
        .send(&mut write_buffer)
        .await
        .map_err(|_| Error::Http("failed to send request"))?
        .body()
        .reader()
        .read_to_end(read_buffer)
        .await
        .map_err(|_| Error::Http("failed to read into buffer"))?;

    Ok(core::str::from_utf8(&read_buffer[..size])
        .map_err(|_| Error::ParseUtf8)?
        .split('\n'))
}
