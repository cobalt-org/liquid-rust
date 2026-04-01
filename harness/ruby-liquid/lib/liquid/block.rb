# frozen_string_literal: true

module Liquid
  class Block < Tag
    attr_reader :nodelist

    def initialize(tag_name, markup, parse_context)
      super
      @body = BlockBody.new([])
      @nodelist = @body.nodelist
    end

    def render(_context)
      @nodelist.map(&:to_s).join
    end

    def unknown_tag(tag, _markup, _tokens)
      raise Liquid::SyntaxError, "Unknown tag '#{tag}'"
    end

    def blank?
      @body.blank?
    end
  end
end
