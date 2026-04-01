# frozen_string_literal: true

require "json"
require "strscan"
require "liquid/liquid_ext"

module Liquid
  VERSION = "0.1.0"
  RUST_BACKED = true
  RAISE_EXCEPTION_LAMBDA = ->(_error) { raise }

  class << self
    attr_accessor :cache_classes
  end

  self.cache_classes = false

  module Usage
    module_function

    def increment(_name); end
  end

  class I18n
    attr_reader :path

    def initialize(path = nil)
      @path = path
    end
  end
end

require "liquid/errors"
require "liquid/resource_limits"
require "liquid/registers"
require "liquid/environment"
require "liquid/parse_context"
require "liquid/block_body"
require "liquid/variable"
require "liquid/variable_lookup"
require "liquid/drop"
require "liquid/tag"
require "liquid/block"
require "liquid/context"
require "liquid/template"
require "liquid/profiler"
require "liquid/test_reporter"
