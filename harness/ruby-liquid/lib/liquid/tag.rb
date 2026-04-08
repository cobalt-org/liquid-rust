# frozen_string_literal: true

module Liquid
  class Tag
    attr_reader :tag_name, :markup, :parse_context, :line_number, :raw
    alias options parse_context

    class << self
      def disable_tags(*tag_names)
        tag_names += disabled_tags
        define_singleton_method(:disabled_tags) { tag_names }
        prepend(Disabler)
      end

      protected

      def disabled_tags
        []
      end
    end

    def self.tag_name
      name.to_s.split("::").last.to_s.downcase
    end

    def self.parse(tag_name, markup, tokenizer, parse_context)
      tag = new(tag_name, markup, parse_context)
      tag.parse(tokenizer)
      tag
    end

    def initialize(tag_name, markup, parse_context)
      @tag_name = tag_name
      @markup = markup
      @parse_context = parse_context
      @line_number = parse_context&.line_number
      @raw = [tag_name, markup].reject(&:empty?).join(" ")
    end

    def name
      self.class.name.to_s.downcase
    end

    def parse(_tokenizer)
      nil
    end

    def render(_context)
      ""
    end

    def render_to_output_buffer(context, output)
      rendered = render(context)
      output << rendered.to_s unless rendered.nil?
      output
    end

    def blank?
      render(nil).to_s.empty?
    end
  end
end
