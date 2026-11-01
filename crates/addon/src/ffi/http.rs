//! Minimal HTTP GET over WinHTTP (no third-party client, no TLS inside the DLL).
//!
//! The addon has to talk to `api.guildwars2.com` from inside the game. Pulling a full HTTP
//! stack with bundled TLS would add around 2 MB to a 1.2 MB DLL, so we use the WinHTTP
//! service Windows already ships: it handles TLS, proxies and redirects, and costs nothing
//! in size. Everything unsafe lives here, with a `SAFETY` note per call.

use core::ffi::c_void;
use std::mem::size_of;
use std::ptr;

use gw2lurker_api::ApiError;

type Handle = *mut c_void;

/// WinHTTP wants UTF-16 with a terminating NUL.
fn wide(text: &str) -> Vec<u16> {
    let mut buffer: Vec<u16> = text.encode_utf16().collect();
    buffer.push(0);
    buffer
}

#[link(name = "winhttp")]
extern "system" {
    fn WinHttpOpen(
        agent: *const u16,
        access_type: u32,
        proxy: *const u16,
        bypass: *const u16,
        flags: u32,
    ) -> Handle;
    fn WinHttpConnect(session: Handle, server: *const u16, port: u16, reserved: u32) -> Handle;
    fn WinHttpOpenRequest(
        connect: Handle,
        verb: *const u16,
        object: *const u16,
        version: *const u16,
        referrer: *const u16,
        accept_types: *const *const u16,
        flags: u32,
    ) -> Handle;
    fn WinHttpSendRequest(
        request: Handle,
        headers: *const u16,
        headers_len: u32,
        optional: *mut c_void,
        optional_len: u32,
        total_len: u32,
        context: usize,
    ) -> i32;
    fn WinHttpReceiveResponse(request: Handle, reserved: *mut c_void) -> i32;
    fn WinHttpQueryHeaders(
        request: Handle,
        info_level: u32,
        name: *const u16,
        buffer: *mut c_void,
        buffer_len: *mut u32,
        index: *mut u32,
    ) -> i32;
    fn WinHttpReadData(request: Handle, buffer: *mut c_void, to_read: u32, read: *mut u32) -> i32;
    fn WinHttpSetTimeouts(
        session: Handle,
        resolve: i32,
        connect: i32,
        send: i32,
        receive: i32,
    ) -> i32;
    fn WinHttpCloseHandle(handle: Handle) -> i32;
}

const ACCESS_TYPE_DEFAULT_PROXY: u32 = 0;
const FLAG_SECURE: u32 = 0x0080_0000;
const QUERY_STATUS_CODE: u32 = 19;
const QUERY_FLAG_NUMBER: u32 = 0x2000_0000;
const MAX_BODY: usize = 8 * 1024 * 1024;

/// RAII wrapper so no handle leaks on an early return.
struct Owned(Handle);

impl Owned {
    fn new(handle: Handle, what: &str) -> Result<Self, ApiError> {
        if handle.is_null() {
            Err(ApiError::Transport(format!(
                "WinHTTP: no se pudo crear {what}"
            )))
        } else {
            Ok(Self(handle))
        }
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: the handle comes from a successful `WinHttp*` creation call, is closed
        // exactly once and is not used afterwards.
        unsafe { WinHttpCloseHandle(self.0) };
    }
}

/// Splits a URL into `(secure, host, port, path)`.
fn split_url(url: &str) -> Result<(bool, &str, u16, &str), ApiError> {
    let (secure, rest) = if let Some(rest) = url.strip_prefix("https://") {
        (true, rest)
    } else if let Some(rest) = url.strip_prefix("http://") {
        (false, rest)
    } else {
        return Err(ApiError::Transport(format!("URL no soportada: {url}")));
    };

    let (authority, path) = match rest.find('/') {
        Some(index) => (&rest[..index], &rest[index..]),
        None => (rest, "/"),
    };
    if authority.is_empty() {
        return Err(ApiError::Transport(format!("URL sin host: {url}")));
    }

    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            let port = port
                .parse::<u16>()
                .map_err(|_| ApiError::Transport(format!("puerto inválido en {url}")))?;
            (host, port)
        }
        None => (authority, if secure { 443 } else { 80 }),
    };
    if host.is_empty() {
        return Err(ApiError::Transport(format!("URL sin host: {url}")));
    }
    Ok((secure, host, port, path))
}

/// Performs the GET and returns `(status, body)`.
pub fn get(url: &str, timeout_ms: i32) -> Result<(u16, Vec<u8>), ApiError> {
    let (secure, host, port, path) = split_url(url)?;
    let agent = wide("gw2-lurker (Raidcore Nexus addon)");
    let host_w = wide(host);
    let path_w = wide(path);
    let verb = wide("GET");

    // SAFETY: `agent` is a NUL-terminated UTF-16 buffer that outlives the call; the remaining
    // arguments are constants or null, which WinHTTP documents as "use the default".
    let session = Owned::new(
        unsafe {
            WinHttpOpen(
                agent.as_ptr(),
                ACCESS_TYPE_DEFAULT_PROXY,
                ptr::null(),
                ptr::null(),
                0,
            )
        },
        "la sesión",
    )?;

    // SAFETY: `session` is valid; timeouts are plain integers. A failure only means fewer
    // timeouts configured, so it is not fatal.
    unsafe {
        WinHttpSetTimeouts(session.0, timeout_ms, timeout_ms, timeout_ms, timeout_ms);
    }

    // SAFETY: `session` is valid and `host_w` is a live NUL-terminated buffer. A null return
    // is reported instead of dereferenced.
    let connection = Owned::new(
        unsafe { WinHttpConnect(session.0, host_w.as_ptr(), port, 0) },
        "la conexión",
    )?;

    // SAFETY: both handles are valid and the two strings live until the end of the function;
    // `FLAG_SECURE` is required for https. Accept types and referrer stay null (documented).
    let request = Owned::new(
        unsafe {
            WinHttpOpenRequest(
                connection.0,
                verb.as_ptr(),
                path_w.as_ptr(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                if secure { FLAG_SECURE } else { 0 },
            )
        },
        "la petición",
    )?;

    // SAFETY: valid request handle; no body and no extra headers, both documented as null/0.
    let sent = unsafe { WinHttpSendRequest(request.0, ptr::null(), 0, ptr::null_mut(), 0, 0, 0) };
    if sent == 0 {
        return Err(ApiError::Transport(error_message("WinHttpSendRequest")));
    }

    // SAFETY: valid request handle, no reserved value.
    let received = unsafe { WinHttpReceiveResponse(request.0, ptr::null_mut()) };
    if received == 0 {
        return Err(ApiError::Transport(error_message("WinHttpReceiveResponse")));
    }

    let status = query_status(request.0)?;

    let mut body = Vec::new();
    let mut buffer = [0u8; 16 * 1024];
    loop {
        let mut read: u32 = 0;
        // SAFETY: the request is valid and `buffer` is a live, correctly sized array; `read`
        // receives the number of bytes written into it.
        let ok = unsafe {
            WinHttpReadData(
                request.0,
                buffer.as_mut_ptr().cast(),
                buffer.len() as u32,
                &mut read,
            )
        };
        if ok == 0 {
            return Err(ApiError::Transport(error_message("WinHttpReadData")));
        }
        if read == 0 {
            break;
        }
        body.extend_from_slice(&buffer[..read as usize]);
        if body.len() > MAX_BODY {
            return Err(ApiError::Transport(format!(
                "respuesta mayor de {MAX_BODY} bytes"
            )));
        }
    }

    Ok((status, body))
}

fn query_status(request: Handle) -> Result<u16, ApiError> {
    let mut status: u32 = 0;
    let mut len = size_of::<u32>() as u32;
    // SAFETY: valid request handle; the buffer is a live `u32` and `len` its size as WinHTTP
    // expects. `index` stays null because there is a single status code.
    let ok = unsafe {
        WinHttpQueryHeaders(
            request,
            QUERY_STATUS_CODE | QUERY_FLAG_NUMBER,
            ptr::null(),
            (&mut status as *mut u32).cast(),
            &mut len,
            ptr::null_mut(),
        )
    };
    if ok == 0 {
        return Err(ApiError::Transport(error_message("WinHttpQueryHeaders")));
    }
    Ok(status as u16)
}

fn error_message(call: &str) -> String {
    format!("{call} falló: {}", std::io::Error::last_os_error())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_are_split_without_guessing() {
        assert_eq!(
            split_url("https://api.guildwars2.com/v2/wvw/matches/2-2").unwrap(),
            (true, "api.guildwars2.com", 443, "/v2/wvw/matches/2-2")
        );
        assert_eq!(
            split_url("http://example.com:8080/x?y=1").unwrap(),
            (false, "example.com", 8080, "/x?y=1")
        );
        assert_eq!(
            split_url("https://example.com").unwrap(),
            (true, "example.com", 443, "/")
        );
        assert!(split_url("ftp://example.com").is_err());
        assert!(split_url("https:///sinhost").is_err());
    }

    /// Real request against the public API. Ignored by default: CI must not depend on the
    /// network. Run with `cargo test -p gw2lurker-addon -- --ignored`.
    #[test]
    #[ignore = "requiere red"]
    fn live_get_returns_the_match_json() {
        let (status, body) = get("https://api.guildwars2.com/v2/wvw/matches/2-2", 5_000).unwrap();
        assert_eq!(status, 200);
        assert!(body.starts_with(b"{"), "cuerpo inesperado");
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["id"], "2-2");
    }

    /// El ZIP del índice se descarga con **este** cliente, no con otro: si aquí da 404, el addon
    /// también. Sirve para separar «no está publicado» de «el addon no sabe bajarlo».
    #[test]
    #[ignore = "requiere red"]
    fn live_the_index_zip_is_downloadable() {
        let url = crate::guild_index::DEFAULT_URL;
        for query in [
            "",
            "?cb=1",
            "?v=1",
            "?v=1761950000",
            "?a=2",
            "?x=abcdef",
            "?cb=999999",
        ] {
            let probe = format!("{url}{query}");
            match get(&probe, 10_000) {
                Ok((status, body)) => println!("PROBE {status} {:>8} bytes <- {query}", body.len()),
                Err(error) => println!("PROBE error {error:?} <- {query}"),
            }
        }
        let stamp = chrono::Utc::now().timestamp();
        let (status, body) = get(&crate::guild_index::cache_busted(url, stamp), 10_000)
            .unwrap_or_else(|error| panic!("PROBE {url}: {error:?}"));
        assert_eq!(status, 200, "el ZIP publicado tiene que responder 200");
        assert!(body.len() > 1000, "bytes: {}", body.len());
    }
}
