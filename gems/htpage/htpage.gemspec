# frozen_string_literal: true

require_relative "lib/htpage/version"

Gem::Specification.new do |spec|
  spec.name = "htpage"
  spec.version = Htpage::VERSION
  spec.authors = ["Mustafa Naeem"]
  spec.email = ["mustafa.naeem.iq@gmail.com"]
  spec.summary = "Native in-process compiler and Rails template handler for htpage"
  spec.description = "High-performance markup language compiler powered by a pure Rust core"
  spec.homepage = "https://github.com/mustafa-naeem-iq/htpage"
  spec.license = "MIT"
  spec.files = Dir["lib/**/*.{rb,bundle,so}", "README.md", "LICENSE*"]
  spec.require_paths = ["lib"]
  spec.required_ruby_version = ">= 3.0.0"
  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = spec.homepage
end
