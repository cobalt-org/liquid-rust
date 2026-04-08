# frozen_string_literal: true

module Liquid
  class Include < Tag
    prepend Tag::Disableable

    SYNTAX = /(#{QuotedFragment}+)(\s+(?:with|for)\s+(#{QuotedFragment}+))?(\s+(?:as)\s+(#{VariableSegment}+))?/o
    Syntax = SYNTAX

    attr_reader :template_name_expr, :variable_name_expr, :attributes

    include ParserSwitching

    def initialize(tag_name, markup, options)
      super
      parse_with_selected_parser(markup)
    end

    def parse(_tokens)
    end

    def render_to_output_buffer(context, output)
      template_name = context.evaluate(@template_name_expr)
      raise Liquid::ArgumentError, options.locale.t("errors.argument.include") unless template_name.is_a?(String)

      partial = PartialCache.load(
        template_name,
        context: context,
        parse_context: parse_context
      )

      context_variable_name = @alias_name || template_name.split("/").last
      variable =
        if @variable_name_expr
          context.evaluate(@variable_name_expr)
        else
          context.find_variable(template_name, raise_on_not_found: false)
        end

      old_template_name = context.template_name
      old_partial = context.partial

      begin
        context.template_name = partial.name
        context.partial = true

        context.stack do
          @attributes.each do |key, value|
            context[key] = context.evaluate(value)
          end

          if variable.is_a?(Array)
            variable.each do |item|
              context[context_variable_name] = item
              partial.render_to_output_buffer(context, output)
            end
          else
            context[context_variable_name] = variable
            partial.render_to_output_buffer(context, output)
          end
        end
      ensure
        context.template_name = old_template_name
        context.partial = old_partial
      end

      output
    end

    alias_method :options, :parse_context

    private

    def strict2_parse(markup)
      parser = @parse_context.new_parser(markup)

      @template_name_expr = @parse_context.safe_parse_expression(parser)
      @variable_name_expr = @parse_context.safe_parse_expression(parser) if parser.id?("for") || parser.id?("with")
      @alias_name = parser.consume(:id) if parser.id?("as")

      parser.consume?(:comma)

      @attributes = {}
      while parser.look(:id)
        key = parser.consume
        parser.consume(:colon)
        @attributes[key] = @parse_context.safe_parse_expression(parser)
        parser.consume?(:comma)
      end

      parser.consume(:end_of_string)
    end

    def strict_parse(markup)
      lax_parse(markup)
    end

    def lax_parse(markup)
      raise Liquid::SyntaxError, options.locale.t("errors.syntax.include") unless markup =~ SYNTAX

      template_name = Regexp.last_match(1)
      variable_name = Regexp.last_match(3)

      @alias_name = Regexp.last_match(5)
      @variable_name_expr = variable_name ? @parse_context.parse_expression(variable_name) : nil
      @template_name_expr = @parse_context.parse_expression(template_name)
      @attributes = {}

      markup.scan(TagAttributes) do |key, value|
        @attributes[key] = @parse_context.parse_expression(value)
      end
    end

    class ParseTreeVisitor < Liquid::ParseTreeVisitor
      def children
        [
          @node.template_name_expr,
          @node.variable_name_expr,
        ] + @node.attributes.values
      end
    end
  end
end
