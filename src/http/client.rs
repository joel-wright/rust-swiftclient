use hyper::client;

/*
 * Abstract the underlying HTTP library
 */

struct HyperHTTPClient {
    client
}

enum HTTPRequest {
    GetRequest ...
    PutRequest ...
    PostRequest ...
}

struct GetRequest {
    url
    params
    headers
}

impl Handler for GetRequest

struct PutRequest {
    url
    params
    headers
    source
}

pub fn add_query_param<K: Display, V: Display>(
    name: &K, value: &V, query_params: &mut Vec<String>
) -> () {
    query_params.push(format!("{}={}", name, value));
}

pub fn add_optional_query_param<K: Display, V: Display>(
    name: &K, value: &Option<V>, query_params: &mut Vec<String>
) -> () {
    match value {
        &Some(ref v) => query_params.push(
            format!("{}={}", name, v)),
        &None => ()
    }
}

pub fn run_request(client: Client, request: HTTPRequest) ->
        Result<response::Response, SwiftError> {
    // make request, wait for result
}

/*
 * Hyper Handler for Post requests
 */

#[derive(Debug)]
struct PostRequest(
    url: &[u8],
    params: Vec<String>,
    headers: Vec<Header>,
    payload: &[u8],
    sender: mpsc::Sender<()>,
    bytes_sent: u64
);

impl Drop for PostRequest {
    fn drop(&mut self) {
        let _ = self.sender.send(());
    }
};

impl hyper::client::Handler<HttpStream> for PostRequest {
    fn on_request(&mut self, req: &mut Request) -> Next {
        req.set_method(Method::Post);
        for head in self.headers {
            req.headers_mut().set(head)
        };
        Next::write()
    }

    fn on_request_writable(
        &mut self, _encoder: &mut Encoder<HttpStream>
    ) -> Next {
        if (len(self.payload) > self.bytes_sent) {
            match transport.try_write(
                &mut self.payload[self.bytes_sent..]
            ) {
                Ok(Some(0)) => panic!("write ZERO"),
                Ok(Some(n)) => {
                    self.bytes_sent += n;
                    if (self.bytes_sent >= len(self.payload)) {
                        Next::read()
                    } else {
                        Next::write()
                    }
                }
                Ok(None) => Next::write(),
                Err(e) => {
                    println!("write error {:?}", e);
                    Next::end()
                }
            }
        } else {
            Next::read().timeout(Duration::from_secs(10))
        }
    }

    fn on_response(&mut self, res: Response) -> Next {
        Next::read().timeout(Duration::from_secs(10))
    }

    fn on_response_readable(
        &mut self, decoder: &mut Decoder<HttpStream>
    ) -> Next {
        decoder.try_read...
        match io::copy(decoder, &mut io::stdout()) {
            Ok(0) => Next::end(),
            Ok(_) => read(),
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => Next::read(),
                _ => {
                    println!("ERROR:example: {}", e);
                    Next::end()
                }
            }
        }
    }

    fn on_error(&mut self, err: hyper::Error) -> Next {
        println!("ERROR:example: {}", err);
        Next::remove()
    }
}
