# frozen_string_literal: true

module Liquid
  module AstTemplateRoot
    module_function

    def build(source, environment:, error_mode:, line_numbers:, locale:)
      ast_environment = build_environment(environment)
      parse_context = Liquid::ParseContext.new(
        environment: ast_environment,
        line_number: line_numbers ? 1 : nil,
        locale: locale,
        error_mode: error_mode
      )
      tokenizer = build_tokenizer(
        source,
        parse_context,
        environment: ast_environment,
        error_mode: error_mode,
        line_number: parse_context.line_number
      )
      nodes, tag_name, markup = parse_nodes(tokenizer, parse_context)
      raise_parse_nodes_error!(tag_name, markup, parse_context)
      Liquid::BlockBody.new(nodes)
    end

    def build_environment(environment)
      ast_environment = environment.dup
      {
        "assign" => Liquid::Assign,
        "break" => Liquid::Break,
        "capture" => Liquid::Capture,
        "case" => Liquid::Case,
        "comment" => Liquid::Comment,
        "continue" => Liquid::Continue,
        "cycle" => Liquid::Cycle,
        "doc" => Liquid::Doc,
        "echo" => Liquid::Echo,
        "for" => Liquid::For,
        "if" => Liquid::If,
        "ifchanged" => Liquid::IfChanged,
        "include" => Liquid::Include,
        "raw" => Liquid::Raw,
        "render" => Liquid::Render,
        "tablerow" => Liquid::TableRow,
        "unless" => Liquid::Unless,
      }.each do |name, klass|
        ast_environment.register_tag(name, klass)
      end
      ast_environment
    end

    def parse_nodes(tokenizer, parse_context)
      nodes = []

      loop do
        token_line = tokenizer.line_number
        token = tokenizer.shift
        break unless token
        next if token.empty?

        parse_context.line_number = token_line

        if tokenizer.for_liquid_tag
          stripped = token.strip
          next if stripped.empty?

          unless stripped =~ BlockBody::LiquidTagToken
            return [nodes, token, token]
          end

          tag_name = Regexp.last_match(1)
          markup = Regexp.last_match(2)

          if tag_name == "liquid"
            nodes.concat(parse_liquid_markup(markup, parse_context))
            next
          end

          tag_class = parse_context.environment.tag_for_name(tag_name)
          return [nodes, tag_name, markup] unless tag_class

          nodes << tag_class.parse(tag_name, markup, tokenizer, child_parse_context(parse_context, token_line))
          next
        end

        case
        when token.start_with?(BlockBody::TAGSTART)
          unless token =~ BlockBody::FullToken
            return [nodes, token, token]
          end

          tag_name = Regexp.last_match(2)
          markup = Regexp.last_match(4)

          if tag_name == "liquid"
            nodes.concat(parse_liquid_markup(markup, parse_context))
            next
          end

          tag_class = parse_context.environment.tag_for_name(tag_name)
          return [nodes, tag_name, markup] unless tag_class

          nodes << tag_class.parse(tag_name, markup, tokenizer, child_parse_context(parse_context, token_line))
        when token.start_with?(BlockBody::VARSTART)
          nodes << create_variable(token, child_parse_context(parse_context, token_line))
        else
          nodes << token
        end
      end

      [nodes, nil, nil]
    end

    def parse_liquid_markup(markup, parse_context)
      child_context = child_parse_context(parse_context, parse_context.line_number)
      tokenizer = build_tokenizer(
        markup,
        parse_context,
        environment: parse_context.environment,
        error_mode: parse_context.error_mode,
        line_number: parse_context.line_number,
        for_liquid_tag: true
      )
      nodes, tag_name, inner_markup = parse_nodes(tokenizer, child_context)
      raise_parse_nodes_error!(tag_name, inner_markup, child_context, inside_liquid_tag: true)
      nodes
    end

    def raise_parse_nodes_error!(tag_name, markup, parse_context, inside_liquid_tag: false)
      return unless tag_name || markup

      if inside_liquid_tag && tag_name&.start_with?("end")
        raise Liquid::SyntaxError.new(
          "'#{tag_name}' is not a valid delimiter for liquid tags. use %}",
          line_number: parse_context.line_number
        )
      end

      raise Liquid::SyntaxError.new(
        "Unknown tag '#{tag_name || markup}'",
        line_number: parse_context.line_number
      )
    end

    def build_tokenizer(source, parse_context, environment:, error_mode:, line_number:, for_liquid_tag: false)
      Liquid::Template::CustomBlockTokenizer.new(
        source: source,
        string_scanner: StringScanner.new(""),
        line_numbers: !line_number.nil?,
        line_number: line_number,
        for_liquid_tag: for_liquid_tag,
        environment: environment,
        error_mode: error_mode,
        block_tag_names: custom_block_tag_names(environment)
      )
    end

    def custom_block_tag_names(environment)
      Array(environment&.tags).filter_map do |tag_name, klass|
        tag_name.to_s if klass.is_a?(Class) && klass <= Liquid::Block
      end
    end

    def child_parse_context(parse_context, line_number)
      Liquid::ParseContext.new(
        environment: parse_context.environment,
        line_number: line_number,
        locale: parse_context.locale,
        error_mode: parse_context.error_mode,
        warnings: parse_context.warnings
      )
    end

    def create_variable(token, parse_context)
      if token.end_with?("}}")
        index = 2
        index = 3 if token[index] == "-"
        parse_end = token.length - 3
        parse_end -= 1 if token[parse_end] == "-"
        markup_end = parse_end - index + 1
        markup = markup_end <= 0 ? "" : token.slice(index, markup_end)

        return Liquid::Variable.new(markup, parse_context)
      end

      raise Liquid::SyntaxError, "Variable was not properly terminated"
    end
  end

  class Assign < Tag
    Syntax = /(#{VariableSignature}+)\s*=\s*(.*)\s*/om

    def self.raise_syntax_error(parse_context)
      raise Liquid::SyntaxError, parse_context.locale.t("errors.syntax.assign")
    end

    attr_reader :to, :from

    def initialize(tag_name, markup, parse_context)
      super
      if markup =~ Syntax
        @to = Regexp.last_match(1)
        @from = Variable.new(Regexp.last_match(2), parse_context)
      else
        self.class.raise_syntax_error(parse_context)
      end
    end

    def blank?
      true
    end

    class ParseTreeVisitor < Liquid::ParseTreeVisitor
      def children
        [@node.from]
      end
    end
  end

  class Break < Tag
  end

  class Continue < Tag
  end

  class Echo < Tag
    attr_reader :variable

    def initialize(tag_name, markup, parse_context)
      super
      @variable = Variable.new(markup, parse_context)
    end

    class ParseTreeVisitor < Liquid::ParseTreeVisitor
      def children
        [@node.variable]
      end
    end
  end

  class Cycle < Tag
    SimpleSyntax = /\A#{QuotedFragment}+/o
    NamedSyntax = /\A(#{QuotedFragment})\s*\:\s*(.*)/om
    UNNAMED_CYCLE_PATTERN = /\w+:0x\h{8}/

    attr_reader :variables

    include ParserSwitching

    def initialize(tag_name, markup, options)
      super
      parse_with_selected_parser(markup)
    end

    private

    def strict2_parse(markup)
      parser = @parse_context.new_parser(markup)

      @variables = []
      raise SyntaxError, options[:locale].t("errors.syntax.cycle") if parser.look(:end_of_string)

      first_expression = @parse_context.safe_parse_expression(parser)
      if parser.look(:colon)
        @name = first_expression
        @is_named = true
        parser.consume(:colon)
        @variables << maybe_dup_lookup(@parse_context.safe_parse_expression(parser))
      else
        @variables << maybe_dup_lookup(first_expression)
      end

      while parser.consume?(:comma)
        break if parser.look(:end_of_string)

        @variables << maybe_dup_lookup(@parse_context.safe_parse_expression(parser))
      end

      parser.consume(:end_of_string)

      unless @is_named
        @name = @variables.to_s
        @is_named = !@name.match?(UNNAMED_CYCLE_PATTERN)
      end
    end

    def strict_parse(markup)
      lax_parse(markup)
    end

    def lax_parse(markup)
      case markup
      when NamedSyntax
        @variables = variables_from_string(Regexp.last_match(2))
        @name = @parse_context.parse_expression(Regexp.last_match(1))
        @is_named = true
      when SimpleSyntax
        @variables = variables_from_string(markup)
        @name = @variables.to_s
        @is_named = !@name.match?(UNNAMED_CYCLE_PATTERN)
      else
        raise SyntaxError, options[:locale].t("errors.syntax.cycle")
      end
    end

    def variables_from_string(markup)
      markup.split(",").collect do |variable|
        variable =~ /\s*(#{QuotedFragment})\s*/o
        next unless Regexp.last_match(1)

        parsed = @parse_context.parse_expression(Regexp.last_match(1))
        maybe_dup_lookup(parsed)
      end.compact
    end

    def maybe_dup_lookup(variable)
      variable.is_a?(VariableLookup) ? variable.dup : variable
    end

    class ParseTreeVisitor < Liquid::ParseTreeVisitor
      def children
        Array(@node.variables)
      end
    end
  end

  class Capture < Block
    Syntax = /(#{VariableSignature}+)/o

    def initialize(tag_name, markup, options)
      super
      if markup =~ Syntax
        @to = Regexp.last_match(1)
      else
        raise SyntaxError, options[:locale].t("errors.syntax.capture")
      end
    end

    def parse(tokens)
      parse_body(@body, tokens)
      @body.remove_blank_strings if blank?
    end
  end

  class If < Block
    Syntax = /(#{QuotedFragment})\s*([=!<>a-z_]+)?\s*(#{QuotedFragment})?/o
    ExpressionsAndOperators = /(?:\b(?:\s?and\s?|\s?or\s?)\b|(?:\s*(?!\b(?:\s?and\s?|\s?or\s?)\b)(?:#{QuotedFragment}|\S+)\s*)+)/o
    BOOLEAN_OPERATORS = %w[and or].freeze

    attr_reader :blocks

    include ParserSwitching

    def initialize(tag_name, markup, options)
      super
      @blocks = []
      push_block("if", markup)
    end

    def nodelist
      @blocks.map(&:attachment)
    end

    def parse(tokens)
      while parse_body(@blocks.last.attachment, tokens)
      end
      @blocks.reverse_each do |block|
        block.attachment.remove_blank_strings if blank?
      end
    end

    def unknown_tag(tag, markup, tokens)
      if %w[elsif else].include?(tag)
        push_block(tag, markup)
      else
        super
      end
    end

    private

    def strict2_parse(markup)
      strict_parse(markup)
    end

    def push_block(tag, markup)
      block = tag == "else" ? ElseCondition.new : parse_with_selected_parser(markup)
      block.attach(new_body)
      @blocks << block
    end

    def parse_expression(markup, safe: false)
      Condition.parse_expression(parse_context, markup, safe: safe)
    end

    def lax_parse(markup)
      expressions = markup.scan(ExpressionsAndOperators)
      raise SyntaxError, options[:locale].t("errors.syntax.if") unless expressions.pop =~ Syntax

      condition = Condition.new(parse_expression(Regexp.last_match(1)), Regexp.last_match(2), parse_expression(Regexp.last_match(3)))

      until expressions.empty?
        operator = expressions.pop.to_s.strip
        raise SyntaxError, options[:locale].t("errors.syntax.if") unless expressions.pop.to_s =~ Syntax

        new_condition = Condition.new(parse_expression(Regexp.last_match(1)), Regexp.last_match(2), parse_expression(Regexp.last_match(3)))
        raise SyntaxError, options[:locale].t("errors.syntax.if") unless BOOLEAN_OPERATORS.include?(operator)

        new_condition.send(operator, condition)
        condition = new_condition
      end

      condition
    end

    def strict_parse(markup)
      parser = @parse_context.new_parser(markup)
      condition = parse_binary_comparisons(parser)
      parser.consume(:end_of_string)
      condition
    end

    def parse_binary_comparisons(parser)
      condition = parse_comparison(parser)
      first_condition = condition
      while (operator = parser.id?("and") || parser.id?("or"))
        child_condition = parse_comparison(parser)
        condition.send(operator, child_condition)
        condition = child_condition
      end
      first_condition
    end

    def parse_comparison(parser)
      left = parse_expression(parser.expression, safe: true)
      if (operator = parser.consume?(:comparison))
        right = parse_expression(parser.expression, safe: true)
        Condition.new(left, operator, right)
      else
        Condition.new(left)
      end
    end

    class ParseTreeVisitor < Liquid::ParseTreeVisitor
      def children
        @node.blocks
      end
    end
  end

  class Unless < If
  end

  class For < Block
    Syntax = /\A(#{VariableSegment}+)\s+in\s+(#{QuotedFragment}+)\s*(reversed)?/o

    attr_reader :collection_name, :variable_name, :limit, :from

    include ParserSwitching

    def initialize(tag_name, markup, options)
      super
      @from = @limit = nil
      parse_with_selected_parser(markup)
      @for_block = new_body
      @else_block = nil
    end

    def parse(tokens)
      if parse_body(@for_block, tokens)
        parse_body(@else_block, tokens)
      end
      if blank?
        @else_block&.remove_blank_strings
        @for_block.remove_blank_strings
      end
    end

    def nodelist
      @else_block ? [@for_block, @else_block] : [@for_block]
    end

    def unknown_tag(tag, _markup, _tokens)
      return super unless tag == "else"

      @else_block = new_body
    end

    protected

    def lax_parse(markup)
      if markup =~ Syntax
        @variable_name = Regexp.last_match(1)
        collection_name = Regexp.last_match(2)
        @reversed = !Regexp.last_match(3).nil?
        @name = "#{@variable_name}-#{collection_name}"
        @collection_name = parse_expression(collection_name)
        markup.scan(TagAttributes) do |key, value|
          set_attribute(key, value)
        end
      else
        raise SyntaxError, options[:locale].t("errors.syntax.for")
      end
    end

    def strict_parse(markup)
      parser = @parse_context.new_parser(markup)
      @variable_name = parser.consume(:id)
      raise SyntaxError, options[:locale].t("errors.syntax.for_invalid_in") unless parser.id?("in")

      collection_name = parser.expression
      @collection_name = parse_expression(collection_name, safe: true)
      @name = "#{@variable_name}-#{collection_name}"
      @reversed = parser.id?("reversed")

      while parser.look(:comma) || parser.look(:id)
        parser.consume?(:comma)
        attribute = parser.id?("limit") || parser.id?("offset")
        raise SyntaxError, options[:locale].t("errors.syntax.for_invalid_attribute") unless attribute

        parser.consume(:colon)
        set_attribute(attribute, parser.expression, safe: true)
      end
      parser.consume(:end_of_string)
    end

    private

    def strict2_parse(markup)
      strict_parse(markup)
    end

    def parse_expression(markup, safe: false)
      @parse_context.parse_expression(markup, safe: safe)
    end

    def set_attribute(key, expression, safe: false)
      case key
      when "offset"
        @from =
          if expression == "continue"
            :continue
          else
            parse_expression(expression, safe: safe)
          end
      when "limit"
        @limit = parse_expression(expression, safe: safe)
      end
    end

    class ParseTreeVisitor < Liquid::ParseTreeVisitor
      def children
        (super + [@node.limit, @node.from, @node.collection_name]).compact
      end
    end
  end

  class IfChanged < Block
    def parse(tokens)
      parse_body(@body, tokens)
      @body.remove_blank_strings if blank?
    end
  end

  class TableRow < Block
    Syntax = /(\w+)\s+in\s+(#{QuotedFragment}+)/o
    ALLOWED_ATTRIBUTES = %w[cols limit offset range].freeze

    attr_reader :variable_name, :collection_name, :attributes

    include ParserSwitching

    def initialize(tag_name, markup, options)
      super
      parse_with_selected_parser(markup)
    end

    def parse(tokens)
      parse_body(@body, tokens)
      @body.remove_blank_strings if blank?
    end

    def strict2_parse(markup)
      parser = @parse_context.new_parser(markup)
      @variable_name = parser.consume(:id)
      raise SyntaxError, options[:locale].t("errors.syntax.for_invalid_in") unless parser.id?("in")

      @collection_name = @parse_context.safe_parse_expression(parser)
      parser.consume?(:comma)

      @attributes = {}
      while parser.look(:id)
        key = parser.consume
        raise SyntaxError, options[:locale].t("errors.syntax.table_row_invalid_attribute", attribute: key) unless ALLOWED_ATTRIBUTES.include?(key)

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
      if markup =~ Syntax
        @variable_name = Regexp.last_match(1)
        @collection_name = @parse_context.parse_expression(Regexp.last_match(2))
        @attributes = {}
        markup.scan(TagAttributes) do |key, value|
          @attributes[key] = @parse_context.parse_expression(value)
        end
      else
        raise SyntaxError, options[:locale].t("errors.syntax.table_row")
      end
    end

    class ParseTreeVisitor < Liquid::ParseTreeVisitor
      def children
        super + @node.attributes.values + [@node.collection_name]
      end
    end
  end

  class Case < Block
    Syntax = /(#{QuotedFragment})/o
    WhenSyntax = /(#{QuotedFragment})(?:(?:\s+or\s+|\s*\,\s*)(#{QuotedFragment}.*))?/om

    attr_reader :blocks, :left

    include ParserSwitching

    def initialize(tag_name, markup, options)
      super
      @blocks = []
      parse_with_selected_parser(markup)
    end

    def parse(tokens)
      body = case_body = new_body
      body = @blocks.last.attachment while parse_body(body, tokens)
      @blocks.reverse_each do |condition|
        body = condition.attachment
        body.remove_blank_strings if blank?
      end
      case_body
    end

    def nodelist
      @blocks.map(&:attachment)
    end

    def unknown_tag(tag, markup, _tokens)
      case tag
      when "when"
        record_when_condition(markup)
      when "else"
        record_else_condition(markup)
      else
        super
      end
    end

    private

    def strict2_parse(markup)
      parser = @parse_context.new_parser(markup)
      @left = @parse_context.safe_parse_expression(parser)
      parser.consume(:end_of_string)
    end

    def strict_parse(markup)
      lax_parse(markup)
    end

    def lax_parse(markup)
      if markup =~ Syntax
        @left = @parse_context.parse_expression(Regexp.last_match(1))
      else
        raise SyntaxError, options[:locale].t("errors.syntax.case")
      end
    end

    def record_when_condition(markup)
      body = new_body

      if @parse_context.error_mode == :strict2
        parser = @parse_context.new_parser(markup)
        loop do
          expression = Condition.parse_expression(parse_context, parser.expression, safe: true)
          block = Condition.new(@left, "==", expression)
          block.attach(body)
          @blocks << block
          break unless parser.id?("or") || parser.consume?(:comma)
        end
        parser.consume(:end_of_string)
      else
        while markup
          raise SyntaxError, options[:locale].t("errors.syntax.case_invalid_when") unless markup =~ WhenSyntax

          markup = Regexp.last_match(2)
          block = Condition.new(@left, "==", Condition.parse_expression(parse_context, Regexp.last_match(1)))
          block.attach(body)
          @blocks << block
        end
      end
    end

    def record_else_condition(markup)
      raise SyntaxError, options[:locale].t("errors.syntax.case_invalid_else") unless markup.strip.empty?

      block = ElseCondition.new
      block.attach(new_body)
      @blocks << block
    end

    class ParseTreeVisitor < Liquid::ParseTreeVisitor
      def children
        [@node.left] + @node.blocks
      end
    end
  end

  class Comment < Block
    def parse(tokens)
      parse_body(nil, tokens)
    end

    def render_to_output_buffer(_context, output)
      output
    end

    def unknown_tag(_tag, _markup, _tokens)
    end

    def blank?
      true
    end

    private

    def parse_body(_body, tokenizer)
      if parse_context.depth >= MAX_DEPTH
        raise StackLevelError, "Nesting too deep"
      end

      parse_context.depth += 1
      comment_tag_depth = 1

      begin
        while (token = tokenizer.send(:shift))
          tag_name =
            if tokenizer.for_liquid_tag
              next if token.empty? || token.match?(BlockBody::WhitespaceOrNothing)

              tag_name_match = BlockBody::LiquidTagToken.match(token)
              next if tag_name_match.nil?
              tag_name_match[1]
            else
              token =~ BlockBody::FullToken
              Regexp.last_match(2)
            end

          case tag_name
          when "raw"
            parse_raw_tag_body(tokenizer)
          when "comment"
            comment_tag_depth += 1
          when "endcomment"
            comment_tag_depth -= 1
          end

          return false if comment_tag_depth.zero?
        end

        raise_tag_never_closed(block_name)
      ensure
        parse_context.depth -= 1
      end

      false
    end

    def parse_raw_tag_body(tokenizer)
      while (token = tokenizer.send(:shift))
        return if token =~ BlockBody::FullTokenPossiblyInvalid && Regexp.last_match(2) == "endraw"
      end

      raise_tag_never_closed("raw")
    end
  end

  class Doc < Block
    NO_UNEXPECTED_ARGS = /\A\s*\z/

    def initialize(tag_name, markup, parse_context)
      super
      ensure_valid_markup(tag_name, markup, parse_context)
    end

    def parse(tokens)
      @body = +""

      while (token = tokens.shift)
        tag_name = token =~ BlockBody::FullTokenPossiblyInvalid && Regexp.last_match(2)
        raise_nested_doc_error if tag_name == @tag_name

        if tag_name == block_delimiter
          @body << Regexp.last_match(1) if Regexp.last_match(1) != ""
          return
        end

        @body << token unless token.empty?
      end

      return if options[:custom_block_body_only]

      raise_tag_never_closed(block_name)
    end

    def render_to_output_buffer(_context, output)
      output
    end

    def blank?
      @body.empty?
    end

    def nodelist
      [@body]
    end

    private

    def ensure_valid_markup(tag_name, markup, parse_context)
      return if NO_UNEXPECTED_ARGS.match?(markup)

      raise SyntaxError, parse_context.locale.t("errors.syntax.block_tag_unexpected_args", tag: tag_name)
    end

    def raise_nested_doc_error
      raise SyntaxError, parse_context.locale.t("errors.syntax.doc_invalid_nested")
    end
  end

  class Raw < Block
    def parse(tokens)
      @body = +""

      while (token = tokens.shift)
        tag_name = token =~ BlockBody::FullTokenPossiblyInvalid && Regexp.last_match(2)

        if tag_name == block_delimiter
          @body << Regexp.last_match(1) if Regexp.last_match(1) != ""
          return
        end

        @body << token unless token.empty?
      end

      raise_tag_never_closed(block_name)
    end

    def nodelist
      [@body]
    end

    def blank?
      @body.to_s.empty?
    end
  end
end
