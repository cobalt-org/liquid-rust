# frozen_string_literal: true

require "json"
require "securerandom"
require "strscan"
require "time"
require "bigdecimal"
require "liquid/version"
require "liquid/liquid_ext"
require "liquid/rust_extension/live_scope_value_proxy"

module Liquid
  RUST_BACKED = true
  FilterSeparator = /\|/
  ArgumentSeparator = ","
  FilterArgumentSeparator = ":"
  VariableAttributeSeparator = "."
  WhitespaceControl = "-"
  TagStart = /\{\%/
  TagEnd = /\%\}/
  TagName = /#|\w+/
  VariableSignature = /\(?[\w\-\.\[\]]\)?/
  VariableSegment = /[\w\-]/
  VariableStart = /\{\{/
  VariableEnd = /\}\}/
  VariableIncompleteEnd = /\}\}?/
  QuotedString = /"[^"]*"|'[^']*'/
  QuotedFragment = /#{QuotedString}|(?:[^\s,\|'"]|#{QuotedString})+/o
  TagAttributes = /(\w[\w-]*)\s*\:\s*(#{QuotedFragment})/o
  AnyStartingTag = /#{TagStart}|#{VariableStart}/o
  PartialTemplateParser = /#{TagStart}.*?#{TagEnd}|#{VariableStart}.*?#{VariableIncompleteEnd}/om
  TemplateParser = /(#{PartialTemplateParser}|#{AnyStartingTag})/om
  VariableParser = /\[(?>[^\[\]]+|\g<0>)*\]|#{VariableSegment}+\??/o
  RAISE_EXCEPTION_LAMBDA = ->(_error) { raise }
  HAS_STRING_SCANNER_SCAN_BYTE = StringScanner.instance_methods.include?(:scan_byte)

  class << self
    attr_accessor :cache_classes
  end

  self.cache_classes = false

  module Usage
    module_function

    def increment(_name); end
  end
end

require "liquid/i18n"
require "liquid/errors"
require "liquid/resource_limits"
require "liquid/registers"
require "liquid/strainer_template"
require "liquid/template_factory"
require "liquid/file_system"
require "liquid/standardfilters"
require "liquid/environment"
require "liquid/lexer"
require "liquid/parser"
require "liquid/parser_switching"
require "liquid/parse_context"
require "liquid/tokenizer"
require "liquid/block_body"
require "liquid/utils"
require "liquid/parse_tree_visitor"
require "liquid/condition"
require "liquid/variable"
require "liquid/variable_lookup"
require "liquid/range_lookup"
require "liquid/expression"
require "liquid/partial_cache"
require "liquid/drop"
require "liquid/forloop_drop"
require "liquid/tag"
require "liquid/tag/disabler"
require "liquid/tag/disableable"
require "liquid/block"
require "liquid/include"
require "liquid/render"
require "liquid/ast_tags"
require "liquid/increment"
require "liquid/decrement"
require "liquid/context"
require "liquid/template"
require "liquid/profiler"
require "liquid/test_reporter"

Liquid::Environment.default.register_tag("render", Liquid::Render)
Liquid::Environment.default.register_tag("increment", Liquid::Increment)
Liquid::Environment.default.register_tag("decrement", Liquid::Decrement)
