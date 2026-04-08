# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

crate_root = File.expand_path(__dir__)

create_rust_makefile("liquid/liquid_ext") do |builder|
  builder.ext_dir = crate_root
end
