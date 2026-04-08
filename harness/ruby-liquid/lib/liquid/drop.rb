# frozen_string_literal: true

require "set"

module Liquid
  class Drop
    attr_writer :context

    def initialize
      @context = nil
    end

    def liquid_method_missing(method)
      return unless @context&.strict_variables

      raise Liquid::UndefinedDropMethod, "undefined method #{method}"
    end

    def invoke_drop(method_or_key)
      result = if self.class.invokable?(method_or_key)
        send(method_or_key)
      else
        liquid_method_missing(method_or_key)
      end

      result.context = @context if @context && result.respond_to?(:context=)
      result
    end

    def key?(_name)
      true
    end

    def inspect
      self.class.to_s
    end

    def to_liquid
      self
    end

    def to_s
      self.class.name
    end

    alias [] invoke_drop

    def self.invokable?(method_name)
      invokable_methods.include?(method_name.to_s)
    end

    def self.invokable_methods
      @invokable_methods ||= begin
        blacklist = Liquid::Drop.public_instance_methods + [:each]

        if include?(Enumerable)
          blacklist += Enumerable.public_instance_methods
          blacklist -= [:sort, :count, :first, :min, :max]
        end

        whitelist = [:to_liquid] + (public_instance_methods - blacklist)
        Set.new(whitelist.map(&:to_s))
      end
    end
  end
end
