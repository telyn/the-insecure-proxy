use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, PartialEq, Clone, Copy)]
enum State {
    NotInScheme,
    HaveH,
    HaveHT,
    HaveHTT,
    HaveHTTP,
    HaveHTTPS,
    HaveHTTPSC,  // c for colon innit
    HaveHTTPSCS, //s for slash innit
}

// rewrites any https:// into http:// in received chunks - even where it
// spreads across chunk boundaries
pub struct HttpsUrlRewriter {
    output_buffer: BytesMut,
    buffer: BytesMut,
    state: State,
}

pub fn url_rewriter() -> HttpsUrlRewriter {
    HttpsUrlRewriter {
        output_buffer: BytesMut::with_capacity(1024),
        buffer: BytesMut::with_capacity(16),
        state: State::NotInScheme,
    }
}

impl HttpsUrlRewriter {
    pub fn consume_str(&mut self, string: &mut Bytes) {
        while string.has_remaining() {
            self.consume(string.get_u8());
        }
    }

    pub fn move_output(&mut self) -> Bytes {
        let bytes = std::mem::replace(&mut self.output_buffer, BytesMut::with_capacity(1024));
        bytes.freeze()
    }

    pub fn consume(&mut self, chr: u8) {
        match (self.state, chr) {
            (_, b'h') => {
                self.flush();
                self.store(chr, State::HaveH)
            }
            (State::HaveH, b't') => self.store(chr, State::HaveHT),
            (State::HaveHT, b't') => self.store(chr, State::HaveHTT),
            (State::HaveHTT, b'p') => self.store(chr, State::HaveHTTP),
            (State::HaveHTTP, b's') => self.store(chr, State::HaveHTTPS),
            (State::HaveHTTPS, b':') => self.store(chr, State::HaveHTTPSC),
            (State::HaveHTTPSC, b'/') => self.store(chr, State::HaveHTTPSCS),
            (State::HaveHTTPSCS, b'/') => self.output_http(),
            _ => self.flush_and_output(chr),
        }
    }

    fn reset_buffer(&mut self) -> Bytes {
        let buf = std::mem::replace(&mut self.buffer, BytesMut::with_capacity(16));
        self.state = State::NotInScheme;
        buf.freeze()
    }

    fn flush(&mut self) {
        let bytes = self.reset_buffer();
        self.output_buffer.put(bytes);
    }

    fn flush_and_output(&mut self, chr: u8) {
        self.flush();
        self.output_buffer.put_u8(chr);
    }

    // stores chr on the temporary buffer and updates state
    fn store(&mut self, chr: u8, next_state: State) {
        self.buffer.put_u8(chr);
        self.state = next_state;
    }

    // adds http:// to the output buffer and resets buffer & state
    fn output_http(&mut self) {
        self.output_buffer.put(&b"http://"[..]);
        self.reset_buffer();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod consume {
        use super::*;

        #[test]
        fn state_n_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::with_capacity(5),
                state: State::NotInScheme,
            };
            rewriter.consume(b'h');
            assert_eq!(&rewriter.buffer[..], b"h");
            assert_eq!(&rewriter.output_buffer[..], b"");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_h_consume_t() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"h"[..]),
                state: State::HaveH,
            };
            rewriter.consume(b't');
            assert_eq!(&rewriter.buffer[..], b"ht");
            assert_eq!(&rewriter.output_buffer[..], b"");
            assert_eq!(rewriter.state, State::HaveHT);
        }

        #[test]
        fn state_ht_consume_t() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"ht"[..]),
                state: State::HaveHT,
            };
            rewriter.consume(b't');
            assert_eq!(&rewriter.buffer[..], b"htt");
            assert_eq!(&rewriter.output_buffer[..], b"");
            assert_eq!(rewriter.state, State::HaveHTT);
        }

        #[test]
        fn state_htt_consume_p() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"htt"[..]),
                state: State::HaveHTT,
            };
            rewriter.consume(b'p');
            assert_eq!(&rewriter.buffer[..], b"http");
            assert_eq!(&rewriter.output_buffer[..], b"");
            assert_eq!(rewriter.state, State::HaveHTTP);
        }

        #[test]
        fn state_http_consume_s() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"http"[..]),
                state: State::HaveHTTP,
            };
            rewriter.consume(b's');
            assert_eq!(&rewriter.buffer[..], b"https");
            assert_eq!(&rewriter.output_buffer[..], b"");
            assert_eq!(rewriter.state, State::HaveHTTPS);
        }

        #[test]
        fn state_https_consume_colon() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"https"[..]),
                state: State::HaveHTTPS,
            };
            rewriter.consume(b':');
            assert_eq!(&rewriter.buffer[..], b"https:");
            assert_eq!(&rewriter.output_buffer[..], b"");
            assert_eq!(rewriter.state, State::HaveHTTPSC);
        }

        #[test]
        fn state_httpsc_consume_slash() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"https:"[..]),
                state: State::HaveHTTPSC,
            };
            rewriter.consume(b'/');
            assert_eq!(&rewriter.buffer[..], b"https:/");
            assert_eq!(&rewriter.output_buffer[..], b"");
            assert_eq!(rewriter.state, State::HaveHTTPSCS);
        }

        #[test]
        fn state_httpscs_consume_slash() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"https:/"[..]),
                state: State::HaveHTTPSCS,
            };
            rewriter.consume(b'/');
            assert_eq!(&rewriter.buffer[..], b"");
            assert_eq!(&rewriter.output_buffer[..], b"http://");
            assert_eq!(rewriter.state, State::NotInScheme);
        }

        // unhappy paths

        #[test]
        fn state_h_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"h"[..]),
                state: State::NotInScheme,
            };
            rewriter.consume(b'h');
            assert_eq!(&rewriter.buffer[..], b"h");
            assert_eq!(&rewriter.output_buffer[..], b"h");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_ht_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"ht"[..]),
                state: State::HaveHT,
            };
            rewriter.consume(b'h');
            assert_eq!(&rewriter.buffer[..], b"h");
            assert_eq!(&rewriter.output_buffer[..], b"ht");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_htt_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"htt"[..]),
                state: State::HaveHTT,
            };
            rewriter.consume(b'h');
            assert_eq!(&rewriter.buffer[..], b"h");
            assert_eq!(&rewriter.output_buffer[..], b"htt");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_http_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"http"[..]),
                state: State::HaveHTTP,
            };
            rewriter.consume(b'h');
            assert_eq!(&rewriter.buffer[..], b"h");
            assert_eq!(&rewriter.output_buffer[..], b"http");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_https_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"https"[..]),
                state: State::HaveHTTPS,
            };
            rewriter.consume(b'h');
            assert_eq!(&rewriter.buffer[..], b"h");
            assert_eq!(&rewriter.output_buffer[..], b"https");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_httpsc_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"https:"[..]),
                state: State::HaveHTTPSC,
            };
            rewriter.consume(b'h');
            assert_eq!(&rewriter.buffer[..], b"h");
            assert_eq!(&rewriter.output_buffer[..], b"https:");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_httpscs_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(1),
                buffer: BytesMut::from(&b"https:/"[..]),
                state: State::HaveHTTPSCS,
            };
            rewriter.consume(b'h');
            assert_eq!(&rewriter.buffer[..], b"h");
            assert_eq!(&rewriter.output_buffer[..], b"https:/");
            assert_eq!(rewriter.state, State::HaveH);
        }
    }

    #[cfg(test)]
    mod consume_str {
        use super::*;

        #[test]
        fn blank_consume_some_with_https_url() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(200),
                buffer: BytesMut::with_capacity(16),
                state: State::NotInScheme,
            };

            rewriter.consume_str(&mut Bytes::from_static(b"hello https://google.com"));

            assert_eq!(&rewriter.output_buffer[..], b"hello http://google.com");
            assert_eq!(&rewriter.buffer[..], b"");
            assert_eq!(rewriter.state, State::NotInScheme);
        }

        #[test]
        fn from_halfway_through_consume_remaining_https() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(200),
                buffer: BytesMut::from(&b"ht"[..]),
                state: State::HaveHT,
            };

            rewriter.consume_str(&mut Bytes::from_static(b"tps://google.com hello"));

            assert_eq!(&rewriter.output_buffer[..], b"http://google.com hello");
            assert_eq!(&rewriter.buffer[..], b"");
            assert_eq!(rewriter.state, State::NotInScheme);
        }

        #[test]
        fn consume_a_few_chunks() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::with_capacity(200),
                buffer: BytesMut::with_capacity(16),
                state: State::NotInScheme,
            };

            rewriter.consume_str(&mut Bytes::from_static(b"hello https://google.com"));
            rewriter.consume_str(&mut Bytes::from_static(b"/goog http://website https:"));
            rewriter.consume_str(&mut Bytes::from_static(b"//example.com"));

            assert_eq!(
                &rewriter.output_buffer[..],
                b"hello http://google.com/goog http://website http://example.com"
            );
        }
    }

    #[cfg(test)]
    mod move_output {
        use super::*;

        #[test]
        fn returns_output() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: BytesMut::from(&b"hello"[..]),
                buffer: BytesMut::with_capacity(1),
                state: State::NotInScheme,
            };

            let result = rewriter.move_output();

            assert_eq!(&result[..], b"hello");
            assert_eq!(&rewriter.output_buffer[..], b"");
        }
    }
}
