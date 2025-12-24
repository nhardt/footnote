use anyhow::Result;
use iroh::endpoint::{RecvStream, SendStream};
use n0_error::StdResultExt;
use std::path::{Path, PathBuf};

pub async fn send_u32(stream: &mut SendStream, value: u32) -> Result<()> {
    SendStream::write_all(stream, &value.to_be_bytes())
        .await
        .anyerr()?;
    Ok(())
}

pub async fn receive_u32(stream: &mut RecvStream) -> Result<u32> {
    let mut buf = [0u8; 4];
    RecvStream::read_exact(stream, &mut buf).await.anyerr()?;
    Ok(u32::from_be_bytes(buf))
}

pub async fn send_u64(stream: &mut SendStream, value: u64) -> Result<()> {
    SendStream::write_all(stream, &value.to_be_bytes())
        .await
        .anyerr()?;
    Ok(())
}

pub async fn receive_u64(stream: &mut RecvStream) -> Result<u64> {
    let mut buf = [0u8; 8];
    RecvStream::read_exact(stream, &mut buf).await.anyerr()?;
    Ok(u64::from_be_bytes(buf))
}

pub async fn send_bytes(stream: &mut SendStream, data: &[u8]) -> Result<()> {
    let len = data.len() as u32;
    send_u32(stream, len).await?;
    SendStream::write_all(stream, data).await.anyerr()?;
    Ok(())
}

pub async fn receive_bytes(stream: &mut RecvStream) -> Result<Vec<u8>> {
    let len = receive_u32(stream).await? as usize;
    let mut buf = vec![0u8; len];
    RecvStream::read_exact(stream, &mut buf).await.anyerr()?;
    Ok(buf)
}

pub async fn send_file_request(stream: &mut SendStream, path: &Path) -> Result<()> {
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
    send_bytes(stream, path_str.as_bytes()).await
}

pub async fn receive_file_request(stream: &mut RecvStream) -> Result<Option<PathBuf>> {
    let len = receive_u32(stream).await?;

    if len == 0 {
        return Ok(None);
    }

    let mut buf = vec![0u8; len as usize];
    RecvStream::read_exact(stream, &mut buf).await.anyerr()?;
    let path_str = String::from_utf8(buf)?;
    Ok(Some(PathBuf::from(path_str)))
}

pub async fn send_file_contents(stream: &mut SendStream, contents: &[u8]) -> Result<()> {
    send_u64(stream, contents.len() as u64).await?;
    SendStream::write_all(stream, contents).await.anyerr()?;
    Ok(())
}

pub async fn receive_file_contents(stream: &mut RecvStream) -> Result<Vec<u8>> {
    let len = receive_u64(stream).await? as usize;
    let mut buf = vec![0u8; len];
    RecvStream::read_exact(stream, &mut buf).await.anyerr()?;
    Ok(buf)
}

pub async fn send_eof(stream: &mut SendStream) -> Result<()> {
    send_u32(stream, 0).await
}
