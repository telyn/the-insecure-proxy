# frozen_string_literal: true

require_relative "lib/the-insecure-proxy/version"

Gem::Specification.new do |spec|
  spec.name          = "the-insecure-proxy"
  spec.version       = TheInsecureProxy::VERSION
  spec.authors       = ["Telyn Z."]
  spec.email         = ["175827+telyn@users.noreply.github.com"]

  spec.summary       = "ssl termination, but on the other side"
  spec.description   = ""
  spec.homepage      = "https://github.com/telyn/the-insecure-proxy"
  spec.license       = "MIT"
  spec.required_ruby_version = ">= 2.4.0"

  spec.metadata["allowed_push_host"] = "https://rubygems.org"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = spec.homepage
  spec.metadata["changelog_uri"] = spec.homepage

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files = Dir["lib/**/*"].to_a +
               Dir["doc/*"].to_a +
               %w[config.ru README.txt]
  spec.bindir        = "exe"
  spec.executables   = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  spec.add_dependency "rack-proxy"

  # For more information and examples about making a new gem, checkout our
  # guide at: https://bundler.io/guides/creating_gem.html
end
