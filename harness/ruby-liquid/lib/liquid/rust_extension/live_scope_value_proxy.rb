# frozen_string_literal: true

module Liquid
  module RustExtension
    class LiveScopeValueProxy
      def initialize(original, semantic = nil)
        @original = original
        @semantic = semantic || original
      end

      def to_liquid
        @semantic
      end

      def to_liquid_value
        return @semantic.to_liquid_value if @semantic.respond_to?(:to_liquid_value)
        return @semantic.to_liquid if @semantic.respond_to?(:to_liquid)

        @semantic
      end

      def context=(context)
        @semantic.context = context if @semantic.respond_to?(:context=)
        @original.context = context if @original.respond_to?(:context=)
      end

      def liquid_render_value
        Liquid::Utils.to_s(@original)
      end

      def to_s
        liquid_render_value
      end

      def inspect
        @original.inspect
      end

      def respond_to_missing?(name, include_private = false)
        @semantic.respond_to?(name, include_private) ||
          @original.respond_to?(name, include_private) ||
          super
      end

      def method_missing(name, *args, &block)
        return @semantic.public_send(name, *args, &block) if @semantic.respond_to?(name)
        return @original.public_send(name, *args, &block) if @original.respond_to?(name)

        super
      end
    end
  end
end
