# frozen_string_literal: true

module Liquid
  class Render < Tag
    prepend Tag::Disableable

    FOR = "for"
    SYNTAX = /(#{QuotedString}+)(\s+(with|#{FOR})\s+(#{QuotedFragment}+))?(\s+(?:as)\s+(#{VariableSegment}+))?/o

    disable_tags "include"

    attr_reader :template_name_expr, :variable_name_expr, :attributes, :alias_name

    include ParserSwitching
    alias_method :options, :parse_context

    def initialize(tag_name, markup, options)
      super
      parse_with_selected_parser(markup)
    end

    def for_loop?
      @is_for_loop
    end

    def render_to_output_buffer(context, output)
      template_name = @template_name_expr
      raise Liquid::ArgumentError, options.locale.t("errors.syntax.render") unless template_name.is_a?(String)

      partial = PartialCache.load(
        template_name,
        context: context,
        parse_context: parse_context,
        parse_options: { include_options_blacklist: [] }
      )

      context_variable_name = @alias_name || template_name.split("/").last

      render_partial = lambda do |var, forloop|
        inner_context = context.new_isolated_subcontext
        inner_context.template_name = partial.name
        inner_context.partial = true
        inner_context["forloop"] = forloop if forloop

        @attributes.each do |key, value|
          inner_context[key] = context.evaluate(value)
        end
        inner_context[context_variable_name] = var unless var.nil?
        fragment = +""
        partial.render_to_output_buffer(inner_context, fragment)
        rewrite_fragment_errors!(fragment, partial.errors, partial.name, context)
        output << fragment
        forloop&.send(:increment!)
      end

      variable = @variable_name_expr ? context.evaluate(@variable_name_expr) : nil
      if @is_for_loop && variable.respond_to?(:each) && variable.respond_to?(:count)
        forloop = Liquid::ForloopDrop.new(template_name, variable.count, nil)
        variable.each { |var| render_partial.call(var, forloop) }
      else
        render_partial.call(variable, nil)
      end

      output
    end

    def strict2_parse(markup)
      parser = @parse_context.new_parser(markup)

      @template_name_expr = @parse_context.parse_expression(parser.consume(:string), safe: true)
      with_or_for = parser.id?("for") || parser.id?("with")
      @variable_name_expr = @parse_context.safe_parse_expression(parser) if with_or_for
      @alias_name = parser.consume(:id) if parser.id?("as")
      @is_for_loop = (with_or_for == FOR)

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
      raise Liquid::SyntaxError, options.locale.t("errors.syntax.render") unless markup =~ SYNTAX

      template_name = Regexp.last_match(1)
      with_or_for = Regexp.last_match(3)
      variable_name = Regexp.last_match(4)

      @alias_name = Regexp.last_match(6)
      @variable_name_expr = variable_name ? @parse_context.parse_expression(variable_name) : nil
      @template_name_expr = @parse_context.parse_expression(template_name)
      @is_for_loop = (with_or_for == FOR)

      @attributes = {}
      markup.scan(TagAttributes) do |key, value|
        @attributes[key] = @parse_context.parse_expression(value)
      end
    end

    def rewrite_fragment_errors!(fragment, errors, template_name, context)
      Array(errors).each do |error|
        next unless error.is_a?(Liquid::Error)
        next if error.template_name && error.template_name != template_name

        previous = error.to_s
        error.template_name ||= template_name
        error.line_number ||= 1
        current = error.to_s
        context.errors << error unless context.errors.include?(error)
        next if previous == current

        fragment.sub!(previous, current)
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
