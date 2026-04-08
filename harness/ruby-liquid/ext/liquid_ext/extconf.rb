# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("liquid/liquid_ext") do |builder|
  builder.ext_dir = "../../../../crates/ruby-ext"
end
