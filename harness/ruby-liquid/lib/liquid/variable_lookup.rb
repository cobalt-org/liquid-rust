# frozen_string_literal: true

module Liquid
  class VariableLookup
    COMMAND_METHODS = ["size", "first", "last"].freeze

    attr_reader :name, :lookups

    def self.parse(markup, string_scanner = StringScanner.new(""), cache = nil)
      new(markup, string_scanner, cache)
    end

    def initialize(markup, string_scanner = StringScanner.new(""), cache = nil)
      lookups = markup.scan(VariableParser)

      name = lookups.shift
      if name&.start_with?("[") && name&.end_with?("]")
        name = Expression.parse(name[1..-2], string_scanner, cache)
      end
      @name = name

      @lookups = lookups
      @command_flags = 0

      @lookups.each_index do |i|
        lookup = lookups[i]
        if lookup&.start_with?("[") && lookup&.end_with?("]")
          lookups[i] = Expression.parse(lookup[1..-2], string_scanner, cache)
        elsif COMMAND_METHODS.include?(lookup)
          @command_flags |= 1 << i
        end
      end
    end

    def lookup_command?(lookup_index)
      @command_flags & (1 << lookup_index) != 0
    end

    def ==(other)
      other.is_a?(VariableLookup) &&
        name == other.name &&
        lookups == other.lookups &&
        @command_flags == other.instance_variable_get(:@command_flags)
    end
    alias eql? ==

    def hash
      [name, lookups, @command_flags].hash
    end

    def evaluate(context)
      name = context.evaluate(@name)
      object = context.find_variable(name)

      @lookups.each_index do |i|
        key = context.evaluate(@lookups[i])
        key = Liquid::Utils.to_liquid_value(key)

        if object.respond_to?(:[]) &&
            ((object.respond_to?(:key?) && object.key?(key)) ||
             (object.respond_to?(:fetch) && key.is_a?(Integer)))
          res = context.lookup_and_evaluate(object, key)
          object = res.respond_to?(:to_liquid) ? res.to_liquid : res
        elsif lookup_command?(i) && object.respond_to?(key)
          res = object.public_send(key)
          object = res.respond_to?(:to_liquid) ? res.to_liquid : res
        elsif lookup_command?(i) && object.is_a?(String) && (key == "first" || key == "last")
          object = key == "first" ? (object[0] || "") : (object[-1] || "")
        else
          return nil unless context.strict_variables

          raise Liquid::UndefinedVariable, "undefined variable #{key}"
        end

        object.context = context if object.respond_to?(:context=)
      end

      object
    end

    class ParseTreeVisitor < Liquid::ParseTreeVisitor
      def children
        @node.lookups
      end
    end
  end
end
