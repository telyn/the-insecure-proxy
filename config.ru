#!/usr/bin/env ruby

$LOAD_PATH << File.join(__dir__, "lib")

require "rack"
require "the-insecure-proxy/proxy"

run TheInsecureProxy::Proxy.new(streaming: true)
