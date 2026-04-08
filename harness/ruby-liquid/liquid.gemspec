# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name = "liquid"
  spec.version = "0.1.0"
  spec.summary = "Rust-backed replacement gem for Shopify/liquid"
  spec.description = "Compatibility harness that routes Shopify/liquid tests into liquid-rust."
  spec.authors = ["Codex"]
  spec.email = ["noreply@example.com"]
  spec.files = Dir.chdir(__dir__) do
    Dir["Gemfile", "Rakefile", "lib/**/*.rb", "ext/**/*", "*.gemspec"]
  end
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/liquid_ext/extconf.rb"]

  spec.add_development_dependency "rake", "~> 13.0"
  spec.add_development_dependency "rake-compiler", "~> 1.2"
  spec.add_development_dependency "rb_sys", "~> 0.9.95"
end
