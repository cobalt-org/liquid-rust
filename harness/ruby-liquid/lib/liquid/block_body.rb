# frozen_string_literal: true

module Liquid
  class BlockBody
    LiquidTagToken = /\A\s*(#{TagName})\s*(.*?)\z/o
    FullToken = /\A#{TagStart}#{WhitespaceControl}?(\s*)(#{TagName})(\s*)(.*?)#{WhitespaceControl}?#{TagEnd}\z/om
    FullTokenPossiblyInvalid = /\A(.*)#{TagStart}#{WhitespaceControl}?\s*(\w+)\s*(.*)?#{WhitespaceControl}?#{TagEnd}\z/om
    WhitespaceOrNothing = /\A\s*\z/
    TAGSTART = "{%"
    VARSTART = "{{"

    attr_reader :nodelist

    def initialize(nodelist = [])
      @nodelist = Array(nodelist)
    end

    def self.from_native(native)
      new(native || [])
    end

    def blank?
      @nodelist.all? do |node|
        node.respond_to?(:blank?) ? node.blank? : node.to_s.empty?
      end
    end

    def remove_blank_strings
      raise "remove_blank_strings only support being called on a blank block body" unless blank?

      @nodelist.reject! { |node| node.instance_of?(String) }
    end

    def render(context)
      render_to_output_buffer(context, +"")
    end

    def render_to_output_buffer(context, output)
      @nodelist.each do |node|
        if node.is_a?(String)
          output << node
        elsif node.respond_to?(:render_to_output_buffer)
          node.render_to_output_buffer(context, output)
        else
          output << node.to_s
        end

        break if context&.interrupt?
      end

      output
    end
  end
end
