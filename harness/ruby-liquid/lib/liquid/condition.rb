# frozen_string_literal: true

module Liquid
  class Condition
    @@operators = {
      "==" => ->(cond, left, right) { cond.send(:equal_variables, left, right) },
      "!=" => ->(cond, left, right) { !cond.send(:equal_variables, left, right) },
      "<>" => ->(cond, left, right) { !cond.send(:equal_variables, left, right) },
      "<" => :<,
      ">" => :>,
      ">=" => :>=,
      "<=" => :<=,
      "contains" => lambda do |_cond, left, right|
        if left && right && left.respond_to?(:include?)
          right = right.to_s if left.is_a?(String)
          left.include?(right)
        else
          false
        end
      rescue Encoding::CompatibilityError
        left.b.include?(right.b)
      end,
    }

    class MethodLiteral
      attr_reader :method_name, :to_s

      def initialize(method_name, to_s)
        @method_name = method_name
        @to_s = to_s
      end
    end

    @@method_literals = {
      "blank" => MethodLiteral.new(:blank?, "").freeze,
      "empty" => MethodLiteral.new(:empty?, "").freeze,
    }

    def self.operators
      @@operators
    end

    def self.parse_expression(parse_context, markup, safe: false)
      @@method_literals[markup] || parse_context.parse_expression(markup, safe: safe)
    end

    attr_reader :attachment, :child_condition
    attr_accessor :left, :operator, :right

    def initialize(left = nil, operator = nil, right = nil)
      @left = left
      @operator = operator
      @right = right
      @child_relation = nil
      @child_condition = nil
    end

    def evaluate(context = deprecated_default_context)
      condition = self
      result = nil

      loop do
        result = interpret_condition(condition.left, condition.right, condition.operator, context)

        case condition.child_relation
        when :or
          break if Liquid::Utils.to_liquid_value(result)
        when :and
          break unless Liquid::Utils.to_liquid_value(result)
        else
          break
        end

        condition = condition.child_condition
      end

      result
    end

    def or(condition)
      @child_relation = :or
      @child_condition = condition
    end

    def and(condition)
      @child_relation = :and
      @child_condition = condition
    end

    def attach(attachment)
      @attachment = attachment
    end

    def else?
      false
    end

    def inspect
      "#<Condition #{[@left, @operator, @right].compact.join(' ')}>"
    end

    protected

    attr_reader :child_relation

    private

    def equal_variables(left, right)
      return call_method_literal(left, right) if left.is_a?(MethodLiteral)
      return call_method_literal(right, left) if right.is_a?(MethodLiteral)

      left == right
    end

    def call_method_literal(literal, value)
      method_name = literal.method_name

      if value.respond_to?(method_name)
        value.send(method_name)
      else
        case method_name
        when :blank?
          liquid_blank?(value)
        when :empty?
          liquid_empty?(value)
        else
          false
        end
      end
    end

    def liquid_blank?(value)
      case value
      when NilClass, FalseClass
        true
      when TrueClass, Numeric
        false
      when String
        value.empty? || value.match?(/\A\s*\z/)
      when Array, Hash
        value.empty?
      else
        value.respond_to?(:empty?) ? value.empty? : false
      end
    end

    def liquid_empty?(value)
      case value
      when String, Array, Hash
        value.empty?
      else
        value.respond_to?(:empty?) ? value.empty? : false
      end
    end

    def interpret_condition(left, right, operator, context)
      return context.evaluate(left) if operator.nil?

      left = Liquid::Utils.to_liquid_value(context.evaluate(left))
      right = Liquid::Utils.to_liquid_value(context.evaluate(right))

      operation = self.class.operators[operator] || raise(Liquid::ArgumentError, "Unknown operator #{operator}")

      if operation.respond_to?(:call)
        operation.call(self, left, right)
      elsif left.respond_to?(operation) && right.respond_to?(operation) && !left.is_a?(Hash) && !right.is_a?(Hash)
        begin
          left.send(operation, right)
        rescue ::ArgumentError => error
          raise Liquid::ArgumentError, error.message
        end
      end
    end

    def deprecated_default_context
      warn(
        "DEPRECATION WARNING: Condition#evaluate without a context argument is deprecated " \
          "and will be removed from Liquid 6.0.0."
      )
      Context.new
    end

    class ParseTreeVisitor < Liquid::ParseTreeVisitor
      def children
        [
          @node.left,
          @node.right,
          @node.child_condition,
          @node.attachment,
        ].compact
      end
    end
  end

  class ElseCondition < Condition
    def else?
      true
    end

    def evaluate(_context)
      true
    end
  end
end
