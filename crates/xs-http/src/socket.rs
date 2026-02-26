use std::{
    error::Error,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
};
use xs_tls::TlsStream;

struct ReadWrapper<R>
where
    R: Read,
{
    inner: Arc<Mutex<R>>,
}

impl<R: Read> Read for ReadWrapper<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.inner.lock().unwrap().read(buf)
    }
}

struct WriteWrapper<W>
where
    W: Write,
{
    inner: Arc<Mutex<W>>,
}

impl<W: Write> Write for WriteWrapper<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.inner.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.inner.lock().unwrap().flush()
    }
}

fn bind_secure(host: &str, port: u16) -> Result<Socket, Box<dyn Error>> {
    let tls_stream = Arc::new(Mutex::new(TlsStream::new(host, port)?));

    let input_stream = Arc::new(Mutex::new(ReadWrapper {
        inner: tls_stream.clone(),
    }));
    let output_stream = Arc::new(Mutex::new(WriteWrapper {
        inner: tls_stream.clone(),
    }));

    Ok(Socket {
        output_stream,
        input_stream,
    })
}

fn bind_unsecure(tcp_stream: TcpStream) -> Result<Socket, Box<dyn Error>> {
    let stream = Arc::new(Mutex::new(tcp_stream));
    let input_stream = Arc::new(Mutex::new(ReadWrapper {
        inner: stream.clone(),
    }));
    let output_stream = Arc::new(Mutex::new(WriteWrapper { inner: stream }));
    Ok(Socket {
        output_stream,
        input_stream,
    })
}

pub struct Socket {
    pub output_stream: Arc<Mutex<dyn Write + Send>>,
    pub input_stream: Arc<Mutex<dyn Read + Send>>,
}

impl Socket {
    pub fn bind(host: &str, port: u16, secure: bool) -> Result<Socket, Box<dyn Error>> {
        if secure {
            bind_secure(host, port)
        } else {
            let tcp_stream = TcpStream::connect((host, port))?;
            bind_unsecure(tcp_stream)
        }
    }
}
