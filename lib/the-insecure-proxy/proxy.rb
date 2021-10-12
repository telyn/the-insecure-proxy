# frozen_string_literal: true

require "rack-proxy"
require "securerandom"
require "the-insecure-proxy/streaming_body_rewriter"

module TheInsecureProxy
  class Proxy < Rack::Proxy
    def rewrite_env(env)
      @rid = SecureRandom.base64(16)
      env["rack.backend"] = Rack::Request.new(
        env.reject { |k,_| k.start_with?("puma.") }
        .merge("rack.url_scheme" => "https",
               "SERVER_PORT" => 443,
               "HTTPS" => "on")
      )
      log "http://#{env["HTTP_HOST"]}#{env["REQUEST_URI"]}"

      env
    end

    def rewrite_response(response)
      status, headers, body = response
      @content_type = headers["content-type"] || headers["Content-Type"]
      @content_type = @content_type.split(";", 2).first.strip

      [
        status,
        rewrite_headers(headers),
        rewrite_body(body)
      ]
    end

    REWRITTEN_MIME_TYPES = %w[
    text/html
    image/svg
    application/javascript
    application/rss+xml
    application/xml
    application/xhtml+xml
    text/css
    text/javascript
    ].freeze

    def rewrite_body(body)
      if REWRITTEN_MIME_TYPES.include?(@content_type)
        log "Rewriting response"
        StreamingBodyRewriter.new(body)
      else
        log "Streaming response"
        return body
      end
    end

    def rewrite_headers(headers)
      return nil if headers.nil?

      if headers.key?("set-cookie")
        headers["set-cookie"].gsub!(/; ?[sS]ecure/, "")
      end

      if REWRITTEN_MIME_TYPES.include?(@content_type)
        headers["content-length"] = nil
      end

      headers.each { |k, v| headers[k] = v&.gsub("https://", "http://") }
      headers.delete("link")
      headers.delete("strict-transport-security")
      headers
    end

    def log(msg)
      puts "[#{@rid}] #{msg}"
    end
  end
end
