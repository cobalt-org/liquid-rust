# frozen_string_literal: true

module Liquid
  class Variable
    FilterMarkupRegex = /#{FilterSeparator}\s*(.*)/om
    FilterParser = /(?:\s+|#{QuotedFragment}|#{ArgumentSeparator})+/o
    FilterArgsRegex = /(?:#{FilterArgumentSeparator}|#{ArgumentSeparator})\s*((?:\w+\s*\:\s*)?#{QuotedFragment})/o
    JustTagAttributes = /\A#{TagAttributes}\z/o
    MarkupWithQuotedFragment = /(#{QuotedFragment})(.*)/om

    attr_accessor :filters, :name, :line_number
    attr_reader :parse_context
    alias options parse_context

    include ParserSwitching

    def initialize(markup, parse_context)
      @markup = markup.to_s
      @name = nil
      @filters = []
      @parse_context = parse_context
      @line_number = parse_context.line_number

      parse_with_selected_parser(@markup)
    end

    def raw
      @markup
    end

    def markup_context(markup)
      %(in "{{#{markup}}}")
    end

    def lax_parse(markup)
      @filters = []
      return unless markup =~ MarkupWithQuotedFragment

      name_markup = Regexp.last_match(1)
      filter_markup = Regexp.last_match(2)
      @name = parse_context.parse_expression(name_markup)

      if filter_markup =~ FilterMarkupRegex
        Regexp.last_match(1).scan(FilterParser).each do |filter_source|
          next unless filter_source =~ /\w+/

          filter_name = Regexp.last_match(0)
          filter_args = filter_source.scan(FilterArgsRegex).flatten
          @filters << lax_parse_filter_expressions(filter_name, filter_args)
        end
      end
    end

    def strict_parse(markup)
      @filters = []
      parser = @parse_context.new_parser(markup)
      return if parser.look(:end_of_string)

      @name = parse_context.safe_parse_expression(parser)
      while parser.consume?(:pipe)
        filter_name = parser.consume(:id)
        filter_args = parser.consume?(:colon) ? parse_filterargs(parser) : []
        @filters << lax_parse_filter_expressions(filter_name, filter_args)
      end
      parser.consume(:end_of_string)
    end

    def strict2_parse(markup)
      @filters = []
      parser = @parse_context.new_parser(markup)
      return if parser.look(:end_of_string)

      @name = parse_context.safe_parse_expression(parser)
      @filters << strict2_parse_filter_expressions(parser) while parser.consume?(:pipe)
      parser.consume(:end_of_string)
    end

    def parse_filterargs(parser)
      filter_args = [parser.argument]
      filter_args << parser.argument while parser.consume?(:comma)
      filter_args
    end

    def render(context)
      obj = context.evaluate(@name)

      @filters.each do |filter_name, filter_args, filter_kwargs|
        args = evaluate_filter_expressions(context, filter_args, filter_kwargs)
        obj = context.invoke(filter_name, obj, *args)
      end

      context.apply_global_filter(obj)
    end

    def render_to_output_buffer(context, output)
      render_obj_to_output(render(context), output)
      output
    end

    private

    def lax_parse_filter_expressions(filter_name, unparsed_args)
      filter_args = []
      keyword_args = nil

      unparsed_args.each do |arg|
        if (matches = arg.match(JustTagAttributes))
          keyword_args ||= {}
          keyword_args[matches[1]] = parse_context.parse_expression(matches[2])
        else
          filter_args << parse_context.parse_expression(arg)
        end
      end

      result = [filter_name, filter_args]
      result << keyword_args if keyword_args
      result
    end

    def strict2_parse_filter_expressions(parser)
      filter_name = parser.consume(:id)
      filter_args = []
      keyword_args = {}

      if parser.consume?(:colon)
        argument(parser, filter_args, keyword_args) unless end_of_arguments?(parser)
        argument(parser, filter_args, keyword_args) while parser.consume?(:comma) && !end_of_arguments?(parser)
      end

      result = [filter_name, filter_args]
      result << keyword_args unless keyword_args.empty?
      result
    end

    def argument(parser, positional_arguments, keyword_arguments)
      if parser.look(:id) && parser.look(:colon, 1)
        key = parser.consume(:id)
        parser.consume(:colon)
        keyword_arguments[key] = parse_context.safe_parse_expression(parser)
      else
        positional_arguments << parse_context.safe_parse_expression(parser)
      end
    end

    def end_of_arguments?(parser)
      parser.look(:pipe) || parser.look(:end_of_string)
    end

    def evaluate_filter_expressions(context, filter_args, filter_kwargs)
      parsed_args = filter_args.map { |expr| context.evaluate(expr) }
      if filter_kwargs
        parsed_kwargs = {}
        filter_kwargs.each do |key, expr|
          parsed_kwargs[key] = context.evaluate(expr)
        end
        parsed_args << parsed_kwargs
      end
      parsed_args
    end

    def render_obj_to_output(obj, output)
      case obj
      when nil
        output
      when Array
        obj.each { |item| render_obj_to_output(item, output) }
      else
        output << obj.to_s
      end
    end

    class ParseTreeVisitor < Liquid::ParseTreeVisitor
      def children
        [@node.name] + @node.filters.flatten
      end
    end
  end
end
