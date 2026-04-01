# frozen_string_literal: true

module Liquid
  class Registers
    attr_reader :static

    def initialize(registers = nil, file_system: nil, template_factory: nil, **kwargs)
      @static =
        case registers
        when nil
          {}
        when Hash
          registers.dup
        else
          raise Liquid::ArgumentError, "expected Hash or keyword arguments for registers"
        end

      @static[:file_system] = file_system if file_system
      @static[:template_factory] = template_factory if template_factory
      @static.merge!(kwargs) unless kwargs.empty?
    end

    def [](key)
      @static[key]
    end

    def []=(key, value)
      @static[key] = value
    end

    def to_h
      @static.dup
    end
  end
end
