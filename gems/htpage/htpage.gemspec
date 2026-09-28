# frozen_string_literal: true

require_relative "lib/htpage/version"

Gem::Specification.new do |spec|
  spec.name = "htpage"
  spec.version = Htpage::VERSION
  spec.authors = ["Mustafa Naeem"]
  spec.summary = "Native in-process compiler and Rails template handler for htpage"
  spec.description = "High-performance markup language compiler powered by a pure Rust core"
  spec.files = Dir["lib/**/*.rb", "README.md", "LICENSE*"]
  spec.require_paths = ["lib"]
  spec.required_ruby_version = ">= 3.0.0"
end
