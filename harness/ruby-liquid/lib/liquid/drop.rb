# frozen_string_literal: true

module Liquid
  class Drop
    attr_writer :context

    def context
      @context
    end

    def to_liquid
      self
    end

    def liquid_method_missing(_method)
      nil
    end

    def invoke_drop(method_name)
      if respond_to?(method_name)
        public_send(method_name)
      else
        liquid_method_missing(method_name.to_s)
      end
    end

    def key?(name)
      respond_to?(name) || !liquid_method_missing(name.to_s).nil?
    end

    def [](name)
      invoke_drop(name)
    end

    def inspect
      "#<#{self.class.name}>"
    end

    def to_s
      self.class.name.split("::").last.to_s
    end
  end
end
