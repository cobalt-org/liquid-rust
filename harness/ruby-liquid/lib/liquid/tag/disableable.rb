# frozen_string_literal: true

module Liquid
  class Tag
    module Disableable
      def render_to_output_buffer(context, output)
        if context.tag_disabled?(tag_name)
          output << disabled_error(context)
          return output
        end

        super
      end

      def disabled_error(context)
        raise Liquid::DisabledError, "#{tag_name} #{parse_context.locale.t('errors.disabled.tag')}"
      rescue Liquid::DisabledError => error
        context.handle_error(error, line_number)
      end
    end
  end
end
