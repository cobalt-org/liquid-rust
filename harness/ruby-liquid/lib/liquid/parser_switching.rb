# frozen_string_literal: true

module Liquid
  module ParserSwitching
    def parse_with_selected_parser(markup)
      case parse_context.error_mode
      when :strict2 then strict2_parse_with_error_context(markup)
      when :strict then strict_parse_with_error_context(markup)
      when :lax then lax_parse(markup)
      when :warn
        begin
          strict2_parse_with_error_context(markup)
        rescue SyntaxError => error
          parse_context.warnings << error
          lax_parse(markup)
        end
      else
        lax_parse(markup)
      end
    end

    private

    def strict2_parse_with_error_context(markup)
      strict2_parse(markup)
    rescue SyntaxError => error
      error.line_number = line_number
      error.markup_context = markup_context(markup)
      raise error
    end

    def strict_parse_with_error_context(markup)
      strict_parse(markup)
    rescue SyntaxError => error
      error.line_number = line_number
      error.markup_context = markup_context(markup)
      raise error
    end

    def markup_context(markup)
      %(in "#{markup.strip}")
    end
  end
end
