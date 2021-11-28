use std::str;

#[derive(Debug, PartialEq, Clone, Copy)]
enum State {
    NotInScheme,
    HaveH,
    HaveHT,
    HaveHTT,
    HaveHTTP,
    HaveHTTPS,
    HaveHTTPSC, // c for colon innit
    HaveHTTPSCS //s for slash innit
}


// rewrites any https:// into http:// in received chunks - even where it
// spreads across chunk boundaries
pub struct HttpsUrlRewriter {
    output_buffer: String,
    buffer: String,
    state: State
}

pub fn url_rewriter() -> HttpsUrlRewriter {
    return HttpsUrlRewriter {
        output_buffer: String::with_capacity(1024),
        buffer: String::with_capacity(16),
        state: State::NotInScheme
    };
}

impl HttpsUrlRewriter {
    // pub fn read_chunk(&mut self, chunk: hyper::Chunk) {
    //     let chunk_str = String::from_utf8(chunk)
    //         .expect("chunk received was not valid utf-8");
    //     self.consume_string(&chunk_str);
    // }

    pub fn consume_str(&mut self, string: &str) {
        for chr in string.chars() {
            self.consume(chr);
        }
    }

    fn move_output(&mut self) -> String {
        std::mem::replace(&mut self.output_buffer, String::with_capacity(1024))
    }

    pub fn consume(&mut self, chr: char) {
        match (self.state, chr) {
            (_,                  'h') => { self.flush(); self.store(chr, State::HaveH) },
            (State::HaveH,       't') => self.store(chr, State::HaveHT),
            (State::HaveHT,      't') => self.store(chr, State::HaveHTT),
            (State::HaveHTT,     'p') => self.store(chr, State::HaveHTTP),
            (State::HaveHTTP,    's') => self.store(chr, State::HaveHTTPS),
            (State::HaveHTTPS,   ':') => self.store(chr, State::HaveHTTPSC),
            (State::HaveHTTPSC,  '/') => self.store(chr, State::HaveHTTPSCS),
            (State::HaveHTTPSCS, '/') => self.output_http(),
            _ => { self.flush_and_output(chr) }
        }
    }

    fn reset_buffer(&mut self) {
        self.buffer = String::with_capacity(16);
        self.state = State::NotInScheme;
    }

    fn flush(&mut self) {
        self.output_buffer.push_str(&self.buffer);
        self.reset_buffer();
    }

    fn flush_and_output(&mut self, chr: char) {
        self.flush();
        self.output_buffer.push(chr);
    }

    // stores chr on the temporary buffer and updates state
    fn store(&mut self, chr: char, next_state: State) {
        self.buffer.push(chr);
        self.state = next_state;
    }

    // adds http:// to the output buffer and resets buffer & state
    fn output_http(&mut self) {
        self.output_buffer.push_str("http://");
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
                output_buffer: String::with_capacity(1),
                buffer: String::with_capacity(5),
                state: State::NotInScheme
            };
            rewriter.consume('h');
            assert_eq!(rewriter.buffer, "h");
            assert_eq!(rewriter.output_buffer, "");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_h_consume_t() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("h"),
                state: State::HaveH
            };
            rewriter.consume('t');
            assert_eq!(rewriter.buffer, "ht");
            assert_eq!(rewriter.output_buffer, "");
            assert_eq!(rewriter.state, State::HaveHT);
        }

        #[test]
        fn state_ht_consume_t() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("ht"),
                state: State::HaveHT
            };
            rewriter.consume('t');
            assert_eq!(rewriter.buffer, "htt");
            assert_eq!(rewriter.output_buffer, "");
            assert_eq!(rewriter.state, State::HaveHTT);
        }

        #[test]
        fn state_htt_consume_p() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("htt"),
                state: State::HaveHTT
            };
            rewriter.consume('p');
            assert_eq!(rewriter.buffer, "http");
            assert_eq!(rewriter.output_buffer, "");
            assert_eq!(rewriter.state, State::HaveHTTP);
        }

        #[test]
        fn state_http_consume_s() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("http"),
                state: State::HaveHTTP
            };
            rewriter.consume('s');
            assert_eq!(rewriter.buffer, "https");
            assert_eq!(rewriter.output_buffer, "");
            assert_eq!(rewriter.state, State::HaveHTTPS);
        }

        #[test]
        fn state_https_consume_colon() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("https"),
                state: State::HaveHTTPS
            };
            rewriter.consume(':');
            assert_eq!(rewriter.buffer, "https:");
            assert_eq!(rewriter.output_buffer, "");
            assert_eq!(rewriter.state, State::HaveHTTPSC);
        }

        #[test]
        fn state_httpsc_consume_slash() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("https:"),
                state: State::HaveHTTPSC
            };
            rewriter.consume('/');
            assert_eq!(rewriter.buffer, "https:/");
            assert_eq!(rewriter.output_buffer, "");
            assert_eq!(rewriter.state, State::HaveHTTPSCS);
        }

        #[test]
        fn state_httpscs_consume_slash() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("https:/"),
                state: State::HaveHTTPSCS
            };
            rewriter.consume('/');
            assert_eq!(rewriter.buffer, "");
            assert_eq!(rewriter.output_buffer, "http://");
            assert_eq!(rewriter.state, State::NotInScheme);
        }

        // unhappy paths

        #[test]
        fn state_h_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("h"),
                state: State::NotInScheme
            };
            rewriter.consume('h');
            assert_eq!(rewriter.buffer, "h");
            assert_eq!(rewriter.output_buffer, "h");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_ht_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("ht"),
                state: State::HaveHT
            };
            rewriter.consume('h');
            assert_eq!(rewriter.buffer, "h");
            assert_eq!(rewriter.output_buffer, "ht");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_htt_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("htt"),
                state: State::HaveHTT
            };
            rewriter.consume('h');
            assert_eq!(rewriter.buffer, "h");
            assert_eq!(rewriter.output_buffer, "htt");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_http_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("http"),
                state: State::HaveHTTP
            };
            rewriter.consume('h');
            assert_eq!(rewriter.buffer, "h");
            assert_eq!(rewriter.output_buffer, "http");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_https_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("https"),
                state: State::HaveHTTPS
            };
            rewriter.consume('h');
            assert_eq!(rewriter.buffer, "h");
            assert_eq!(rewriter.output_buffer, "https");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_httpsc_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("https:"),
                state: State::HaveHTTPSC
            };
            rewriter.consume('h');
            assert_eq!(rewriter.buffer, "h");
            assert_eq!(rewriter.output_buffer, "https:");
            assert_eq!(rewriter.state, State::HaveH);
        }

        #[test]
        fn state_httpscs_consume_h() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(1),
                buffer: String::from("https:/"),
                state: State::HaveHTTPSCS
            };
            rewriter.consume('h');
            assert_eq!(rewriter.buffer, "h");
            assert_eq!(rewriter.output_buffer, "https:/");
            assert_eq!(rewriter.state, State::HaveH);
        }
    }

    #[cfg(test)]
    mod consume_str {
        use super::*;

        #[test]
        fn blank_consume_some_with_https_url() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(200),
                buffer: String::with_capacity(16),
                state: State::NotInScheme
            };

            rewriter.consume_str("hello https://google.com");

            assert_eq!(rewriter.output_buffer, "hello http://google.com");
            assert_eq!(rewriter.buffer, "");
            assert_eq!(rewriter.state, State::NotInScheme);
        }

        #[test]
        fn from_halfway_through_consume_remaining_https() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(200),
                buffer: String::from("ht"),
                state: State::HaveHT
            };

            rewriter.consume_str("tps://google.com hello");

            assert_eq!(rewriter.output_buffer, "http://google.com hello");
            assert_eq!(rewriter.buffer, "");
            assert_eq!(rewriter.state, State::NotInScheme);
        }

        #[test]
        fn consume_a_few_chunks() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::with_capacity(200),
                buffer: String::with_capacity(16),
                state: State::NotInScheme
            };

            rewriter.consume_str("hello https://google.com");
            rewriter.consume_str("/goog http://website https:");
            rewriter.consume_str("//example.com");

            assert_eq!(rewriter.output_buffer, "hello http://google.com/goog http://website http://example.com");
        }
    }

    #[cfg(test)]
    mod move_output {
        use super::*;

        #[test]
        fn returns_output() {
            let mut rewriter = HttpsUrlRewriter {
                output_buffer: String::from("hello"),
                buffer: String::with_capacity(1),
                state: State::NotInScheme
            };

            let result = rewriter.move_output();

            assert_eq!(result, "hello");
            assert_eq!(rewriter.output_buffer, "");
        }
    }
}
