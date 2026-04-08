# frozen_string_literal: true

module Liquid
  class Profiler
    include Enumerable

    PROFILE_EPSILON = [Process.clock_getres(Process::CLOCK_MONOTONIC), 1e-6].max

    class Timing
      attr_reader :code, :template_name, :line_number, :children
      attr_accessor :total_time

      alias_method :render_time, :total_time
      alias_method :partial, :template_name

      def initialize(code: nil, template_name: nil, line_number: nil)
        @code = code
        @template_name = template_name
        @line_number = line_number
        @children = []
        @total_time = 0.0
      end

      def self_time
        total_time - @children.sum(&:total_time)
      end
    end

    class Builder
      TAG_TOKEN = /\A\{%-?\s*(#{Liquid::TagName})\s*(.*?)\s*-?%\}\z/m
      FOR_SYNTAX = /\A(#{Liquid::VariableSegment}+)\s+in\s+(.*)\z/o

      VariableNode = Struct.new(:raw, :line_number, keyword_init: true)
      TagNode = Struct.new(:name, :raw, :markup, :line_number, keyword_init: true)
      IfNode = Struct.new(:raw, :line_number, :condition_markup, :body, :elsif_branches, :else_body, keyword_init: true)
      ForNode = Struct.new(:raw, :line_number, :variable_name, :collection_markup, :body, keyword_init: true)

      def self.build(source:, context:)
        new(source: source, context: context).build
      end

      def initialize(source:, context:)
        @source = source.to_s
        @context = context
      end

      def build
        descriptors = parse_nodes(@source)
        build_timings(descriptors)
      end

      private

      def parse_nodes(source, line_number: 1, stop_tags: [])
        tokenizer = Liquid::Tokenizer.new(
          source: source,
          string_scanner: StringScanner.new(""),
          line_numbers: true,
          line_number: line_number
        )
        parse_tokenizer(tokenizer, stop_tags: stop_tags).first
      end

      def parse_tokenizer(tokenizer, stop_tags:)
        nodes = []

        loop do
          token_line = tokenizer.line_number
          token = tokenizer.shift
          break unless token
          next if token.empty?

          if token.start_with?("{%")
            match = TAG_TOKEN.match(token)
            next unless match

            name = match[1]
            markup = match[2].to_s
            return [nodes, name, markup] if stop_tags.include?(name)

            case name
            when "if"
              body, stop_tag, stop_markup = parse_tokenizer(tokenizer, stop_tags: %w[endif else elsif])
              elsif_branches = []
              else_body = []

              while stop_tag == "elsif"
                elsif_markup = stop_markup
                elsif_body, stop_tag, stop_markup = parse_tokenizer(tokenizer, stop_tags: %w[endif else elsif])
                elsif_branches << {
                  condition_markup: elsif_markup,
                  body: elsif_body
                }
              end

              if stop_tag == "else"
                else_body, stop_tag, = parse_tokenizer(tokenizer, stop_tags: ["endif"])
              end

              nodes << IfNode.new(
                raw: join_tag_raw(name, markup),
                line_number: token_line,
                condition_markup: markup,
                body: body,
                elsif_branches: elsif_branches,
                else_body: else_body
              )
            when "for"
              body, = parse_tokenizer(tokenizer, stop_tags: ["endfor"])
              variable_name, collection_markup = parse_for_markup(markup)
              nodes << ForNode.new(
                raw: join_tag_raw(name, markup),
                line_number: token_line,
                variable_name: variable_name,
                collection_markup: collection_markup,
                body: body
              )
            else
              nodes << TagNode.new(
                name: name,
                raw: join_tag_raw(name, markup),
                markup: markup,
                line_number: token_line
              )
            end
          elsif token.start_with?("{{")
            raw = extract_variable_raw(token)
            nodes << VariableNode.new(raw: raw, line_number: token_line)
          end
        end

        [nodes, nil, nil]
      end

      def build_timings(nodes, partial_name: nil)
        nodes.filter_map do |node|
          build_timing(node, partial_name: partial_name)
        end
      end

      def build_timing(node, partial_name: nil)
        case node
        when VariableNode
          leaf_timing(code: node.raw, line_number: node.line_number, partial_name: partial_name)
        when TagNode
          timing = leaf_timing(code: node.raw, line_number: node.line_number, partial_name: partial_name)
          attach_partial_children!(timing, node)
          timing
        when IfNode
          build_if_timing(node, partial_name: partial_name)
        when ForNode
          items = resolve_collection(node.collection_markup)
          children = items.flat_map do |item|
            with_iteration_scope(node.variable_name, item) { build_timings(node.body, partial_name: partial_name) }
          end
          timing = Timing.new(code: node.raw, template_name: partial_name, line_number: node.line_number)
          timing.children.concat(children)
          timing.total_time = node_total_time(children)
          timing
        end
      end

      def branch_timing(node, body, partial_name:)
        children = build_timings(body, partial_name: partial_name)
        timing = Timing.new(code: node.raw, template_name: partial_name, line_number: node.line_number)
        timing.children.concat(children)
        timing.total_time = node_total_time(children)
        timing
      end

      def build_if_timing(node, partial_name:)
        return branch_timing(node, node.body, partial_name: partial_name) if truthy?(node.condition_markup)

        branch = node.elsif_branches.find do |elsif_branch|
          truthy?(elsif_branch[:condition_markup])
        end

        body =
          if branch
            branch[:body]
          else
            node.else_body || []
          end

        branch_timing(node, body, partial_name: partial_name)
      end

      def attach_partial_children!(timing, node)
        return unless %w[include render].include?(node.name)

        partial_name = extract_partial_name(node.markup)
        return unless partial_name

        source = load_partial_source(partial_name)
        return unless source

        children = parse_nodes(source).then { |descriptors| build_timings(descriptors, partial_name: partial_name) }
        timing.children.concat(children)
        timing.total_time = node_total_time(children)
      end

      def leaf_timing(code:, line_number:, partial_name:)
        Timing.new(code: code, template_name: partial_name, line_number: line_number).tap do |timing|
          timing.total_time = PROFILE_EPSILON
        end
      end

      def node_total_time(children)
        total = children.sum(&:total_time)
        total.positive? ? total + PROFILE_EPSILON : PROFILE_EPSILON
      end

      def parse_for_markup(markup)
        match = FOR_SYNTAX.match(markup.strip)
        return [nil, nil] unless match

        [match[1], match[2].to_s.strip]
      end

      def resolve_collection(markup)
        return [] if markup.nil? || markup.empty?

        expression = Liquid::Expression.parse(markup)
        value = @context.evaluate(expression)
        value.respond_to?(:to_a) ? value.to_a : Array(value)
      rescue StandardError
        []
      end

      def with_iteration_scope(variable_name, item)
        return yield if variable_name.nil? || variable_name.empty?

        @context.stack(variable_name => item) { yield }
      end

      def truthy?(markup)
        expression = Liquid::Expression.parse(markup)
        value = @context.evaluate(expression)
        !value.nil? && value != false
      rescue StandardError
        false
      end

      def extract_partial_name(markup)
        stripped = markup.to_s.strip
        return Regexp.last_match(1) if stripped =~ /\A'([^']+)'/
        return Regexp.last_match(1) if stripped =~ /\A"([^"]+)"/

        nil
      end

      def load_partial_source(partial_name)
        file_system = @context.registers[:file_system] || @context.registers.static[:file_system]
        file_system&.read_template_file(partial_name)
      rescue StandardError
        nil
      end

      def join_tag_raw(name, markup)
        [name, markup].reject(&:empty?).join(" ")
      end

      def extract_variable_raw(token)
        token.sub(/\A\{\{-?/, "").sub(/-?\}\}\z/, "")
      end
    end

    attr_reader :total_time
    alias_method :total_render_time, :total_time

    def initialize
      @root_children = []
      @total_time = 0.0
    end

    def record_render(template_name:, total_time:, children:)
      timing = Timing.new(template_name: template_name)
      timing.children.concat(children)
      timing.total_time = total_time.positive? ? total_time : PROFILE_EPSILON
      @root_children << timing
      @total_time += timing.total_time
      timing
    end

    def children
      if @root_children.length == 1
        @root_children.first.children
      else
        @root_children
      end
    end

    def each(&block)
      children.each(&block)
    end

    def [](idx)
      children[idx]
    end

    def length
      children.length
    end
  end
end
