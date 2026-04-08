# frozen_string_literal: true

module Liquid
  class Registers
    attr_reader :static

    UNDEFINED = Object.new

    def initialize(registers = {}, file_system: nil, template_factory: nil, **kwargs)
      @static =
        case registers
        when Registers
          registers.static
        when nil
          {}
        when Hash
          registers.dup
        else
          raise Liquid::ArgumentError, "expected Hash or Liquid::Registers"
        end
      @changes = {}

      @static[:file_system] = file_system if file_system
      @static[:template_factory] = template_factory if template_factory
      @static.merge!(kwargs) unless kwargs.empty?
    end

    def []=(key, value)
      @changes[key] = value
    end

    def [](key)
      if @changes.key?(key)
        @changes[key]
      else
        @static[key]
      end
    end

    def delete(key)
      @changes.delete(key)
    end

    def fetch(key, default = UNDEFINED, &block)
      if @changes.key?(key)
        @changes.fetch(key)
      elsif default != UNDEFINED
        if block_given?
          @static.fetch(key, &block)
        else
          @static.fetch(key, default)
        end
      else
        @static.fetch(key, &block)
      end
    end

    def key?(key)
      @changes.key?(key) || @static.key?(key)
    end

    def to_h
      @static.merge(@changes)
    end
  end

  StaticRegisters = Registers
end
