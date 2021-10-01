# frozen_string_literal: true

require "rack-proxy"

class TheInsecureProxy::Proxy < Rack::Proxy
  def rewrite_env(env)
    env["rack.backend"] = Rack::Request.new(
      env.reject { |k,_| k.start_with?("puma.") }
         .merge("rack.url_scheme" => "https",
                "SERVER_PORT" => 443,
                "HTTPS" => "on")
    )

    pp env.reject { |k,v| k.start_with?("puma.") }
    env
  end

  def rewrite_response(response)
    status, headers, body = response

    [
     status,
     rewrite_headers(headers),
     rewrite_body(body, content_type: headers["content-type"] || headers["Content-Type"])
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

  def rewrite_body(body, content_type:)
    content_type = content_type.split(";", 2).first.strip

    return body unless REWRITTEN_MIME_TYPES.include?(content_type)
    puts "REWRITING HTTPS LINKS TO HTTP"

    chunked = body.is_a?(Array)
    body = body.join("") if chunked
    body&.gsub!("https://","http://")
    body = [body] if chunked

    body
  end

  def rewrite_headers(headers)
    return nil if headers.nil?

    if headers.key?("set-cookie")
      headers["set-cookie"].gsub!(/; ?[sS]ecure/, "")
    end

    headers["SERVER_VERSION"] = "HTTP/1.0"

    headers["content-length"] = nil
    headers.each { |k, v| headers[k] = v&.gsub("https://", "http://") }
    headers.delete("link")
    headers.delete("strict-transport-security")
    puts headers.inspect
    headers
  end
end
