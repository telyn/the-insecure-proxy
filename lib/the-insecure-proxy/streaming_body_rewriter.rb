# frozen_string_literal: true

module TheInsecureProxy
  class StreamingBodyRewriter
    STATES = [ :nil, :h, :ht, :http, :https, :httpsC, :httpCS ]
    def initialize(streaming_body)
      @state = :nil
      @streaming_body = streaming_body
      @chunk = "".dup
      @tmp = "".dup
    end

    def each(&block)
      chunks = 0
      @streaming_body.each do |chunk|
        chunks += 1
        yield rewrite_chunk(chunk)
      end
      puts "Rewrote #{chunks} chunks"
    end

    def rewrite_chunk(chunk)
      @chunk = "".dup
      chunk.each_char do |char|
        process_char(char)
      end
      @chunk
    end

    def keep_tmp(char, new_state)
      @tmp << char
      @acted = true
      @state = new_state
    end

    def keep_http
      @tmp = "".dup
      keep_body("http://")
    end

    def keep_body(char)
      @acted = true
      @chunk << @tmp
      @chunk << char
      @tmp = "".dup
      @state = :nil
    end

    def process_char(char)
      @acted = false
      return keep_body(char) unless %w[h t p s : /].include?(char)

      case char
      when "h"
        keep_tmp(char, :h) if @state == :nil

      when "t"
        if @state == :h
          keep_tmp(char, :ht)
        elsif @state == :ht
          keep_tmp(char, :htt)
        end

      when "p"
        keep_tmp(char, :http) if @state == :htt
      when "s"
        keep_tmp(char, :https) if @state == :http
      when ":"
        keep_tmp(char, :httpsC) if @state == :https

      when "/"
        if @state == :httpsCS
          keep_http
        elsif @state == :httpsC
          keep_tmp(char, :httpsCS)
        end
      end

      if !@acted
        keep_body(char)
      end
    end
  end
end
