# frozen_string_literal: true

module Htpage
  class TemplateHandler
    def self.call(template, source = nil)
      source ||= template.source
      compiled_html = Htpage.compile(source)
      "%Q(#{compiled_html})"
    end
  end
end
